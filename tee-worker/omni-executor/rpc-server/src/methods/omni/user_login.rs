use crate::{
	error_code::*, server::RpcContext, verify_auth::verify_auth, Deserialize, ErrorCode, Serialize,
};
use chrono::{Days, Utc};
use executor_crypto::jwt;
use executor_primitives::{utils::hex::ToHexPrefixed, OmniAuth, UserAuth, UserId};
use heima_authentication::auth_token::{
	AuthOptions, AuthTokenClaims, AUTH_TOKEN_EXPIRATION_DAYS, AUTH_TOKEN_ID_TYPE,
};
use heima_primitives::Identity;
use jsonrpsee::{types::ErrorObject, RpcModule};
use tracing::error;

#[derive(Debug, Deserialize, Clone)]
pub struct UserLoginParams {
	pub user_id: UserId,
	pub user_auth: UserAuth,
	pub client_id: String,
}

#[derive(Serialize, Clone)]
pub struct UserLoginResponse {
	pub access_token: String,
	pub id_token: String,
}

impl TryFrom<UserLoginParams> for OmniAuth {
	type Error = ErrorCode;

	fn try_from(p: UserLoginParams) -> Result<Self, Self::Error> {
		let omni_auth = match p.user_auth {
			UserAuth::Email(code) => {
				let UserId::Email(email) = p.user_id else {
					error!("User ID must be an email for Email authentication");
					return Err(ErrorCode::ParseError);
				};
				OmniAuth::Email(email, code)
			},
			UserAuth::Web3(signature) => {
				let identity = Identity::try_from(p.user_id).map_err(|_| {
					error!("Invalid user ID format");
					ErrorCode::ParseError
				})?;
				if !identity.is_web3() {
					error!("User ID must be a Web3 identity for Web3 authentication");
					return Err(ErrorCode::ParseError);
				}
				OmniAuth::Web3(identity, signature)
			},
			UserAuth::AuthToken(token) => OmniAuth::AuthToken(token),
			UserAuth::OAuth2(data) => {
				let identity = Identity::try_from(p.user_id).map_err(|_| {
					error!("Invalid user ID format");
					ErrorCode::ParseError
				})?;
				OmniAuth::OAuth2(identity, data)
			},
		};

		Ok(omni_auth)
	}
}

pub fn register_user_login(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_userLogin", |params, ctx, _| async move {
			let params = params.parse::<UserLoginParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				ErrorCode::ParseError
			})?;
			let auth = OmniAuth::try_from(params.clone()).map_err(|e| {
				error!("Failed to convert params to OmniAuth: {:?}", e);
				ErrorCode::ParseError
			})?;
			verify_auth(ctx.clone(), &auth).await.map_err(|_| {
				error!("Failed to verify auth: {:?}", auth);
				ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE)
			})?;
			let identity = Identity::try_from(params.user_id).map_err(|_| {
				error!("Invalid user ID format");
				ErrorCode::ParseError
			})?;
			let id_token =
				create_jwt_for_user(identity, AUTH_TOKEN_ID_TYPE, &ctx.jwt_rsa_private_key)
					.map_err(|_| {
						error!("Failed to create access token for user");
						ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE)
					})?;

			// TOOD:
			// - Call client specific api ?
			// - Generate access token
			// - Store tokens
			// - include client's api response in our response?

			Ok::<UserLoginResponse, ErrorObject>(UserLoginResponse {
				access_token: id_token,
				id_token: "mock_id_token".to_string(),
			})
		})
		.expect("Failed to register omni_requestJwt method");
}

fn create_jwt_for_user(
	identity: Identity,
	token_type: &str,
	jwt_rsa_private_key: &[u8],
) -> Result<String, ()> {
	let expires_at = Utc::now()
		.checked_add_days(Days::new(AUTH_TOKEN_EXPIRATION_DAYS))
		.expect("Failed to calculate expiration")
		.timestamp();
	let auth_options = AuthOptions { expires_at };
	let omni_account = identity.to_omni_account();
	let token_claims =
		AuthTokenClaims::new(omni_account.to_hex(), token_type.to_string(), auth_options.clone());
	jwt::create(&token_claims, jwt_rsa_private_key).map_err(|e| {
		error!("Failed to create JWT token: {:?}", e);
	})
}
