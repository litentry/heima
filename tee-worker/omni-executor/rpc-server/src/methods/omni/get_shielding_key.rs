use crate::server::RpcContext;
use executor_core::intent_executor::IntentExecutor;
use executor_crypto::rsa::{traits::PublicKeyParts, Rsa3072PubKey, SerdeRsa3072PubKey};
use jsonrpsee::{types::ErrorObject, RpcModule};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};

// example output:
// {"jsonrpc":"2.0","id":1,"result":{"e":"0x010001","n":"0x0dee6b464b9a1033d0442be11ea013565e7cda30ff50bfafa9245e4d221e27a056be373251771cd5a949c6a29c1264eceaddf078d820d331f8ab6d9f4da167c55364ed8094cdc1ab467c2e68f2bcebe83453af3044f33cb4269428842cb354571fbc7c10aca61a06843a9d82c52231c18799dd8249d6e696f2c92695b40594b0b00d0b0fb77e7f7c9c89d2be0b9fd105d51d643094ebb2eb03185f056e75caf57df818ce6fca6ed104919296ba171caf950d2241c34257db7323e5c90fbbec813243ecbc41c0fbf6df2112c781d4f8540d4af356fbe999368c3404ebb7f806fdd94694d231704097be1da0ce895ce5ce69130a0d43432024cf3018e4621cfb370507d2ec1070ba65b6223de2d6a39dc997e3a3407302d7f0b456412d579f13ce57d8e4230d3c935956a526c77de48f73c73606f713aeb3ab9e8817aa53a48df1a961df9d262d88ee116fecfe4c4551b1fd704a23c25568f6448e3f22c53451061203cff70ec87780416dc02e4078a608359b20f6d129438bd5bce5bee16f24d3"}}
pub fn register_get_shielding_key<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<
			Header,
			RpcClient,
			RpcClientFactory,
			EthereumIntentExecutor,
			SolanaIntentExecutor,
			CrossChainIntentExecutor,
		>,
	>,
) {
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
	use config_loader::ConfigLoader;
	use executor_core::intent_executor::MockedIntentExecutor;
	use executor_storage::{StorageDB, WildmetaTimestampStorage};
	use jsonrpsee::core::client::ClientT;
	use jsonrpsee::rpc_params;
	use jsonrpsee::ws_client::WsClientBuilder;
	use native_task_handler::NativeTaskChannelType;
	use parentchain_rpc_client::metadata::SubxtMetadataProvider;
	use parentchain_rpc_client::{CustomConfig, SubxtClientFactory};
	use parentchain_signer::key_store::SubstrateKeyStore;
	use parentchain_signer::TxSigner;
	use pumpx::PumpxApiClient;
	use rsa::{pkcs1::EncodeRsaPrivateKey, RsaPrivateKey};
	use signer_client::{mocks::MockSignerClient, SignerClient};
	use std::collections::HashMap;
	use std::path::Path;
	use std::sync::Arc;
	use tempfile::tempdir;
	use tokio::sync::mpsc;
	use wildmeta_api::{MockWildmetaApi, WildmetaApi};

	#[tokio::test]
	pub async fn get_shielding_key_works() {
		let tmp_dir = tempdir().unwrap();
		let port: u16 = 2005;
		let shielding_key = ShieldingKey::new();
		let (sender, _) = mpsc::channel::<NativeTaskChannelType>(1);
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());
		let mut rng = rand::thread_rng();
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let jwt_private_key = rsa_private_key.to_pkcs1_der().unwrap();
		let pumpx_api = PumpxApiClient::new("https://api.pumpx.ai".to_string());
		let config_loader = ConfigLoader::from_env();
		let signer_client: Arc<Box<dyn SignerClient>> = Arc::new(Box::new(MockSignerClient::new()));

		let wildmeta_api: Arc<Box<dyn WildmetaApi>> = Arc::new(Box::new(MockWildmetaApi));
		let wildmeta_timestamp_storage = Arc::new(WildmetaTimestampStorage::new(db.clone()));

		let (solana_intent_executor, solana_mock_recv) = MockedIntentExecutor::new();
		let (ethereum_intent_executor, ethereum_mock_recv) = MockedIntentExecutor::new();
		let (cross_chain_intent_executor, cross_chain_mock_recv) = MockedIntentExecutor::new();

		let client_factory =
			SubxtClientFactory::<CustomConfig>::new(&config_loader.parentchain_url);
		let metadata_provider = Arc::new(SubxtMetadataProvider::new(client_factory.clone()));
		let parentchain_rpc_client_factory = Arc::new(client_factory);

		let aes_key = [0u8; 32];
		let entry_point_clients = HashMap::new();

		let substrate_key_store = Arc::new(SubstrateKeyStore::new(
			Path::new("./")
				.join("keystore/substrate_key.bin")
				.into_os_string()
				.into_string()
				.unwrap(),
		));

		let signer_account_nonce = 0;
		let parentchain_signer = parentchain_signer::get_signer(substrate_key_store.clone());

		let tx_signer = Arc::new(TxSigner::new(
			metadata_provider,
			parentchain_rpc_client_factory.clone(),
			parentchain_signer.clone(),
			signer_account_nonce,
		));

		start_server(
			port,
			shielding_key.clone(),
			Arc::new(Box::new(pumpx_api)),
			db,
			jwt_private_key.as_bytes().to_vec(),
			&config_loader,
			signer_client,
			wildmeta_api,
			wildmeta_timestamp_storage,
			Arc::new(ethereum_intent_executor),
			Arc::new(solana_intent_executor),
			Arc::new(cross_chain_intent_executor),
			parentchain_rpc_client_factory,
			aes_key,
			tx_signer,
			Arc::new(entry_point_clients),
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
