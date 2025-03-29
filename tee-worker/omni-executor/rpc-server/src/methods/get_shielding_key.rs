use crate::server::RpcContext;
use executor_crypto::rsa::{traits::PublicKeyParts, Rsa3072PubKey};
use executor_primitives::utils::hex::ToHexPrefixed;
use jsonrpsee::{
	types::{ErrorCode, ErrorObject},
	RpcModule,
};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};

pub fn register_get_shielding_key<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<Header, RpcClient, RpcClientFactory>>,
) {
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
	use crate::{start_server, ShieldingKey};
	use executor_primitives::utils::hex::FromHexPrefixed;
	use executor_storage::StorageDB;
	use jsonrpsee::core::client::ClientT;
	use jsonrpsee::rpc_params;
	use jsonrpsee::ws_client::WsClientBuilder;
	use native_task_handler::NativeTaskChannelType;
	use parentchain_rpc_client::{CustomConfig, SubxtClientFactory};
	use rsa::{pkcs1::EncodeRsaPrivateKey, RsaPrivateKey};
	use std::sync::Arc;
	use tokio::sync::mpsc;

	#[tokio::test]
	pub async fn get_shielding_key_works() {
		let port: u16 = 2000;
		let shielding_key = ShieldingKey::new();
		let (sender, _) = mpsc::channel::<NativeTaskChannelType>(1);
		let client_factory = SubxtClientFactory::<CustomConfig>::new("ws://localhost:9944");
		let db = StorageDB::open_default("test_storage_db").unwrap();
		let mut rng = rand::thread_rng();
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let jwt_private_key = rsa_private_key.to_pkcs1_der().unwrap();

		start_server(
			port,
			Arc::new(client_factory),
			shielding_key.clone(),
			Arc::new(sender),
			Arc::new(db),
			[0u8; 32],
			jwt_private_key.as_bytes().to_vec(),
		)
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
