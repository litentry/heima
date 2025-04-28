use crate::server::RpcContext;
use executor_crypto::rsa::{traits::PublicKeyParts, Rsa3072PubKey, SerdeRsa3072PubKey};
use jsonrpsee::{types::ErrorObject, RpcModule};

// example output:
// {"jsonrpc":"2.0","id":1,"result":{"e":"0x010001","n":"0x0dee6b464b9a1033d0442be11ea013565e7cda30ff50bfafa9245e4d221e27a056be373251771cd5a949c6a29c1264eceaddf078d820d331f8ab6d9f4da167c55364ed8094cdc1ab467c2e68f2bcebe83453af3044f33cb4269428842cb354571fbc7c10aca61a06843a9d82c52231c18799dd8249d6e696f2c92695b40594b0b00d0b0fb77e7f7c9c89d2be0b9fd105d51d643094ebb2eb03185f056e75caf57df818ce6fca6ed104919296ba171caf950d2241c34257db7323e5c90fbbec813243ecbc41c0fbf6df2112c781d4f8540d4af356fbe999368c3404ebb7f806fdd94694d231704097be1da0ce895ce5ce69130a0d43432024cf3018e4621cfb370507d2ec1070ba65b6223de2d6a39dc997e3a3407302d7f0b456412d579f13ce57d8e4230d3c935956a526c77de48f73c73606f713aeb3ab9e8817aa53a48df1a961df9d262d88ee116fecfe4c4551b1fd704a23c25568f6448e3f22c53451061203cff70ec87780416dc02e4078a608359b20f6d129438bd5bce5bee16f24d3"}}
pub fn register_get_shielding_key(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_getShieldingKey", |_params, ctx, _| async move {
			let pubkey = ctx.shielding_key.public_key();
			let pubkey =
				Rsa3072PubKey { n: pubkey.n().to_bytes_le(), e: pubkey.e().to_bytes_le() }.into();
			Ok::<SerdeRsa3072PubKey, ErrorObject>(pubkey)
		})
		.expect("Failed to register omni_getShieldingKey method");
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{start_server, ShieldingKey};
	use executor_storage::StorageDB;
	use jsonrpsee::core::client::ClientT;
	use jsonrpsee::rpc_params;
	use jsonrpsee::ws_client::WsClientBuilder;
	use native_task_handler::NativeTaskChannelType;
	use pumpx::PumpxApiClient;
	use rsa::{pkcs1::EncodeRsaPrivateKey, RsaPrivateKey};
	use std::sync::Arc;
	use tempfile::tempdir;
	use tokio::sync::mpsc;

	#[tokio::test]
	pub async fn get_shielding_key_works() {
		let tmp_dir = tempdir().unwrap();
		let port: u16 = 2005;
		let shielding_key = ShieldingKey::new();
		let (sender, _) = mpsc::channel::<NativeTaskChannelType>(1);
		let db = StorageDB::open_default(tmp_dir.path()).unwrap();
		let mut rng = rand::thread_rng();
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let jwt_private_key = rsa_private_key.to_pkcs1_der().unwrap();
		let pumpx_api = PumpxApiClient::new(None);

		start_server(
			port,
			shielding_key.clone(),
			Arc::new(sender),
			Arc::new(Box::new(pumpx_api)),
			Arc::new(db),
			[0u8; 32],
			jwt_private_key.as_bytes().to_vec(),
		)
		.await
		.unwrap();

		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();
		let response: serde_json::Value =
			client.request("omni_getShieldingKey", rpc_params![]).await.unwrap();
		let pubkey: SerdeRsa3072PubKey = serde_json::from_value(response).unwrap();

		assert_eq!(pubkey.n, shielding_key.public_key().n().to_bytes_le());
		assert_eq!(pubkey.e, shielding_key.public_key().e().to_bytes_le());
	}
}
