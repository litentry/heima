use async_trait::async_trait;
use executor_crypto::ecdsa;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::core::params::ArrayParams;
use jsonrpsee::core::traits::ToRpcParams;
use jsonrpsee::http_client::HttpClient;
use jsonrpsee::server::tracing::error;
use rand::rngs::OsRng;
use rsa::BigUint;
use rsa::Oaep;
use rsa::RsaPublicKey;
use serde::Deserialize;
use serde::Serialize;
use serde_json::value::to_raw_value;
use serde_with::serde_as;
use sha2::Sha256;
use sp_core::keccak_256;

use crate::constants::*;

pub const NONCE_LEN: usize = 96 / 8;

// we use 256-bit AES-GCM key
pub const AES_KEY_LEN: usize = 32;
pub type Aes256Key = [u8; AES_KEY_LEN];
pub type Aes256KeyNonce = [u8; NONCE_LEN];

// TODO: https://linear.app/litentry/issue/P-1451/use-consistent-0x-prefix-for-hex-string
//       the change needs to be in sync with signer

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct SignedParams<P> {
	pub payload: P,
	#[serde_as(as = "serde_with::hex::Hex")]
	pub signature: [u8; 65],
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct SignWalletPayload {
	#[serde(flatten)]
	pub wallet: Wallet,
	#[serde_as(as = "serde_with::hex::Hex")]
	pub msg: Vec<u8>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct MultiSignWalletPayload {
	#[serde(flatten)]
	pub wallet: Wallet,
	#[serde_as(as = "Vec<serde_with::hex::Hex>")]
	pub msgs: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetWalletPayload {
	#[serde(flatten)]
	pub wallet: Wallet,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct Wallet {
	pub chain_type: ChainType,
	pub index: u32,
	#[serde_as(as = "serde_with::hex::Hex")]
	pub omni_account: [u8; 32],
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportWalletPayload {
	#[serde(flatten)]
	pub wallet: Wallet,
	#[serde_as(as = "serde_with::hex::Hex")]
	pub key: Vec<u8>, // RSA-encrypted AES key to encrypt the wallet private key
	pub wallet_address: String,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct ShieldingKey {
	#[serde_as(as = "serde_with::hex::Hex")]
	pub n: Vec<u8>,
	#[serde_as(as = "serde_with::hex::Hex")]
	pub e: Vec<u8>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Default, Debug)]
struct AesOutput {
	#[serde_as(as = "serde_with::hex::Hex")]
	pub ciphertext: Vec<u8>,
	#[serde_as(as = "serde_with::hex::Hex")]
	pub aad: Vec<u8>,
	#[serde_as(as = "serde_with::hex::Hex")]
	pub nonce: Aes256KeyNonce,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChainType {
	Evm,
	Solana,
	Tron,
}

impl ChainType {
	pub fn from_pumpx_chain_id(id: u32) -> Option<Self> {
		match id {
			SOLANA_CHAIN_ID => Some(Self::Solana),
			ETHEREUM_CHAIN_ID | BSC_CHAIN_ID | BASE_CHAIN_ID => Some(Self::Evm),
			_ => None,
		}
	}
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
	) -> Result<executor_crypto::aes256::AesOutput, ()>;
}

pub struct PumpxSignerClient {
	url: String,
	request_signer: ecdsa::Pair,
}

impl PumpxSignerClient {
	pub fn new(url: String, request_signer: ecdsa::Pair) -> Self {
		Self { url, request_signer }
	}

	async fn get_shielding_key(&self) -> Result<ShieldingKey, ()> {
		let client = HttpClient::builder()
			.build(&self.url)
			.map_err(|e| error!("Could not create client: {:?}", e))?;

		let shielding_key: ShieldingKey = client
			.request("dex_getShieldingKey", ArrayParams::default())
			.await
			.map_err(|e| error!("Could not get shielding key from signer: {:?}", e))?;
		Ok(shielding_key)
	}
}

#[async_trait]
impl SignerClient for PumpxSignerClient {
	async fn request_wallet(
		&self,
		chain_type: ChainType,
		index: u32,
		omni_account: [u8; 32],
	) -> Result<Vec<u8>, ()> {
		let client = HttpClient::builder()
			.build(&self.url)
			.map_err(|e| error!("Could not create client: {:?}", e))?;
		let wallet = Wallet { chain_type, index, omni_account };
		let get_wallet = GetWalletPayload { wallet };
		let serialized_get_wallet = serde_json::to_vec(&get_wallet)
			.map_err(|e| error!("Could not serialize dex_getWallet request: {:?}", e))?;
		let signature = self.request_signer.sign_prehashed(&keccak_256(&serialized_get_wallet)).0;
		let signed: SignedParams<GetWalletPayload> =
			SignedParams { payload: get_wallet, signature };
		let hex_encoded: String = client
			.request("dex_getWallet", signed)
			.await
			.map_err(|e| error!("Could not get wallet: {:?}", e))?;
		hex::decode(hex_encoded).map_err(|e| error!("Could not decode wallet: {:?}", e))
	}

	async fn request_signature(
		&self,
		chain_type: ChainType,
		index: u32,
		omni_account: [u8; 32],
		message_to_sign: Vec<u8>,
	) -> Result<Vec<u8>, ()> {
		let client = HttpClient::builder()
			.build(&self.url)
			.map_err(|e| error!("Could not create client: {:?}", e))?;
		let wallet = Wallet { chain_type, index, omni_account };
		let sign_wallet = SignWalletPayload { wallet, msg: message_to_sign };
		let serialized_sign_wallet = serde_json::to_vec(&sign_wallet)
			.map_err(|e| error!("Could not serialize dex_signWallet request: {:?}", e))?;
		let signature = self.request_signer.sign_prehashed(&keccak_256(&serialized_sign_wallet)).0;
		let signed: SignedParams<SignWalletPayload> =
			SignedParams { payload: sign_wallet, signature };
		let hex_encoded: String = client
			.request("dex_signWallet", signed)
			.await
			.map_err(|e| println!("Could not sign wallet: {:?}", e))?;
		hex::decode(hex_encoded).map_err(|e| error!("Could not decode signature: {:?}", e))
	}

	async fn request_signatures(
		&self,
		chain_type: ChainType,
		index: u32,
		omni_account: [u8; 32],
		messages_to_sign: Vec<Vec<u8>>,
	) -> Result<Vec<Vec<u8>>, ()> {
		let client = HttpClient::builder()
			.build(&self.url)
			.map_err(|e| error!("Could not create client: {:?}", e))?;
		let wallet = Wallet { chain_type, index, omni_account };
		let multi_sign_wallet = MultiSignWalletPayload { wallet, msgs: messages_to_sign };
		let serialized_multi_sign_wallet = serde_json::to_vec(&multi_sign_wallet)
			.map_err(|e| error!("Could not serialize dex_multiSignWallet request: {:?}", e))?;
		let signature =
			self.request_signer.sign_prehashed(&keccak_256(&serialized_multi_sign_wallet)).0;
		let signed: SignedParams<MultiSignWalletPayload> =
			SignedParams { payload: multi_sign_wallet, signature };
		let hex_encoded: Vec<String> = client
			.request("dex_multiSignWallet", signed)
			.await
			.map_err(|e| println!("Could not multi sign wallet: {:?}", e))?;

		let mut decoded_signatures = vec![];

		for encoded in hex_encoded {
			decoded_signatures.push(
				hex::decode(encoded).map_err(|e| error!("Could not decode signature: {:?}", e))?,
			);
		}
		Ok(decoded_signatures)
	}

	async fn export_wallet(
		&self,
		chain_type: ChainType,
		index: u32,
		omni_account: [u8; 32],
		aes_key: Vec<u8>,
		wallet_address: String,
	) -> Result<executor_crypto::aes256::AesOutput, ()> {
		let client = HttpClient::builder()
			.build(&self.url)
			.map_err(|e| error!("Could not create client: {:?}", e))?;
		let wallet = Wallet { chain_type, index, omni_account };

		let shielding_key = self.get_shielding_key().await?;

		let rsa_shielding_key = RsaPublicKey::new(
			BigUint::from_bytes_le(&shielding_key.n),
			BigUint::from_bytes_le(&shielding_key.e),
		)
		.map_err(|e| error!("Could not create shielding key: {:?}", e))?;

		let encrypted_aes_key =
			rsa_shielding_key.encrypt(&mut OsRng, Oaep::new::<Sha256>(), &aes_key).map_err(
				|e| error!("Could not encrypt aes key with pumpx signer shielding key: {:?}", e),
			)?;

		let export_wallet = ExportWalletPayload { wallet, wallet_address, key: encrypted_aes_key };
		let serialized_export_wallet = serde_json::to_vec(&export_wallet)
			.map_err(|e| error!("Could not serialize dex_signWallet request: {:?}", e))?;
		let signature =
			self.request_signer.sign_prehashed(&keccak_256(&serialized_export_wallet)).0;
		let signed: SignedParams<ExportWalletPayload> =
			SignedParams { payload: export_wallet, signature };
		let output: AesOutput = client
			.request("dex_exportWallet", signed)
			.await
			.map_err(|e| println!("Could not export wallet: {:?}", e))?;
		Ok(executor_crypto::aes256::AesOutput {
			aad: output.aad,
			ciphertext: output.ciphertext,
			nonce: output.nonce,
		})
	}
}

impl<P: Serialize> ToRpcParams for SignedParams<P> {
	fn to_rpc_params(self) -> Result<Option<Box<serde_json::value::RawValue>>, serde_json::Error> {
		Ok(Some(to_raw_value(&self)?))
	}
}

#[cfg(test)]
pub mod tests {

	use jsonrpsee::tokio;
	use sp_core::{ecdsa, Pair};

	use crate::signer_client::SignerClient;
	use crate::signer_client::{ChainType, PumpxSignerClient};

	#[ignore = "manual"]
	#[tokio::test]
	pub async fn test_request_wallet() {
		//dev auth key
		let pair = ecdsa::Pair::from_seed(
			&hex::decode("cb6df9de1efca7a3998a8ead4e02159d5fa99c3e0d4fd6432667390bb4726854")
				.unwrap()
				.try_into()
				.unwrap(),
		);
		let client = PumpxSignerClient::new("http://localhost:2000".to_string(), pair);

		let wallet = client.request_wallet(ChainType::Evm, 0, [0u8; 32]).await;
		println!("Got wallet: {:?}", wallet);
	}

	#[ignore = "manual"]
	#[tokio::test]
	pub async fn test_request_signature() {
		//dev auth key
		let pair = ecdsa::Pair::from_seed(
			&hex::decode("cb6df9de1efca7a3998a8ead4e02159d5fa99c3e0d4fd6432667390bb4726854")
				.unwrap()
				.try_into()
				.unwrap(),
		);
		let client = PumpxSignerClient::new("http://localhost:2000".to_string(), pair);
		let signature =
			client.request_signature(ChainType::Evm, 0, [0u8; 32], [0u8; 32].to_vec()).await;
		println!("Got signature: {:?}", signature);
	}

	#[ignore = "manual"]
	#[tokio::test]
	pub async fn test_get_shielding_key() {
		//dev auth key
		let pair = ecdsa::Pair::from_seed(
			&hex::decode("cb6df9de1efca7a3998a8ead4e02159d5fa99c3e0d4fd6432667390bb4726854")
				.unwrap()
				.try_into()
				.unwrap(),
		);
		let client = PumpxSignerClient::new("http://localhost:2000".to_string(), pair);
		let shielding_key = client.get_shielding_key().await.unwrap();
		println!("Got shielding key: {:?}", shielding_key);
	}

	#[ignore = "manual"]
	#[tokio::test]
	pub async fn test_export_wallet() {
		//dev auth key
		let pair = ecdsa::Pair::from_seed(
			&hex::decode("cb6df9de1efca7a3998a8ead4e02159d5fa99c3e0d4fd6432667390bb4726854")
				.unwrap()
				.try_into()
				.unwrap(),
		);
		let client = PumpxSignerClient::new("http://localhost:2000".to_string(), pair);

		let omni_account =
			hex::decode("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d")
				.unwrap();

		let wallet = client
			.export_wallet(
				ChainType::Solana,
				0,
				omni_account.try_into().unwrap(),
				[0u8; 32].to_vec(),
				"A2KsRiF1CAR7dN5X5ZwcS97PuuXWjP35X2UxTffJ6sbn".to_string(),
			)
			.await
			.unwrap();
		println!("Got wallet: {:?}", wallet);
	}
}
