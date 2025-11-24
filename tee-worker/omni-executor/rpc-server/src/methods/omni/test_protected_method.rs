use crate::server::RpcContext;
use crate::utils::omni::extract_omni_account;
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::RpcModule;
use oe_core::intent::executor::IntentExecutor;
use oe_primitives::utils::hex::hex_encode;

#[cfg(test)]
pub fn register_test_protected_method<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_method("omni_testProtectedMethod", |_, _, ext| {
			if let Ok(omni_account) = extract_omni_account(ext) {
				return Ok::<String, ErrorObjectOwned>(hex_encode(omni_account.as_ref()));
			}
			panic!("RpcExtensions not found in request extensions");
		})
		.expect("Failed to register test method");
}

#[cfg(test)]
mod test {
	use crate::{start_server, ShieldingKey};
	use chrono::{Days, Utc};
	use jsonrpsee::core::client::ClientT;
	use jsonrpsee::rpc_params;
	use jsonrpsee::ws_client::WsClientBuilder;
	use oe_client_binance::mocks::MockBinanceApiClient;
	use oe_client_pumpx::PumpxApiClient;
	use oe_client_signer::{mocks::MockSignerClient, SignerClient};
	use oe_client_wildmeta::{MockWildmetaApi, WildmetaApi};
	use oe_core::auth::{
		auth_token::{AuthOptions, AuthTokenClaims},
		constants::{AUTH_TOKEN_EXPIRATION_DAYS, AUTH_TOKEN_ID_TYPE, CLIENT_ID_HEIMA},
	};
	use oe_core::config::ConfigLoader;
	use oe_core::intent::executor::MockedIntentExecutor;
	use oe_crypto::jwt;
	use oe_primitives::{utils::hex::hex_encode, UserId};
	use oe_storage::{StorageDB, WildmetaTimestampStorage};
	use rsa::{pkcs1::EncodeRsaPrivateKey, rand_core::OsRng, RsaPrivateKey};
	use std::collections::HashMap;
	use std::sync::Arc;
	use tempfile::tempdir;

	#[tokio::test]
	pub async fn test_protected_method() {
		let tmp_dir = tempdir().unwrap();
		let port = 2004;
		let shielding_key = ShieldingKey::new();
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		let mut rng = OsRng;
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let jwt_private_key = rsa_private_key.to_pkcs1_der().unwrap();
		let pumpx_api = PumpxApiClient::new("https://api.pumpx.ai".to_string());
		let config_loader = ConfigLoader::from_env();
		let signer_client: Arc<Box<dyn SignerClient>> = Arc::new(Box::new(MockSignerClient::new()));
		let oe_client_binance_client: Arc<dyn oe_client_binance::BinancePaymasterApi> =
			Arc::new(MockBinanceApiClient::new());

		let wildmeta_api: Arc<Box<dyn WildmetaApi>> = Arc::new(Box::new(MockWildmetaApi));
		let wildmeta_timestamp_storage = Arc::new(WildmetaTimestampStorage::new(db.clone()));
		let loan_record_storage = Arc::new(oe_storage::LoanRecordStorage::new(db.clone()));

		let (cross_chain_intent_executor, _cross_chain_mock_recv) = MockedIntentExecutor::new();
		let aes_key = [0u8; 32];
		let entry_point_clients = HashMap::new();

		start_server(
			port,
			shielding_key.clone(),
			Arc::new(Box::new(pumpx_api)),
			db,
			jwt_private_key.as_bytes().to_vec(),
			&config_loader,
			signer_client,
			oe_client_binance_client,
			wildmeta_api,
			wildmeta_timestamp_storage,
			loan_record_storage,
			[0u8; 33], // Test ECDSA public key
			[0u8; 32], // Test bundler private key
			[0u8; 33], // Test bundler export authorized pubkey
			Arc::new(cross_chain_intent_executor),
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
		let omni_account =
			UserId::Email("test@test.com".into()).to_omni_account(CLIENT_ID_HEIMA).unwrap();

		let access_token_claims = AuthTokenClaims::new(
			hex_encode(omni_account.as_ref()),
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

		assert_eq!(response, hex_encode(omni_account.as_ref()));
	}
}
