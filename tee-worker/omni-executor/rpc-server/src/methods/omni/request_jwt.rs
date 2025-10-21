use super::common::check_omni_api_response;
use crate::{
	detailed_error::DetailedError,
	error_code::{INTERNAL_ERROR_CODE, PARSE_ERROR_CODE, *},
	methods::omni::PumpxRpcError,
	server::RpcContext,
	verify_auth::verify_auth,
	Deserialize,
};
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::OmniAuth;
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::RpcModule;
use native_task_handler::{handle_pumpx_request_jwt, NativeTaskError, NativeTaskOk};
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

pub fn register_request_jwt<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>,
	>,
) {
	module
		.register_async_method("omni_requestJwt", |params, ctx, _ext| async move {
			let params = params.parse::<RequestJwtParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Parse error")
						.with_reason("Invalid JSON format or missing required fields"),
				)
			})?;

			debug!(
				"Received omni_requestJwt, user_email: {}, client_id: {}",
				params.user_email, params.client_id
			);

			// Verify email authentication
			let auth = params.get_omni_auth();
			verify_auth(ctx.clone(), &auth).await.map_err(|e| {
				error!("Failed to verify auth: {:?}, reason: {:?}", auth, e);
				PumpxRpcError::from(
					DetailedError::new(
						AUTH_VERIFICATION_FAILED_CODE,
						"Authentication verification failed",
					)
					.with_suggestion("Please check your authentication credentials"),
				)
			})?;

			// Call the handler directly
			let sender =
				Identity::from_web2_account(params.user_email.as_str(), Web2IdentityType::Email);
			let result = handle_pumpx_request_jwt(
				ctx.to_task_handler_context(),
				sender,
				params.user_email.clone(),
				params.invite_code,
				params.google_code,
				params.language,
				params.client_id,
			)
			.await;

			// Process response
			match result {
				Ok(NativeTaskOk::PumpxRequestJwt { access_token, id_token, backend_response }) => {
					check_omni_api_response(backend_response.clone(), "Request pumpx jwt".into())?;
					Ok(RequestJwtResponse { access_token, id_token, backend_response })
				},
				Ok(_) => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from(
						DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
							.with_reason("Unexpected response type from native task handler"),
					))
				},
				Err(NativeTaskError::PumpxApiError(e)) => {
					error!("Pumpx API error: {:?}", e);
					Err(PumpxRpcError::from(
						DetailedError::new(INTERNAL_ERROR_CODE, "Pumpx API error")
							.with_suggestion("Please try again"),
					))
				},
				Err(NativeTaskError::AuthTokenCreationFailed) => {
					Err(PumpxRpcError::from(DetailedError::new(
						INTERNAL_ERROR_CODE,
						"Failed to create authentication token",
					)))
				},
				Err(NativeTaskError::InternalError(message)) => {
					error!("Internal error in native task");
					match message {
						Some(msg) => {
							Err(PumpxRpcError::from_code_and_message(INTERNAL_ERROR_CODE, msg))
						},
						None => Err(PumpxRpcError::from_error_code(
							jsonrpsee::types::ErrorCode::InternalError,
						)),
					}
				},
				Err(e) => {
					error!("Native task error: {:?}", e);
					Err(PumpxRpcError::from(
						DetailedError::new(INTERNAL_ERROR_CODE, "Operation failed")
							.with_suggestion("Please try again"),
					))
				},
			}
		})
		.expect("Failed to register omni_requestJwt method");
}
