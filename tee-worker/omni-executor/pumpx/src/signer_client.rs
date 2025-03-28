use executor_crypto::ecdsa;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::core::traits::ToRpcParams;
use jsonrpsee::http_client::HttpClient;
use jsonrpsee::server::tracing::error;
use serde::Deserialize;
use serde::Serialize;
use serde_json::value::to_raw_value;
use serde_with::serde_as;
use sp_core::keccak_256;

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

#[derive(Debug, Serialize, Deserialize)]
pub enum ChainType {
	Evm,
	Solana,
	Tron,
}

pub struct SignerClient {
	url: String,
	request_signer: ecdsa::Pair,
}

impl SignerClient {
	pub fn new(url: String, request_signer: ecdsa::Pair) -> Self {
		Self { url, request_signer }
	}

	pub async fn request_wallet(&self) -> Result<Vec<u8>, ()> {
		let client = HttpClient::builder().build(&self.url).unwrap();
		let wallet = Wallet { chain_type: ChainType::Evm, index: 0, omni_account: [0; 32] };
		let get_wallet = GetWalletPayload { wallet };
		let signature = self
			.request_signer
			.sign_prehashed(&keccak_256(&serde_json::to_vec(&get_wallet).unwrap()))
			.0;
		let signed: SignedParams<GetWalletPayload> =
			SignedParams { payload: get_wallet, signature };
		let hex_encoded: String = client
			.request("dex_getWallet", signed)
			.await
			.map_err(|e| error!("Could not get wallet: {:?}", e))?;
		hex::decode(hex_encoded).map_err(|e| error!("Could not decode wallet: {:?}", e))
	}

	pub async fn request_signature(&self, message_to_sign: &[u8]) -> Result<Vec<u8>, ()> {
		let client = HttpClient::builder().build(&self.url).unwrap();
		let wallet = Wallet { chain_type: ChainType::Evm, index: 0, omni_account: [0; 32] };
		let sign_wallet = SignWalletPayload { wallet, msg: message_to_sign.to_vec() };
		let signature = self
			.request_signer
			.sign_prehashed(&keccak_256(&serde_json::to_vec(&sign_wallet).unwrap()))
			.0;
		let signed: SignedParams<SignWalletPayload> =
			SignedParams { payload: sign_wallet, signature };
		let hex_encoded: String = client
			.request("dex_signWallet", signed)
			.await
			.map_err(|e| println!("Could not sign wallet: {:?}", e))?;
		hex::decode(hex_encoded).map_err(|e| error!("Could not decode signature: {:?}", e))
	}
}

impl<P: Serialize> ToRpcParams for SignedParams<P> {
	fn to_rpc_params(self) -> Result<Option<Box<serde_json::value::RawValue>>, serde_json::Error> {
		Ok(Some(to_raw_value(&self).unwrap()))
	}
}

#[cfg(test)]
pub mod tests {
	use jsonrpsee::tokio;
	use sp_core::{ecdsa, Pair};

	use crate::signer_client::SignerClient;

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
		let client = SignerClient::new("http://localhost:2000".to_string(), pair);

		let wallet = client.request_wallet().await;
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
		let client = SignerClient::new("http://localhost:2000".to_string(), pair);
		let signature = client.request_signature(&[0u8; 32]).await;
		println!("Got signature: {:?}", signature);
	}
}
