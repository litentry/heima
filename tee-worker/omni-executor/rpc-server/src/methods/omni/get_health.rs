// Copyright 2020-2024 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

use crate::server::RpcContext;
use executor_core::intent_executor::IntentExecutor;
use jsonrpsee::{types::ErrorObject, RpcModule};

pub fn register_get_health<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>,
	>,
) {
	module
		.register_method("omni_getHealth", |_, _, _| Ok::<String, ErrorObject>("OK".to_string()))
		.expect("Failed to register getHealth method");
}

#[cfg(test)]
mod test {
	use crate::{start_server, ShieldingKey};
	use binance_api::mocks::MockBinanceApiClient;
	use config_loader::ConfigLoader;
	use executor_core::intent_executor::MockedIntentExecutor;
	use executor_storage::{StorageDB, WildmetaTimestampStorage};
	use jsonrpsee::core::client::ClientT;
	use jsonrpsee::rpc_params;
	use jsonrpsee::ws_client::WsClientBuilder;
	use parentchain_signer::key_store::SubstrateKeyStore;
	use parentchain_signer::TxSigner;
	use pumpx::PumpxApiClient;
	use rsa::{pkcs1::EncodeRsaPrivateKey, RsaPrivateKey};
	use signer_client::{mocks::MockSignerClient, SignerClient};
	use std::collections::HashMap;
	use std::path::Path;
	use std::sync::Arc;
	use tempfile::tempdir;
	use wildmeta_api::{MockWildmetaApi, WildmetaApi};

	#[tokio::test]
	pub async fn get_health_works() {
		let tmp_dir = tempdir().unwrap();
		let port = 2004;
		let shielding_key = ShieldingKey::new();
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		let mut rng = rand::thread_rng();
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let jwt_private_key = rsa_private_key.to_pkcs1_der().unwrap();
		let pumpx_api = PumpxApiClient::new("https://api.pumpx.ai".to_string());
		let config_loader = ConfigLoader::from_env();
		let signer_client: Arc<Box<dyn SignerClient>> = Arc::new(Box::new(MockSignerClient::new()));
		let binance_api_client: Arc<dyn binance_api::BinancePaymasterApi> =
			Arc::new(MockBinanceApiClient::new());

		let wildmeta_api: Arc<Box<dyn WildmetaApi>> = Arc::new(Box::new(MockWildmetaApi));
		let wildmeta_timestamp_storage = Arc::new(WildmetaTimestampStorage::new(db.clone()));

		let (solana_intent_executor, _solana_mock_recv) = MockedIntentExecutor::new();
		let (ethereum_intent_executor, _ethereum_mock_recv) = MockedIntentExecutor::new();
		let (cross_chain_intent_executor, _cross_chain_mock_recv) = MockedIntentExecutor::new();

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
			binance_api_client,
			wildmeta_api,
			wildmeta_timestamp_storage,
			[0u8; 33], // Test ECDSA public key
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
		let response: String = client.request("omni_getHealth", rpc_params![]).await.unwrap();

		assert_eq!(response, "OK");
	}
}
