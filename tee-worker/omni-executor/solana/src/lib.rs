mod signer;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
	commitment_config::CommitmentConfig, pubkey::Pubkey, signer::Signer as SignerTrait,
	system_instruction, transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;

pub struct SolanaClient<Signer: SignerTrait> {
	rpc_client: RpcClient,
	signer: Signer,
}

impl<Signer: SignerTrait> SolanaClient<Signer> {
	pub fn new(rpc_url: &str, signer: Signer) -> Result<Self, ()> {
		let client =
			RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
		Ok(Self { rpc_client: client, signer })
	}

	pub async fn transfer_sol(&self, to: [u8; 32], value: u64) -> Result<(), ()> {
		let block_hash = self
			.rpc_client
			.get_latest_blockhash()
			.await
			.map_err(|e| log::error!("Could not get block hash: {:?}", e))?;
		let to_pubkey = Pubkey::new_from_array(to);
		let transfer_instruction =
			system_instruction::transfer(&self.signer.pubkey(), &to_pubkey, value);
		let tx = Transaction::new_signed_with_payer(
			&[transfer_instruction],
			Some(&self.signer.pubkey()),
			&[&self.signer],
			block_hash,
		);
		self.rpc_client
			.send_and_confirm_transaction(&tx)
			.await
			.map_err(|e| log::error!("Could not send transaction: {:?}", e))?;
		Ok(())
	}

	pub async fn transfer_spl(
		&self,
		to: [u8; 32],
		value: u64,
		mint_address: &str,
	) -> Result<(), ()> {
		let block_hash = self
			.rpc_client
			.get_latest_blockhash()
			.await
			.map_err(|e| log::error!("Could not get block hash: {:?}", e))?;

		let mint_pubkey = Pubkey::from_str(mint_address)
			.map_err(|e| log::error!("Could not parse mint address: {:?}", e))?;
		let to_pubkey = Pubkey::new_from_array(to);

		let source_pubkey = get_associated_token_address(&self.signer.pubkey(), &mint_pubkey);
		let destination_pubkey = get_associated_token_address(&to_pubkey, &mint_pubkey);

		let transfer_instruction = spl_token::instruction::transfer(
			&spl_token::id(),
			&source_pubkey,
			&destination_pubkey,
			&self.signer.pubkey(),
			&[&self.signer.pubkey()],
			value,
		)
		.map_err(|e| {
			log::error!("Could not create transfer instruction: {:?}", e);
		})?;
		let tx = Transaction::new_signed_with_payer(
			&[transfer_instruction],
			Some(&self.signer.pubkey()),
			&[&self.signer],
			block_hash,
		);
		self.rpc_client
			.send_and_confirm_transaction(&tx)
			.await
			.map_err(|e| log::error!("Could not send transaction: {:?}", e))?;

		Ok(())
	}
}
