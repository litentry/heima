#[cfg(feature = "mocks")]
pub mod mocks;

pub mod http_client;

use async_trait::async_trait;
use executor_crypto::aes256::AesOutput;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Copy, Serialize, Deserialize, Clone, PartialEq)]
pub enum ChainType {
	Evm,
	Solana,
	Tron,
}

#[async_trait]
pub trait SignerClient: Send + Sync {
	async fn request_wallet(
		&self,
		chain_type: ChainType,
		index: u32,
		omni_account: [u8; 32],
	) -> Result<Vec<u8>, ()>;

	async fn request_signature(
		&self,
		chain_type: ChainType,
		index: u32,
		omni_account: [u8; 32],
		message_to_sign: Vec<u8>,
	) -> Result<Vec<u8>, ()>;

	async fn request_signatures(
		&self,
		chain_type: ChainType,
		index: u32,
		omni_account: [u8; 32],
		messages_to_sign: Vec<Vec<u8>>,
	) -> Result<Vec<Vec<u8>>, ()>;

	async fn export_wallet(
		&self,
		chain_type: ChainType,
		index: u32,
		omni_account: [u8; 32],
		aes_key: Vec<u8>,
		wallet_address: String,
	) -> Result<AesOutput, ()>;
}
