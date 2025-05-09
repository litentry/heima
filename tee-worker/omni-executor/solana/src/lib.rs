pub mod signer;

use async_trait::async_trait;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
	commitment_config::CommitmentConfig, pubkey::Pubkey, signer::Signer as SignerTrait,
	system_instruction, transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;

#[async_trait]
pub trait SolanaClient: Send + Sync {
	async fn transfer_sol(
		&self,
		to: &str,
		value: u64,
		signer: &(dyn SignerTrait + Send + Sync),
	) -> Result<String, ()>;

	async fn transfer_spl(
		&self,
		to: &str,
		value: u64,
		mint_address: &str,
		signer: &(dyn SignerTrait + Send + Sync),
	) -> Result<String, ()>;
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
	async fn transfer_sol(
		&self,
		to: &str,
		value: u64,
		signer: &(dyn SignerTrait + Send + Sync),
	) -> Result<String, ()> {
		let block_hash = self
			.rpc_client
			.get_latest_blockhash()
			.await
			.map_err(|e| tracing::log::error!("Could not get block hash: {:?}", e))?;
		let to_pubkey = Pubkey::from_str(to)
			.map_err(|e| tracing::log::error!("Could not parse to address: {:?}", e))?;
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
			.map_err(|e| tracing::log::error!("Could not send transaction: {:?}", e))?;

		tracing::log::debug!(
			"Successfully transferred {} tokens from sender {} to {}",
			value,
			signer.pubkey(),
			to_pubkey
		);
		tracing::log::debug!("Transaction signature: {:?}", tx_signature);

		Ok(tx_signature.to_string())
	}

	async fn transfer_spl(
		&self,
		to: &str,
		value: u64,
		mint_address: &str,
		signer: &(dyn SignerTrait + Send + Sync),
	) -> Result<String, ()> {
		let mint_pubkey = Pubkey::from_str(mint_address)
			.map_err(|e| tracing::log::error!("Could not parse mint address: {:?}", e))?;
		let to_pubkey = Pubkey::from_str(to)
			.map_err(|e| tracing::log::error!("Could not parse to address: {:?}", e))?;

		let source_pubkey = get_associated_token_address(&signer.pubkey(), &mint_pubkey);
		let destination_pubkey = get_associated_token_address(&to_pubkey, &mint_pubkey);

		let destination_exists = self.rpc_client.get_account(&destination_pubkey).await.is_ok();

		// if dest ATA doesn't exist, send one tx to create it, two ix in one tx didn't seem to work
		// this should happen only once as the dest ATA would be initialized after one interaction
		if !destination_exists {
			tracing::log::debug!(
				"Destination ATA {} doesn't exist, creating it now",
				destination_pubkey
			);
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
				.map_err(|e| tracing::log::error!("Could not get block hash: {:?}", e))?;

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
				.map_err(|e| tracing::log::error!("Could not send transaction: {:?}", e))?;

			tracing::log::debug!(
				"Successfully created destination ATA, tx signature {:?}",
				tx_signature
			);
		}

		let block_hash = self
			.rpc_client
			.get_latest_blockhash()
			.await
			.map_err(|e| tracing::log::error!("Could not get block hash: {:?}", e))?;

		let transfer_ix = spl_token::instruction::transfer(
			&spl_token::id(),
			&source_pubkey,
			&destination_pubkey,
			&signer.pubkey(),
			&[&signer.pubkey()],
			value,
		)
		.map_err(|e| {
			tracing::log::error!("Could not create transfer instruction: {:?}", e);
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
			.map_err(|e| tracing::log::error!("Could not send transaction: {:?}", e))?;

		tracing::log::debug!(
			"Successfully transferred {} tokens from sender {} to {}",
			value,
			signer.pubkey(),
			to_pubkey
		);
		tracing::log::debug!("Transaction signature: {:?}", tx_signature);

		Ok(tx_signature.to_string())
	}
}
