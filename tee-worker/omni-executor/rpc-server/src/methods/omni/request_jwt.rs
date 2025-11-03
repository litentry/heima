use super::check_omni_api_response;
use crate::{
	detailed_error::DetailedError,
	error_code::{INTERNAL_ERROR_CODE, PARSE_ERROR_CODE, *},
	server::RpcContext,
	verify_auth::verify_auth,
	Deserialize,
};
use chrono::{Days, Utc};
use executor_core::intent_executor::IntentExecutor;
use executor_crypto::jwt;
use executor_primitives::{utils::hex::hex_encode, OmniAuth};
use executor_storage::{HeimaJwtStorage, Storage};
use heima_authentication::{
	auth_token::*,
	constants::{AUTH_TOKEN_ACCESS_TYPE, AUTH_TOKEN_EXPIRATION_DAYS, AUTH_TOKEN_ID_TYPE},
};
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::RpcModule;
use pumpx::methods::user_connect::UserConnectResponse;
use serde::Serialize;
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct RequestJwtParams {
	pub client_id: String,
	pub user_email: String,
	pub invite_code: Option<String>,
	pub google_code: String,
	pub language: Option<String>,
	pub email_code: String,
}

#[derive(Serialize, Clone)]
pub struct RequestJwtResponse {
	pub access_token: String,
	pub id_token: String,
	pub backend_response: UserConnectResponse,
}

impl RequestJwtParams {
	pub fn get_omni_auth(&self) -> OmniAuth {
		OmniAuth::Email(self.client_id.clone(), self.user_email.clone(), self.email_code.clone())
	}
}

pub fn register_request_jwt<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_requestJwt", |params, ctx, _ext| async move {
			let params = params.parse::<RequestJwtParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				DetailedError::new(PARSE_ERROR_CODE, "Parse error")
					.with_reason("Invalid JSON format or missing required fields")
			})?;

			debug!(
				"Received omni_requestJwt, user_email: {}, client_id: {}",
				params.user_email, params.client_id
			);

			// Verify email authentication
			let auth = params.get_omni_auth();
			verify_auth(ctx.clone(), &auth).await.map_err(|e| {
				error!("Failed to verify auth: {:?}, reason: {:?}", auth, e);
				DetailedError::new(
					AUTH_VERIFICATION_FAILED_CODE,
					"Authentication verification failed",
				)
				.with_suggestion("Please check your authentication credentials")
			})?;

			// Inlined handler logic from handle_pumpx_request_jwt
			let expires_at = Utc::now()
				.checked_add_days(Days::new(AUTH_TOKEN_EXPIRATION_DAYS))
				.expect("Failed to calculate expiration")
				.timestamp();
			let auth_options = AuthOptions { expires_at };

			debug!("Calling pumpx get_account_user_id, email: {}", params.user_email);
			let res = ctx.pumpx_api.get_account_user_id(params.user_email.clone()).await.map_err(
				|e| {
					error!(
						"Failed to get_account_user_id for email {}: {:?}",
						params.user_email, e
					);
					DetailedError::new(INTERNAL_ERROR_CODE, "Failed to get account user ID")
						.with_suggestion("Please try again")
				},
			)?;
			debug!("Response pumpx get_account_user_id: {:?}", res);

			let Some(user_id) = res.data.user_id else {
				error!("Response data.user_id of call get_account_user_id is none");
				return Err(DetailedError::new(
					INTERNAL_ERROR_CODE,
					"Failed to get account user ID",
				)
				.with_reason("User ID not found in response")
				.into());
			};

			debug!("get_account_user_id ok, email: {}, user_id: {}", params.user_email, user_id);
			let omni_account = Identity::from_web2_account(&user_id, Web2IdentityType::Pumpx)
				.to_omni_account(&params.client_id);

			let access_token_claims: AuthTokenClaims = AuthTokenClaims::new(
				hex_encode(omni_account.as_ref()),
				AUTH_TOKEN_ACCESS_TYPE.to_string(),
				params.client_id.to_string(),
				auth_options.clone(),
			);
			let access_token = jwt::create(&access_token_claims, &ctx.jwt_rsa_private_key)
				.map_err(|e| {
					error!("Failed to create access token: {:?}", e);
					DetailedError::new(INTERNAL_ERROR_CODE, "Failed to create authentication token")
						.into()
				})?;

			debug!(
				"Calling pumpx user_connect, user_id: {}, email: {}, invite_code: {:?}, google_code: {:?}",
				user_id, params.user_email, params.invite_code, params.google_code
			);
			let backend_response = ctx
				.pumpx_api
				.user_connect(
					&access_token,
					user_id.clone(),
					params.user_email.clone(),
					params.invite_code,
					params.google_code,
					params.language,
				)
				.await
				.map_err(|e| {
					error!("Failed to connect user: {:?}", e);
					DetailedError::new(INTERNAL_ERROR_CODE, "Failed to connect user")
						.with_suggestion("Please try again")
						.into()
				})?;
			debug!("Response pumpx user_connect: {:?}", backend_response);

			// check google auth value
			if !backend_response.data.google_auth_check.unwrap_or(false) {
				error!("Google code verification failed from user_connect");
				return Err(DetailedError::new(
					PUMPX_API_GOOGLE_CODE_VERIFICATION_FAILED_CODE,
					"Google code verification failed",
				)
				.with_suggestion("Please check your Google verification code and try again")
				.into());
			}

			let id_token_claims = AuthTokenClaims::new(
				hex_encode(omni_account.as_ref()),
				AUTH_TOKEN_ID_TYPE.to_string(),
				params.client_id.to_string(),
				auth_options,
			);
			let id_token =
				jwt::create(&id_token_claims, &ctx.jwt_rsa_private_key).map_err(|e| {
					error!("Failed to create id token: {:?}", e);
					DetailedError::new(INTERNAL_ERROR_CODE, "Failed to create authentication token")
						.into()
				})?;

			let storage = HeimaJwtStorage::new(ctx.storage_db.clone());
			if storage
				.insert(&(omni_account.clone(), AUTH_TOKEN_ACCESS_TYPE), access_token.clone())
				.is_err()
			{
				error!("Failed to insert pumpx_{}_jwt_token into storage", AUTH_TOKEN_ACCESS_TYPE);
			};

			if storage.insert(&(omni_account, AUTH_TOKEN_ID_TYPE), id_token.clone()).is_err() {
				error!("Failed to insert pumpx_{}_jwt_token into storage", AUTH_TOKEN_ID_TYPE);
			};

			check_omni_api_response(backend_response.clone(), "Request pumpx jwt".into())?;
			Ok(RequestJwtResponse { access_token, id_token, backend_response })
		})
		.expect("Failed to register omni_requestJwt method");
}
