use crate::{
	detailed_error::DetailedError,
	error_code::{AUTH_VERIFICATION_FAILED_CODE, INTERNAL_ERROR_CODE, PARSE_ERROR_CODE},
	server::RpcContext,
	verify_auth::verify_oauth2_authentication,
	Deserialize, ErrorCode, Serialize,
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
use jsonrpsee::{types::ErrorObject, RpcModule};
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

fn parse_oauth2_provider(provider: &str) -> Result<OAuth2Provider, ErrorCode> {
	match provider.to_lowercase().as_str() {
		"google" => Ok(OAuth2Provider::Google),
		"apple" => Ok(OAuth2Provider::Apple),
		_ => {
			error!("Unsupported OAuth2 provider: {}", provider);
			Err(ErrorCode::InvalidParams)
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
			let params = params.parse::<LoginWithOAuth2Params>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				DetailedError::new(PARSE_ERROR_CODE, "Parse error")
					.with_reason("Invalid JSON format or missing required fields")
					.to_error_object()
			})?;

			let provider = parse_oauth2_provider(&params.provider).map_err(|_e| {
				error!("Invalid OAuth2 provider: {}", params.provider);
				DetailedError::new(PARSE_ERROR_CODE, "Invalid provider")
					.with_field("provider")
					.with_received(&params.provider)
					.with_expected("google, apple")
					.to_error_object()
			})?;

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
						.to_error_object()
					})?;

			let access_token = create_jwt_for_user(
				verified_identity.clone(),
				AUTH_TOKEN_ID_TYPE,
				&params.client_id,
				&ctx.jwt_rsa_private_key,
			)
			.map_err(|_| {
				error!("Failed to create access token for user");
				DetailedError::new(INTERNAL_ERROR_CODE, "Failed to create access token")
					.with_reason("Internal error while generating JWT token")
					.to_error_object()
			})?;

			let user_id = match &verified_identity {
				Identity::Google(identity_string) | Identity::Apple(identity_string) => {
					std::str::from_utf8(identity_string.inner_ref())
						.map_err(|_| {
							error!("Failed to convert identity to string");
							DetailedError::new(INTERNAL_ERROR_CODE, "Failed to parse identity")
								.with_reason("Invalid UTF-8 in identity string")
								.to_error_object()
						})?
						.to_string()
				},
				_ => {
					error!("Unexpected identity type for OAuth2: {:?}", verified_identity);
					return Err(DetailedError::new(
						INTERNAL_ERROR_CODE,
						"Failed to extract user identity",
					)
					.with_reason("OAuth2 provider returned unsupported identity type")
					.to_error_object());
				},
			};

			Ok::<LoginWithOAuth2Response, ErrorObject>(LoginWithOAuth2Response {
				user_id,
				access_token,
			})
		})
		.expect("Failed to register omni_loginWithOAuth2 method");
}
