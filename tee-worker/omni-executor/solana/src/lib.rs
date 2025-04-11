pub mod signer;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
	commitment_config::CommitmentConfig, pubkey::Pubkey, signer::Signer as SignerTrait,
	system_instruction, transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;

pub struct SolanaClient {
	rpc_client: RpcClient,
}

impl SolanaClient {
	pub fn new(rpc_url: &str) -> Self {
		let client =
			RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
		Self { rpc_client: client }
	}

	pub async fn transfer_sol<Signer: SignerTrait>(
		&self,
		to: &str,
		value: u64,
		signer: &Signer,
	) -> Result<(), ()> {
		let block_hash = self
			.rpc_client
			.get_latest_blockhash()
			.await
			.map_err(|e| log::error!("Could not get block hash: {:?}", e))?;
		let to_pubkey =
			Pubkey::from_str(to).map_err(|e| log::error!("Could not parse to address: {:?}", e))?;
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
			.map_err(|e| log::error!("Could not send transaction: {:?}", e))?;

		log::debug!("Successfully transferred {} tokens from sender {}", value, signer.pubkey());
		log::debug!("Transaction signature: {:?}", tx_signature);

		Ok(())
	}

	pub async fn transfer_spl<Signer: SignerTrait>(
		&self,
		to: &str,
		value: u64,
		mint_address: &str,
		signer: &Signer,
	) -> Result<(), ()> {
		let block_hash = self
			.rpc_client
			.get_latest_blockhash()
			.await
			.map_err(|e| log::error!("Could not get block hash: {:?}", e))?;

		let mint_pubkey = Pubkey::from_str(mint_address)
			.map_err(|e| log::error!("Could not parse mint address: {:?}", e))?;
		let to_pubkey =
			Pubkey::from_str(to).map_err(|e| log::error!("Could not parse to address: {:?}", e))?;

		let source_pubkey = get_associated_token_address(&signer.pubkey(), &mint_pubkey);
		let destination_pubkey = get_associated_token_address(&to_pubkey, &mint_pubkey);

		let transfer_instruction = spl_token::instruction::transfer(
			&spl_token::id(),
			&source_pubkey,
			&destination_pubkey,
			&signer.pubkey(),
			&[&signer.pubkey()],
			value,
		)
		.map_err(|e| {
			log::error!("Could not create transfer instruction: {:?}", e);
		})?;
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
			.map_err(|e| log::error!("Could not send transaction: {:?}", e))?;

		log::debug!("Successfully transferred {} tokens from sender {}", value, signer.pubkey());
		log::debug!("Transaction signature: {:?}", tx_signature);

		Ok(())
	}
}
