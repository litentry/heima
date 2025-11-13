use crate::ChainType;
use crate::SignerClient as SignerClientTrait;
use async_trait::async_trait;
use mockall::mock;

mock! {
	pub SignerClient {}

	#[async_trait]
	impl SignerClientTrait for SignerClient {

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
		) -> Result<executor_crypto::aes256::AesOutput, ()>;
	}
}
