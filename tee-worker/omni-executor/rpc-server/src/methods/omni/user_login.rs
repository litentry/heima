use super::check_backend_response;
use crate::{
	detailed_error::DetailedError,
	error_code::*,
	server::RpcContext,
	utils::{types::RpcResultExt, validation::parse_rpc_params},
	verify_auth::verify_auth,
	Deserialize, ErrorCode, Serialize,
};

use chrono::{Days, Utc};
use executor_core::intent_executor::IntentExecutor;
use executor_crypto::jwt;
use executor_primitives::{
	to_omni_auth, utils::hex::hex_encode, ClientAuth, OmniAuth, UserAuth, UserId,
};
use executor_storage::{HeimaJwtStorage, Storage};
use heima_authentication::{
	auth_token::{AuthOptions, AuthTokenClaims},
	constants::{
		AUTH_TOKEN_ACCESS_TYPE, AUTH_TOKEN_EXPIRATION_DAYS, AUTH_TOKEN_ID_TYPE, CLIENT_ID_WILDMETA,
	},
};
use jsonrpsee::{types::ErrorObject, RpcModule};
use pumpx::methods::post_heima_login::{PostHeimaLoginBody, PostHeimaLoginResponse};
use tracing::error;

#[derive(Debug, Deserialize, Clone)]
pub struct UserLoginParams {
	pub user_id: UserId,
	pub user_auth: UserAuth,
	pub client_id: String,
	pub client_auth: Option<ClientAuth>,
}

#[derive(Serialize, Clone)]
pub struct UserLoginResponse {
	pub access_token: String,
	pub id_token: String,
	pub backend_response: PostHeimaLoginResponse,
}

impl TryFrom<UserLoginParams> for OmniAuth {
	type Error = ErrorCode;

	fn try_from(p: UserLoginParams) -> Result<Self, Self::Error> {
		to_omni_auth(&p.user_auth, &p.user_id, &p.client_id).map_err(|_| ErrorCode::ParseError)
	}
}

pub fn register_user_login<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_userLogin", |params, ctx, _| async move {
			let params = parse_rpc_params::<UserLoginParams>(params)?;
			let auth = OmniAuth::try_from(params.clone()).map_err(|e| {
				error!("Failed to convert params to OmniAuth: {:?}", e);
				ErrorCode::ParseError
			})?;
			verify_auth(ctx.clone(), &auth).await.map_err(|_| {
				error!("Failed to verify auth: {:?}", auth);
				ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE)
			})?;
			let omni_account = params.user_id.to_omni_account(&params.client_id).map_err(|_| {
				error!("Failed to convert user_id to omni_account");
				ErrorCode::ParseError
			})?;

			let (id_token, access_token) =
				create_jwt(omni_account.as_ref(), &params.client_id, &ctx.jwt_rsa_private_key)
					.map_err_internal("Failed to create jwt")?;

			if params.client_id == CLIENT_ID_WILDMETA {
				let body = PostHeimaLoginBody {
					user_id: params.user_id,
					client_id: params.client_id.clone(),
					client_auth: params.client_auth,
					heima_login_success: true,
				};
				let backend_response =
					ctx.pumpx_api.post_heima_login(&access_token, body).await.map_err(|e| {
						error!("Failed to call post_heim_login: {:?}", e);
						DetailedError::pumpx_service_error("post_heima_login", format!("{:?}", e))
							.to_rpc_error()
					})?;

				check_backend_response(&backend_response, "post_heima_login")?;

				let storage = HeimaJwtStorage::new(ctx.storage_db.clone());
				if let Err(e) =
					storage.insert(&(omni_account, AUTH_TOKEN_ACCESS_TYPE), access_token.clone())
				{
					error!(
						"Failed to insert pumpx_{}_jwt_token into storage: {:?}",
						AUTH_TOKEN_ACCESS_TYPE, e
					);
					return Err(
						DetailedError::storage_service_error("insert access token").to_rpc_error()
					);
				};
				Ok::<UserLoginResponse, ErrorObject>(UserLoginResponse {
					access_token,
					id_token,
					backend_response,
				})
			} else {
				error!("Unsupported client_id: {}", params.client_id);
				Err(ErrorCode::InvalidParams.into())
			}
		})
		.expect("Failed to register omni_requestJwt method");
}

fn create_jwt(
	omni_account: &[u8],
	client_id: &str,
	jwt_rsa_private_key: &[u8],
) -> Result<(String, String), ()> {
	let expires_at = Utc::now()
		.checked_add_days(Days::new(AUTH_TOKEN_EXPIRATION_DAYS))
		.expect("Failed to calculate expiration")
		.timestamp();
	let auth_options = AuthOptions { expires_at };
	let token_claims = AuthTokenClaims::new(
		hex_encode(omni_account),
		AUTH_TOKEN_ID_TYPE.to_string(),
		client_id.to_string(),
		auth_options.clone(),
	);
	let id_token = jwt::create(&token_claims, jwt_rsa_private_key).map_err(|e| {
		error!("Failed to create JWT token: {:?}", e);
	})?;

	let token_claims = AuthTokenClaims::new(
		hex_encode(omni_account),
		AUTH_TOKEN_ACCESS_TYPE.to_string(),
		client_id.to_string(),
		auth_options.clone(),
	);

	let access_token = jwt::create(&token_claims, jwt_rsa_private_key).map_err(|e| {
		error!("Failed to create JWT token: {:?}", e);
	})?;

	Ok((id_token, access_token))
}
