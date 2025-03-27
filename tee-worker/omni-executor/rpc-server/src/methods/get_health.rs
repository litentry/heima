use crate::server::RpcContext;
use jsonrpsee::{types::ErrorObject, RpcModule};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};

pub fn register_get_health<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<Header, RpcClient, RpcClientFactory>>,
) {
	module
		.register_method("get_health", |_, _, _| Ok::<String, ErrorObject>("OK".to_string()))
		.expect("Failed to register getHealth method");
}

#[cfg(test)]
mod test {
	use crate::{start_server, ShieldingKey};
	use executor_storage::StorageDB;
	use jsonrpsee::core::client::ClientT;
	use jsonrpsee::rpc_params;
	use jsonrpsee::ws_client::WsClientBuilder;
	use native_task_handler::NativeTask;
	use parentchain_rpc_client::{CustomConfig, SubxtClientFactory};
	use rsa::{pkcs1::EncodeRsaPrivateKey, RsaPrivateKey};
	use std::sync::Arc;
	use tokio::sync::mpsc;

	#[tokio::test]
	pub async fn get_health_works() {
		let port = 2000;
		let shielding_key = ShieldingKey::new();
		let (sender, _) = mpsc::channel::<NativeTask>(1);
		let client_factory = SubxtClientFactory::<CustomConfig>::new("ws://localhost:9944");
		let db = StorageDB::open_default("test_get_health_storage_db").unwrap();

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
		let response: String = client.request("get_health", rpc_params![]).await.unwrap();

		assert_eq!(response, "OK");
	}
}
