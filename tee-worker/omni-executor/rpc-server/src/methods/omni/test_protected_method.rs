use crate::methods::omni::common::check_auth;
use crate::server::RpcContext;
use jsonrpsee::{types::ErrorObject, RpcModule};

#[cfg(test)]
pub fn register_test_protected_method(module: &mut RpcModule<RpcContext>) {
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
	use chrono::{Days, Utc};
	use config_loader::ConfigLoader;
	use executor_crypto::jwt;
	use executor_primitives::utils::hex::ToHexPrefixed;
	use executor_storage::StorageDB;
	use heima_authentication::{
		auth_token::{AuthOptions, AuthTokenClaims},
		constants::{AUTH_TOKEN_EXPIRATION_DAYS, AUTH_TOKEN_ID_TYPE, CLIENT_ID_HEIMA},
	};
	use heima_identity_verification::web2::email::{mailer::MailerTrait, ConsoleMailer};
	use heima_primitives::{Identity, Web2IdentityType};
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

	#[tokio::test]
	pub async fn test_protected_method() {
		let tmp_dir = tempdir().unwrap();
		let port = 2004;
		let shielding_key = ShieldingKey::new();
		let (sender, _) = mpsc::channel::<NativeTaskChannelType>(1);
		let db = StorageDB::open_default(tmp_dir.path()).unwrap();

		let mut rng = rand::thread_rng();
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let jwt_private_key = rsa_private_key.to_pkcs1_der().unwrap();
		let pumpx_api = PumpxApiClient::new("https://api.pumpx.ai".to_string());
		let config_loader = ConfigLoader::from_env();
		let signer_client: Arc<Box<dyn SignerClient>> = Arc::new(Box::new(MockSignerClient::new()));

		// Create console mailer for test
		let mailer: Box<dyn MailerTrait + Send + Sync> = Box::new(ConsoleMailer::new());

		start_server(
			port,
			shielding_key.clone(),
			Arc::new(sender),
			Arc::new(Box::new(pumpx_api)),
			Arc::new(db),
			jwt_private_key.as_bytes().to_vec(),
			&config_loader,
			signer_client,
			mailer,
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
