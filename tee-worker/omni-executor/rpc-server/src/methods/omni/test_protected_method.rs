use crate::methods::omni::common::check_auth;
use crate::server::RpcContext;
use executor_core::intent_executor::IntentExecutor;
use jsonrpsee::{types::ErrorObject, RpcModule};

#[cfg(test)]
pub fn register_test_protected_method<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<EthereumIntentExecutor, SolanaIntentExecutor>>,
) {
	module
		.register_method("omni_testProtectedMethod", |_, _, ext| {
			if let Ok(user) = check_auth(ext) {
				return Ok::<String, ErrorObject>(user.omni_account);
			}
			panic!("RpcExtensions not found in request extensions");
		})
		.expect("Failed to register test method");
}

#[cfg(test)]
mod test {
	use crate::{start_server, ShieldingKey};
	use binance_api::mocks::MockBinanceApiClient;
	use chrono::{Days, Utc};
	use config_loader::ConfigLoader;
	use executor_core::intent_executor::MockedIntentExecutor;
	use executor_crypto::jwt;
	use executor_primitives::utils::hex::ToHexPrefixed;
	use executor_storage::{StorageDB, WildmetaTimestampStorage};
	use heima_authentication::{
		auth_token::{AuthOptions, AuthTokenClaims},
		constants::{AUTH_TOKEN_EXPIRATION_DAYS, AUTH_TOKEN_ID_TYPE, CLIENT_ID_HEIMA},
	};
	use heima_primitives::{Identity, Web2IdentityType};
	use jsonrpsee::core::client::ClientT;
	use jsonrpsee::rpc_params;
	use jsonrpsee::ws_client::WsClientBuilder;
	use rsa::{pkcs1::EncodeRsaPrivateKey, RsaPrivateKey};
	use signer_client::{mocks::MockSignerClient, SignerClient};
	use std::collections::HashMap;
	use std::sync::Arc;
	use tempfile::tempdir;
	use wildmeta_api::{MockWildmetaApi, WildmetaApi};

	#[tokio::test]
	pub async fn test_protected_method() {
		let tmp_dir = tempdir().unwrap();
		let port = 2004;
		let shielding_key = ShieldingKey::new();
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		let mut rng = rand::thread_rng();
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let jwt_private_key = rsa_private_key.to_pkcs1_der().unwrap();
		let config_loader = ConfigLoader::from_env();
		let signer_client: Arc<Box<dyn SignerClient>> = Arc::new(Box::new(MockSignerClient::new()));
		let binance_api_client: Arc<dyn binance_api::BinancePaymasterApi> =
			Arc::new(MockBinanceApiClient::new());

		let wildmeta_api: Arc<Box<dyn WildmetaApi>> = Arc::new(Box::new(MockWildmetaApi));
		let wildmeta_timestamp_storage = Arc::new(WildmetaTimestampStorage::new(db.clone()));

		let (solana_intent_executor, _solana_mock_recv) = MockedIntentExecutor::new();
		let (ethereum_intent_executor, _ethereum_mock_recv) = MockedIntentExecutor::new();
		let aes_key = [0u8; 32];
		let entry_point_clients = HashMap::new();

		start_server(
			port,
			shielding_key.clone(),
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
			aes_key,
			Arc::new(entry_point_clients),
		)
		.await
		.unwrap();

		let url = format!("ws://127.0.0.1:{}", port);
		let mut headers = http::HeaderMap::new();

		let expires_at = Utc::now()
			.checked_add_days(Days::new(AUTH_TOKEN_EXPIRATION_DAYS))
			.expect("Failed to calculate expiration")
			.timestamp();
		let auth_options = AuthOptions { expires_at };
		let omni_account = Identity::from_web2_account("test@test.com", Web2IdentityType::Email)
			.to_omni_account(CLIENT_ID_HEIMA);

		let access_token_claims = AuthTokenClaims::new(
			omni_account.to_hex(),
			AUTH_TOKEN_ID_TYPE.to_string(),
			CLIENT_ID_HEIMA.to_string(),
			auth_options.clone(),
		);
		let token = jwt::create(&access_token_claims, jwt_private_key.as_bytes())
			.expect("Failed to create access token");

		headers.insert("Authorization", format!("Bearer {}", token).parse().unwrap());

		let client = WsClientBuilder::default().set_headers(headers).build(&url).await.unwrap();
		let response: String =
			client.request("omni_testProtectedMethod", rpc_params![]).await.unwrap();

		assert_eq!(response, omni_account.to_hex());
	}
}
