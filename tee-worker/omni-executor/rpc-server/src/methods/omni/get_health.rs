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
use jsonrpsee::{types::ErrorObject, RpcModule};

pub fn register_get_health(module: &mut RpcModule<RpcContext>) {
	module
		.register_method("omni_getHealth", |_, _, _| Ok::<String, ErrorObject>("OK".to_string()))
		.expect("Failed to register getHealth method");
}

#[cfg(test)]
mod test {
	use crate::{start_server, ShieldingKey};
	use config_loader::ConfigLoader;
	use executor_storage::{StorageDB, WildmetaTimestampStorage};
	use jsonrpsee::core::client::ClientT;
	use jsonrpsee::rpc_params;
	use jsonrpsee::ws_client::WsClientBuilder;
	use native_task_handler::NativeTaskChannelType;
	use pumpx::PumpxApiClient;
	use rsa::{pkcs1::EncodeRsaPrivateKey, RsaPrivateKey};
	use signer_client::{mocks::MockSignerClient, SignerClient};
	use std::sync::Arc;
	use tempfile::tempdir;
	use tokio::sync::mpsc;
	use wildmeta_api::{MockWildmetaApi, WildmetaApi};

	#[tokio::test]
	pub async fn get_health_works() {
		let tmp_dir = tempdir().unwrap();
		let port = 2004;
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

		// Create empty rpc_clients for test
		let rpc_clients = Arc::new(std::collections::HashMap::new());

		let wildmeta_api: Arc<Box<dyn WildmetaApi>> = Arc::new(Box::new(MockWildmetaApi));
		let wildmeta_timestamp_storage = Arc::new(WildmetaTimestampStorage::new(db.clone()));

		start_server(
			port,
			shielding_key.clone(),
			Arc::new(sender),
			Arc::new(Box::new(pumpx_api)),
			db,
			jwt_private_key.as_bytes().to_vec(),
			&config_loader,
			rpc_clients,
			signer_client,
			wildmeta_api,
			wildmeta_timestamp_storage,
		)
		.await
		.unwrap();

		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();
		let response: String = client.request("omni_getHealth", rpc_params![]).await.unwrap();

		assert_eq!(response, "OK");
	}
}
