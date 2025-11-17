use std::str::FromStr;

use crate::AccountingContractApi;
use alloy::primitives::U256;
use anchor_client::{
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
use std::sync::Arc;
use tracing::{error, warn};

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
impl AccountingContractApi<Pubkey, u64> for AccountingContractClient {
	async fn execute_pay_out_request(
		&self,
		beneficiary: Pubkey,
		nonce: u64,
		amount: U256,
	) -> Result<(), ()> {
		let client = self.create_client()?;

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
					error!("Failed to convert U256 amount to u64");
				})?,
				nonce,
			})
			.send()
			.await
			.map_err(|e| {
				error!("Failed to execute pay out request: {:?}", e);
			})?;
		Ok(())
	}

	async fn get_nonce(&self, user: Pubkey) -> Result<u64, ()> {
		let client = self.create_client()?;

		let program = client.program(self.program_id).map_err(|e| {
			error!("Failed to create program client: {:?}", e);
		})?;

		let (account_pubkey, _bump) =
			Pubkey::find_program_address(&[user.to_bytes().as_ref(), b"nonce"], &self.program_id);

		match program.rpc().get_account(&account_pubkey).await {
			// return default nonce 0 when fail to get nonce
			Ok(account) => {
				match bincode::decode_from_slice::<accounting_contract::Nonce, _>(
					&account.data[8..],
					bincode::config::standard(),
				) {
					Ok((nonce, _)) => Ok(nonce.nonce),
					Err(e) => {
						error!("Failed deserialize nonce from account {:?}: {:?}", account, e);
						Ok(0)
					},
				}
			},
			Err(e) => {
				error!("Failed to get_account {:?}", e);
				Ok(0)
			},
		}
	}

	async fn get_balance(&self) -> Result<U256, ()> {
		let client = self.create_client()?;

		let program = client.program(self.program_id).expect("Failed to create program client");
		match program.rpc().get_balance(&self.payer.pubkey()).await {
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
	use crate::AccountingContractApi;
	use crate::U256;
	use anchor_client::solana_sdk::pubkey::Pubkey;
	use async_trait::async_trait;
	use mockall::mock;

	mock! {

		pub AccountingContractClient {}

		#[async_trait]
		impl AccountingContractApi<Pubkey, u64> for AccountingContractClient {

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

#[cfg(test)]
mod tests {
	use super::*;
	use sp_core::{ed25519, Pair};

	#[tokio::test]
	async fn test_mock_server_execute_pay_out_request() {
		// Start mock server and get dynamic URL
		let mock_url = mock_server::async_run_test_only().await;
		let param_id = "D3S1ZTrFNkfeoHaLSTAjMXZVXnRJvsNnbwh9k5mRYqqV";

		// has balance
		// address: AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9
		let seed = [1u8; 32];
		let pair = ed25519::Pair::from_seed(&seed);
		let client = AccountingContractClient::new(
			pair,
			format!("{}/solana", mock_url),
			param_id.to_string(),
		);

		// Test execute_pay_out_request
		// base64 encoded tx string (sendTransaction):
		// AaV8xKPN0GfY3XVuTiQsxFPDOFlZhXhWwbcVMBEGd2kUTgid2Ss9GUNqJ1S01iTyvDKCWea8sRMwlmlZ1A8zDAEBAAMIiojj3XQJ8ZX9UtstPLpdcspnCb8dlBIb83SIAbQPb1zG8DcMUx1vBnO2Al4wQ7nCKjwgl/c5iVbUF2HhIwZJ3ez0N0UQ/aETrzeokAeTDaVjdVNHLlFhPu0cKQjhxBso7UkoxijRwsbq6QM4kFmVYSlZJzpcY/k2NsFGFKyHN9Hx9pMmCdAce72TK7SkfatcvRdHPxk+YN0BruCENR+3BgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAsuuu6Iwn1QSzfnYrRIa2f/i5FCzjnwDv32SN1j4fahTyq0V7Xk41kUInl1rH0tI9eWEbcpDkAbgz89/ZTaRNLMxJDpKM0uOHO7ND/JXaMxecpg9Nv0bCw26RKZ1V1Oa5AQYHAgEABwQDBRiE8aphpOG6LgEAAAAAAAAAAMqaOwAAAAA=
		let beneficiary_seed = [3u8; 32];
		let beneficiary_keypair =
			Keypair::from_seed(&beneficiary_seed).expect("Failed to create beneficiary keypair");
		let beneficiary = beneficiary_keypair.pubkey();
		let nonce = 1u64;
		let amount = U256::from(1000000000u64); // 1 SOL
		let result = client.execute_pay_out_request(beneficiary, nonce, amount).await;
		assert!(result.is_ok());
	}

	#[tokio::test]
	async fn test_mock_server_get_balance() {
		// Start mock server and get dynamic URL
		let mock_url = mock_server::async_run_test_only().await;
		let param_id = "D3S1ZTrFNkfeoHaLSTAjMXZVXnRJvsNnbwh9k5mRYqqV";

		// has balance
		// address: AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9
		let seed = [1u8; 32];
		let pair = ed25519::Pair::from_seed(&seed);
		let client = AccountingContractClient::new(
			pair,
			format!("{}/solana", mock_url),
			param_id.to_string(),
		);
		let balance = client.get_balance().await.expect("Fail to get balance");
		assert_eq!(balance, U256::from(1000000000u64));

		// has no balance
		// address: 9hSR6S7WPtxmTojgo6GG3k4yDPecgJY292j7xrsUGWBu
		let seed = [2u8; 32];
		let pair = ed25519::Pair::from_seed(&seed);
		let client = AccountingContractClient::new(
			pair,
			format!("{}/solana", mock_url),
			param_id.to_string(),
		);
		let balance = client.get_balance().await.expect("Fail to get balance");
		assert_eq!(balance, U256::from(0));
	}
}
