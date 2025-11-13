pub mod signer;

use async_trait::async_trait;
use oe_core::wallet_metrics::WalletBalanceFetcher;
use solana_client::rpc_response::RpcKeyedAccount;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_request::TokenAccountsFilter};
use solana_sdk::{
	commitment_config::CommitmentConfig, pubkey::Pubkey, signer::Signer as SignerTrait,
	system_instruction, transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;
use tracing::{debug, error};

#[async_trait]
pub trait SolanaClient: Send + Sync {
	async fn transfer_sol<Signer: SignerTrait + Send + Sync>(
		&self,
		to: &str,
		value: u64,
		signer: &Signer,
	) -> Result<String, ()>;

	async fn transfer_spl<Signer: SignerTrait + Send + Sync>(
		&self,
		to: &str,
		value: u64,
		mint_address: &str,
		signer: &Signer,
	) -> Result<String, ()>;

	async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64, ()>;

	async fn get_token_accounts_by_owner(
		&self,
		owner: &Pubkey,
		token_account_filter: TokenAccountsFilter,
	) -> Result<Vec<RpcKeyedAccount>, ()>;
}

pub struct SolanaRpcClient {
	rpc_client: RpcClient,
}

impl SolanaRpcClient {
	pub fn new(rpc_url: &str) -> Self {
		let client =
			RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
		Self { rpc_client: client }
	}
}

#[async_trait]
impl SolanaClient for SolanaRpcClient {
	async fn transfer_sol<Signer: SignerTrait + Send + Sync>(
		&self,
		to: &str,
		value: u64,
		signer: &Signer,
	) -> Result<String, ()> {
		let block_hash = self
			.rpc_client
			.get_latest_blockhash()
			.await
			.map_err(|e| error!("Could not get block hash: {:?}", e))?;
		let to_pubkey =
			Pubkey::from_str(to).map_err(|e| error!("Could not parse to address: {:?}", e))?;
		let transfer_instruction =
			system_instruction::transfer(&signer.pubkey(), &to_pubkey, value);
		let tx = Transaction::new_signed_with_payer(
			&[transfer_instruction],
			Some(&signer.pubkey()),
			&[signer],
			block_hash,
		);
		let tx_signature = self
			.rpc_client
			.send_and_confirm_transaction(&tx)
			.await
			.map_err(|e| error!("Could not send transaction: {:?}", e))?;

		debug!(
			"Successfully transferred {} tokens from sender {} to {}",
			value,
			signer.pubkey(),
			to_pubkey
		);
		debug!("Transaction signature: {:?}", tx_signature);

		Ok(tx_signature.to_string())
	}

	async fn transfer_spl<Signer: SignerTrait + Send + Sync>(
		&self,
		to: &str,
		value: u64,
		mint_address: &str,
		signer: &Signer,
	) -> Result<String, ()> {
		let mint_pubkey = Pubkey::from_str(mint_address)
			.map_err(|e| error!("Could not parse mint address: {:?}", e))?;
		let to_pubkey =
			Pubkey::from_str(to).map_err(|e| error!("Could not parse to address: {:?}", e))?;

		let source_pubkey = get_associated_token_address(&signer.pubkey(), &mint_pubkey);
		let destination_pubkey = get_associated_token_address(&to_pubkey, &mint_pubkey);

		let destination_exists = self.rpc_client.get_account(&destination_pubkey).await.is_ok();

		// if dest ATA doesn't exist, send one tx to create it, two ix in one tx didn't seem to work
		// this should happen only once as the dest ATA would be initialized after one interaction
		if !destination_exists {
			debug!("Destination ATA {} doesn't exist, creating it now", destination_pubkey);
			let create_ata_ix =
				spl_associated_token_account::instruction::create_associated_token_account(
					&signer.pubkey(), // payer
					&to_pubkey,       // owner of the new ATA
					&mint_pubkey,     // token mint
					&spl_token::id(),
				);
			let block_hash = self
				.rpc_client
				.get_latest_blockhash()
				.await
				.map_err(|e| error!("Could not get block hash: {:?}", e))?;

			let tx = Transaction::new_signed_with_payer(
				&[create_ata_ix],
				Some(&signer.pubkey()),
				&[signer],
				block_hash,
			);

			let tx_signature = self
				.rpc_client
				.send_and_confirm_transaction(&tx)
				.await
				.map_err(|e| error!("Could not send transaction: {:?}", e))?;

			debug!("Successfully created destination ATA, tx signature {:?}", tx_signature);
		}

		let block_hash = self
			.rpc_client
			.get_latest_blockhash()
			.await
			.map_err(|e| error!("Could not get block hash: {:?}", e))?;

		let transfer_ix = spl_token::instruction::transfer(
			&spl_token::id(),
			&source_pubkey,
			&destination_pubkey,
			&signer.pubkey(),
			&[&signer.pubkey()],
			value,
		)
		.map_err(|e| {
			error!("Could not create transfer instruction: {:?}", e);
		})?;

		let tx = Transaction::new_signed_with_payer(
			&[transfer_ix],
			Some(&signer.pubkey()),
			&[signer],
			block_hash,
		);
		let tx_signature = self
			.rpc_client
			.send_and_confirm_transaction(&tx)
			.await
			.map_err(|e| error!("Could not send transaction: {:?}", e))?;

		debug!(
			"Successfully transferred {} tokens from sender {} to {}",
			value,
			signer.pubkey(),
			to_pubkey
		);
		debug!("Transaction signature: {:?}", tx_signature);

		Ok(tx_signature.to_string())
	}

	async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64, ()> {
		self.rpc_client
			.get_balance(pubkey)
			.await
			.map_err(|e| error!("Could not get balance: {:?}", e))
	}

	async fn get_token_accounts_by_owner(
		&self,
		owner: &Pubkey,
		token_account_filter: TokenAccountsFilter,
	) -> Result<Vec<RpcKeyedAccount>, ()> {
		self.rpc_client
			.get_token_accounts_by_owner(owner, token_account_filter)
			.await
			.map_err(|e| error!("Could not get token accounts by owner: {:?}", e))
	}
}

#[async_trait]
impl WalletBalanceFetcher for SolanaRpcClient {
	async fn fetch(&self, address: &str) -> Result<f64, ()> {
		let pubkey =
			Pubkey::from_str(address).map_err(|e| error!("Could not parse pubkey: {:?}", e))?;
		self.rpc_client
			.get_balance(&pubkey)
			.await
			.map_err(|e| error!("Could not fetch wallet balance: {:?}", e))
			.map(|b| b as f64)
	}
}

#[cfg(feature = "mocks")]
pub mod mocks {

	use crate::SignerTrait;
	use crate::SolanaClient;
	use async_trait::async_trait;
	use mockall::mock;
	use solana_client::rpc_request::TokenAccountsFilter;
	use solana_client::rpc_response::RpcKeyedAccount;
	use solana_sdk::pubkey::Pubkey;

	mock! {
		pub SolanaRpcClient {}

		#[async_trait]
		impl SolanaClient for SolanaRpcClient {

			#[mockall::concretize]
			async fn transfer_sol<Signer: SignerTrait + Send + Sync>(
				&self,
				to: &str,
				value: u64,
				signer: &Signer,
			) -> Result<String, ()>;

			#[mockall::concretize]
			async fn transfer_spl<Signer: SignerTrait + Send + Sync>(
				&self,
				to: &str,
				value: u64,
				mint_address: &str,
				signer: &Signer,
			) -> Result<String, ()>;

			async fn get_token_accounts_by_owner(&self, owner: &Pubkey, token_account_filter: TokenAccountsFilter) -> Result<Vec<RpcKeyedAccount>, ()>;
			async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64, ()>;
		}

	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use solana_sdk::{signature::SeedDerivable, signer::keypair::Keypair};

	#[tokio::test]
	async fn test_mock_server_get_balance() {
		// Start mock server and get dynamic URL
		let mock_url = mock_server::async_run_test_only().await;
		let client = SolanaRpcClient::new(format!("{}/solana", mock_url).as_str());

		// has balance
		// address: AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9
		let seed = [1u8; 32];
		let keypair = Keypair::from_seed(&seed).expect("Failed to create keypair from seed");
		let balance = client.get_balance(&keypair.pubkey()).await.expect("Fail to get balance");
		assert_eq!(balance, 1000000000u64);

		// has no balance
		// address: 9hSR6S7WPtxmTojgo6GG3k4yDPecgJY292j7xrsUGWBu
		let seed = [2u8; 32];
		let keypair = Keypair::from_seed(&seed).expect("Failed to create keypair from seed");
		let balance = client.get_balance(&keypair.pubkey()).await.expect("Fail to get balance");
		assert_eq!(balance, 0u64);
	}
}
