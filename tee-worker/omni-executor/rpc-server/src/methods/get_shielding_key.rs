use crate::server::RpcContext;
use jsonrpsee::RpcModule;
use parity_scale_codec::Encode;
use rsa::traits::PublicKeyParts;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

#[derive(Serialize, Deserialize)]
struct Rsa3072PubKey {
	n: Vec<u8>,
	e: Vec<u8>,
}

pub fn register_get_shielding_key(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("author_getShieldingKey", |_params, ctx, _| async move {
			let public_key = ctx.shielding_key.public_key();
			let public_key_json = serde_json::to_string(&Rsa3072PubKey {
				n: public_key.n().to_bytes_le(),
				e: public_key.e().to_bytes_le(),
			})
			.expect("Failed to serialize public key");

			hex::encode(public_key_json.encode())
		})
		.expect("Failed to register author_getShieldingKey method");
}

#[cfg(test)]
mod test {
	use super::Rsa3072PubKey;
	use crate::server::start_server;
	use crate::ShieldingKey;
	use jsonrpsee::core::client::ClientT;
	use jsonrpsee::rpc_params;
	use jsonrpsee::ws_client::WsClientBuilder;
	use parity_scale_codec::Decode;
	use rsa::traits::PublicKeyParts;
	use std::sync::Arc;

	#[tokio::test]
	pub async fn get_shielding_key_works() {
		let port = "2000";
		let shielding_key = Arc::new(ShieldingKey::new());
		start_server(port, shielding_key.clone()).await.unwrap();

		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();
		let response: String =
			client.request("author_getShieldingKey", rpc_params![]).await.unwrap();
		let scale_encoded_json = hex::decode(response).unwrap();
		let decoded_json = String::decode(&mut scale_encoded_json.as_slice()).unwrap();
		let pubkey: Rsa3072PubKey = serde_json::from_str(&decoded_json).unwrap();

		assert_eq!(pubkey.n, shielding_key.public_key().n().to_bytes_le());
		assert_eq!(pubkey.e, shielding_key.public_key().e().to_bytes_le());
	}
}
