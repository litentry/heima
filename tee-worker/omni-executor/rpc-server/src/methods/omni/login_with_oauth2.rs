use crate::{
	detailed_error::DetailedError, error_code::AUTH_VERIFICATION_FAILED_CODE, server::RpcContext,
	utils::validation::parse_rpc_params, verify_auth::verify_oauth2_authentication, Deserialize,
	Serialize,
};
use chrono::{Days, Utc};
use executor_core::intent_executor::IntentExecutor;
use executor_crypto::jwt;
use executor_primitives::{utils::hex::hex_encode, OAuth2Data, OAuth2Provider};
use heima_authentication::{
	auth_token::{AuthOptions, AuthTokenClaims},
	constants::{AUTH_TOKEN_EXPIRATION_DAYS, AUTH_TOKEN_ID_TYPE},
};
use heima_primitives::Identity;
use jsonrpsee::{core::RpcResult, types::ErrorObject, RpcModule};
use tracing::error;

#[derive(Debug, Deserialize, Clone)]
pub struct LoginWithOAuth2Params {
	pub client_id: String,
	pub provider: String,
	pub code: String,
	pub state: String,
	pub redirect_uri: Option<String>,
	pub uid: String,
	pub id_token: String,
}

#[derive(Serialize, Clone)]
pub struct LoginWithOAuth2Response {
	pub user_id: String,
	pub access_token: String,
}

fn parse_oauth2_provider(provider: &str) -> RpcResult<OAuth2Provider> {
	match provider.to_lowercase().as_str() {
		"google" => Ok(OAuth2Provider::Google),
		"apple" => Ok(OAuth2Provider::Apple),
		_ => {
			let msg = format!("Unsupported OAuth2 provider: {}", provider);
			error!(msg);
			Err(DetailedError::parse_error(&msg).to_rpc_error())
		},
	}
}

fn create_jwt_for_user(
	identity: Identity,
	token_type: &str,
	client_id: &str,
	jwt_rsa_private_key: &[u8],
) -> Result<String, ()> {
	let expires_at = Utc::now()
		.checked_add_days(Days::new(AUTH_TOKEN_EXPIRATION_DAYS))
		.expect("Failed to calculate expiration")
		.timestamp();
	let auth_options = AuthOptions { expires_at };
	let omni_account = identity.to_omni_account(client_id);
	let token_claims = AuthTokenClaims::new(
		hex_encode(omni_account.as_ref()),
		token_type.to_string(),
		client_id.to_string(),
		auth_options,
	);
	jwt::create(&token_claims, jwt_rsa_private_key).map_err(|e| {
		error!("Failed to create JWT token: {:?}", e);
	})
}

pub fn register_login_with_oauth2<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_loginWithOAuth2", |params, ctx, _| async move {
			let params = parse_rpc_params::<LoginWithOAuth2Params>(params)?;

			let provider = parse_oauth2_provider(&params.provider)?;

			let oauth2_data = OAuth2Data {
				provider,
				code: params.code.clone(),
				state: params.state.clone(),
				redirect_uri: params.redirect_uri.clone(),
				uid: params.uid.clone(),
				id_token: params.id_token.clone(),
			};

			let verified_identity =
				verify_oauth2_authentication(ctx.clone(), &params.client_id, &oauth2_data)
					.await
					.map_err(|e| {
						error!("Failed to verify OAuth2 authentication: {:?}", e);
						DetailedError::new(
							AUTH_VERIFICATION_FAILED_CODE,
							"OAuth2 authentication failed",
						)
						.with_reason(format!("{}", e))
						.with_suggestion("Please check your OAuth2 credentials and try again")
						.to_rpc_error()
					})?;

			let access_token = create_jwt_for_user(
				verified_identity.clone(),
				AUTH_TOKEN_ID_TYPE,
				&params.client_id,
				&ctx.jwt_rsa_private_key,
			)
			.map_err(|_| {
				let msg = "Failed to create access token for user";
				error!(msg);
				DetailedError::internal_error(msg).to_rpc_error()
			})?;

			let user_id = match &verified_identity {
				Identity::Google(identity_string) | Identity::Apple(identity_string) => {
					std::str::from_utf8(identity_string.inner_ref())
						.map_err(|_| {
							let msg = "Failed to convert identity to string";
							error!(msg);
							DetailedError::internal_error(msg).to_rpc_error()
						})?
						.to_string()
				},
				_ => {
					let msg =
						format!("Unexpected identity type for OAuth2: {:?}", verified_identity);
					error!(msg);
					return Err(DetailedError::internal_error(&msg).to_rpc_error());
				},
			};

			Ok::<LoginWithOAuth2Response, ErrorObject>(LoginWithOAuth2Response {
				user_id,
				access_token,
			})
		})
		.expect("Failed to register omni_loginWithOAuth2 method");
}
