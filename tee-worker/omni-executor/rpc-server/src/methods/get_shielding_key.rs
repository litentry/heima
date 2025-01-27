use crate::{server::RpcContext, utils::hex::ToHexPrefixed};
use executor_crypto::rsa::traits::PublicKeyParts;
use jsonrpsee::{
	types::{ErrorCode, ErrorObject},
	RpcModule,
};
use serde::{Deserialize, Serialize};
use std::vec::Vec;

#[derive(Serialize, Deserialize)]
struct Rsa3072PubKey {
	n: Vec<u8>,
	e: Vec<u8>,
}

pub fn register_get_shielding_key(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("native_getShieldingKey", |_params, ctx, _| async move {
			let public_key = ctx.shielding_key.public_key();
			let public_key_json = serde_json::to_string(&Rsa3072PubKey {
				n: public_key.n().to_bytes_le(),
				e: public_key.e().to_bytes_le(),
			})
			.map_err(|_| ErrorCode::InternalError)?;

			Ok::<String, ErrorObject>(public_key_json.to_hex())
		})
		.expect("Failed to register native_getShieldingKey method");
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{start_server, utils::hex::FromHexPrefixed, ShieldingKey};
	use jsonrpsee::core::client::ClientT;
	use jsonrpsee::rpc_params;
	use jsonrpsee::ws_client::WsClientBuilder;
	use native_task_handler::NativeTask;
	use std::sync::Arc;
	use tokio::sync::mpsc;

	#[tokio::test]
	pub async fn get_shielding_key_works() {
		let port = "2000";
		let shielding_key = ShieldingKey::new();
		let (sender, _) = mpsc::channel::<NativeTask>(1);

		start_server(port, shielding_key.clone(), Arc::new(sender), [0u8; 32])
			.await
			.unwrap();

		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();
		let response: String =
			client.request("native_getShieldingKey", rpc_params![]).await.unwrap();
		let decoded_json = String::from_hex(&response).unwrap();
		let pubkey: Rsa3072PubKey = serde_json::from_str(&decoded_json).unwrap();

		assert_eq!(pubkey.n, shielding_key.public_key().n().to_bytes_le());
		assert_eq!(pubkey.e, shielding_key.public_key().e().to_bytes_le());
	}
}
