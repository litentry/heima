use super::common::check_omni_api_response;
use crate::{
	error_code::*, server::RpcContext, verify_auth::verify_auth, Deserialize, ErrorCode, Serialize,
};

use chrono::{Days, Utc};
use executor_core::intent_executor::IntentExecutor;
use executor_crypto::jwt;
use executor_primitives::{
	to_omni_auth, utils::hex::ToHexPrefixed, ClientAuth, OmniAuth, UserAuth, UserId,
};
use executor_storage::{HeimaJwtStorage, Storage};
use heima_authentication::{
	auth_token::{AuthOptions, AuthTokenClaims},
	constants::{
		AUTH_TOKEN_ACCESS_TYPE, AUTH_TOKEN_EXPIRATION_DAYS, AUTH_TOKEN_ID_TYPE, CLIENT_ID_WILDMETA,
	},
};
use heima_primitives::Identity;
use jsonrpsee::{types::ErrorObject, RpcModule};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
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

pub fn register_user_login<
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
			let identity = Identity::try_from(params.user_id.clone()).map_err(|_| {
				error!("Invalid user ID format");
				ErrorCode::ParseError
			})?;
			let id_token = create_jwt_for_user(
				identity.clone(),
				AUTH_TOKEN_ID_TYPE,
				&params.client_id,
				&ctx.jwt_rsa_private_key,
			)
			.map_err(|_| {
				error!("Failed to create access token for user");
				ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE)
			})?;
			let access_token = create_jwt_for_user(
				identity.clone(),
				AUTH_TOKEN_ACCESS_TYPE,
				&params.client_id,
				&ctx.jwt_rsa_private_key,
			)
			.map_err(|_| {
				error!("Failed to create access token for user");
				ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE)
			})?;

			if params.client_id == CLIENT_ID_WILDMETA {
				let body = PostHeimaLoginBody {
					user_id: params.user_id,
					client_id: params.client_id.clone(),
					client_auth: params.client_auth,
					heima_login_success: true,
				};
				let Ok(backend_response) =
					ctx.pumpx_api.post_heima_login(&access_token, body).await
				else {
					error!("Post_heima_login failed for Wildmeta client");
					return Err(ErrorCode::ServerError(POST_HEIMA_LOGIN_FAILED_CODE).into());
				};

				check_omni_api_response(backend_response.clone(), "Post heima login".into())?;

				let storage = HeimaJwtStorage::new(ctx.storage_db.clone());
				let omni_account = identity.to_omni_account(&params.client_id);
				if storage
					.insert(&(omni_account, AUTH_TOKEN_ACCESS_TYPE), access_token.clone())
					.is_err()
				{
					error!(
						"Failed to insert pumpx_{}_jwt_token into storage",
						AUTH_TOKEN_ACCESS_TYPE
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
		omni_account.to_hex(),
		token_type.to_string(),
		client_id.to_string(),
		auth_options.clone(),
	);
	jwt::create(&token_claims, jwt_rsa_private_key).map_err(|e| {
		error!("Failed to create JWT token: {:?}", e);
	})
}
