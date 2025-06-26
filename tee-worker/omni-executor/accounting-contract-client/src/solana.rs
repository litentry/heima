use std::str::FromStr;
use std::sync::Arc;

use alloy::primitives::U256;
use anchor_client::{
	anchor_lang::AccountDeserialize,
	solana_sdk::{
		commitment_config::CommitmentConfig,
		pubkey::Pubkey,
		signature::{Keypair, Signer},
		signer::SeedDerivable,
		system_program,
	},
	Client, Cluster,
};
use async_trait::async_trait;
use sp_core::ed25519;
use tracing::{error, warn};

#[derive(Debug)]
pub struct NonceAccount {
	pub nonce: u64,
}

impl AccountDeserialize for NonceAccount {
	fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_client::anchor_lang::Result<Self> {
		if buf.len() < 8 {
			return Ok(NonceAccount { nonce: 0 });
		}

		let data = &buf[8..];

		if data.len() < 8 {
			return Ok(NonceAccount { nonce: 0 });
		}

		let nonce = u64::from_le_bytes(data[..8].try_into().map_err(|_| {
			anchor_client::anchor_lang::error::Error::from(
				anchor_client::anchor_lang::error::ErrorCode::AccountDidNotDeserialize,
			)
		})?);

		Ok(NonceAccount { nonce })
	}
}

#[async_trait]
pub trait AccountingContractApi: Send + Sync {
	async fn execute_pay_out_request(
		&self,
		beneficiary: Pubkey,
		nonce: u64,
		amount: U256,
	) -> Result<(), ()>;

	async fn get_nonce(&self, user: Pubkey) -> Result<u64, ()>;

	async fn get_balance(&self) -> Result<U256, ()>;
}

pub struct AccountingContractClient {
	pub payer: Keypair,
	pub solana_url: String,
	pub program_id: Pubkey,
}

impl AccountingContractClient {
	pub fn new(pair: ed25519::Pair, solana_url: String, program_id: String) -> Self {
		let payer =
			Keypair::from_seed(pair.seed().as_slice()).expect("Failed to create keypair from seed");
		let program_id =
			Pubkey::from_str(program_id.as_str()).expect("Failed to parse program ID from string");

		Self { payer, solana_url, program_id }
	}

	fn create_client(&self) -> Result<Client<Arc<Keypair>>, ()> {
		let cluster = Cluster::from_str(&self.solana_url).map_err(|err| {
			error!("Failed to create solana cluster: {:?}", err);
		})?;
		let payer = Arc::new(Keypair::from_bytes(&self.payer.to_bytes()).map_err(|err| {
			error!("Failed to clone keypair: {:?}", err);
		})?);
		Ok(Client::new_with_options(cluster, payer, CommitmentConfig::confirmed()))
	}
}

#[async_trait]
impl AccountingContractApi for AccountingContractClient {
	async fn execute_pay_out_request(
		&self,
		beneficiary: Pubkey,
		nonce: u64,
		amount: U256,
	) -> Result<(), ()> {
		let client = self.create_client()?;
		let program = client.program(self.program_id).map_err(|e| {
			error!("Failed to create program client: {:?}", e);
		})?;

		let result = tokio::task::spawn_blocking(move || {
			program
				.request()
				.accounts(accounting_contract::accounts::CreatePayRequest {
					pay_out_request: Pubkey::find_program_address(
						&[beneficiary.to_bytes().as_ref(), &nonce.to_le_bytes(), b"payout_request"],
						&program.id(),
					)
					.0,
					account_nonce: Pubkey::find_program_address(
						&[beneficiary.to_bytes().as_ref(), b"nonce"],
						&program.id(),
					)
					.0,
					signer: program.payer(),
					worker: Pubkey::find_program_address(&[b"worker"], &program.id()).0,
					treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
					beneficiary,
					system_program: system_program::ID,
				})
				.args(accounting_contract::instruction::CreatePayRequest {
					amount: amount.try_into().map_err(|_| {
						error!("Failed to convert U256 to u64");
						anchor_client::ClientError::from(std::io::Error::new(
							std::io::ErrorKind::InvalidData,
							"Failed to convert U256 to u64",
						))
					})?,
					nonce,
				})
				.send()
		})
		.await;

		match result {
			Ok(anchor_result) => match anchor_result {
				Ok(_) => Ok(()),
				Err(e) => {
					error!("Anchor client error: {:?}", e);
					Err(())
				},
			},
			Err(e) => {
				error!("Failed to execute pay out request: {:?}", e);
				Err(())
			},
		}
	}

	async fn get_nonce(&self, user: Pubkey) -> Result<u64, ()> {
		let client = self.create_client()?;

		let program = client.program(self.program_id).map_err(|e| {
			error!("Failed to create program client: {:?}", e);
		})?;

		let nonce_account: NonceAccount =
			tokio::task::spawn_blocking(move || program.account(user))
				.await
				.map_err(|e| {
					error!("Failed to spawn blocking task: {:?}", e);
				})?
				.map_err(|e| {
					error!("Failed to get nonce account {}: {:?}", user, e);
				})?;

		Ok(nonce_account.nonce)
	}

	async fn get_balance(&self) -> Result<U256, ()> {
		let client = self.create_client()?;
		let program = client.program(self.program_id).map_err(|e| {
			error!("Failed to create program client: {:?}", e);
		})?;

		let payer_pubkey = self.payer.pubkey();
		let balance_result =
			tokio::task::spawn_blocking(move || program.rpc().get_balance(&payer_pubkey))
				.await
				.map_err(|e| {
					error!("Failed to spawn blocking task: {:?}", e);
				})?;

		match balance_result {
			Ok(v) => {
				let balance = U256::try_from(v).map_err(|err| {
					warn!("Failed to convert balance to U256: {:?}", err);
				})?;
				Ok(balance)
			},
			Err(e) => {
				warn!("Failed to get balance: {:?}", e);
				Err(())
			},
		}
	}
}

#[cfg(feature = "mocks")]
pub mod mocks {
	use crate::solana::AccountingContractApi;
	use crate::U256;
	use anchor_client::solana_sdk::pubkey::Pubkey;
	use async_trait::async_trait;
	use mockall::mock;

	mock! {
		pub AccountingContractClient {}

		#[async_trait]
		impl AccountingContractApi for AccountingContractClient {
			async fn execute_pay_out_request(
				&self,
				beneficiary: Pubkey,
				nonce: u64,
				amount: U256,
			) -> Result<(), ()>;

			async fn get_nonce(&self, user: Pubkey) -> Result<u64, ()>;

			async fn get_balance(&self) -> Result<U256, ()>;
		}
	}
}
