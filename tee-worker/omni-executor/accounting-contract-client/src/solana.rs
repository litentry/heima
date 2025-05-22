use std::str::FromStr;

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
use sp_core::ed25519;
use tracing::{error, info, warn};

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

pub trait AccountingContractApi: Send + Sync {
	fn execute_pay_out_request(
		&self,
		beneficiary: Pubkey,
		nonce: u64,
		amount: U256,
	) -> Result<(), ()>;

	fn get_nonce(&self, user: Pubkey) -> Result<u64, ()>;

	fn get_balance(&self) -> Result<U256, ()>;
}

pub struct AccountingContractClient {
	pub payer: Keypair,
	pub program_id: Pubkey,
}

impl AccountingContractClient {
	pub fn new(pair: ed25519::Pair, program_id: String) -> Self {
		let payer =
			Keypair::from_seed(pair.seed().as_slice()).expect("Failed to create keypair from seed");
		let program_id =
			Pubkey::from_str(program_id.as_str()).expect("Failed to parse program ID from string");

		Self { payer, program_id }
	}
}

impl AccountingContractApi for AccountingContractClient {
	fn execute_pay_out_request(
		&self,
		beneficiary: Pubkey,
		nonce: u64,
		amount: U256,
	) -> Result<(), ()> {
		let client =
			Client::new_with_options(Cluster::Mainnet, &self.payer, CommitmentConfig::confirmed());
		let program = client.program(self.program_id).expect("Failed to create program client");

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
				signer: self.payer.pubkey(),
				worker: Pubkey::find_program_address(&[b"worker"], &program.id()).0,
				treasury: Pubkey::find_program_address(&[b"treasury"], &program.id()).0,
				beneficiary,
				system_program: system_program::ID,
			})
			.args(accounting_contract::instruction::CreatePayRequest {
				amount: amount.try_into().map_err(|_| {
					error!("Failed to convert U256 to u64");
				})?,
				nonce,
			})
			.send()
			.map_err(|e| {
				error!("Failed to execute pay out request: {:?}", e);
			})?;
		Ok(())
	}

	fn get_nonce(&self, user: Pubkey) -> Result<u64, ()> {
		let client =
			Client::new_with_options(Cluster::Mainnet, &self.payer, CommitmentConfig::confirmed());

		let (account_nonce, _bump) =
			Pubkey::find_program_address(&[user.to_bytes().as_ref(), b"nonce"], &self.program_id);

		let program = client.program(self.program_id).expect("Failed to create program client");

		let nonce_account: NonceAccount = program.account(account_nonce).map_err(|e| {
			if e.to_string().contains("AccountNotFound") {
				info!("Nonce account {} not found, returning 0", account_nonce);
			} else {
				error!("Failed to get nonce account {}: {:?}", account_nonce, e);
			}
		})?;

		Ok(nonce_account.nonce)
	}

	fn get_balance(&self) -> Result<U256, ()> {
		let client =
			Client::new_with_options(Cluster::Mainnet, &self.payer, CommitmentConfig::confirmed());
		let program = client.program(self.program_id).expect("Failed to create program client");
		match program.rpc().get_balance(&self.payer.pubkey()) {
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
	use mockall::mock;

	mock! {

		pub AccountingContractClient {}

		impl AccountingContractApi for AccountingContractClient {

			fn execute_pay_out_request(
				&self,
				beneficiary: Pubkey,
				nonce: u64,
				amount: U256,
			) -> Result<(), ()>;

			fn get_nonce(&self, user: Pubkey) -> Result<u64, ()>;

			fn get_balance(&self) -> Result<U256, ()>;
		}
	}
}
