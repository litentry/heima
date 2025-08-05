use super::common::{check_pumpx_api_response, handle_pumpx_native_task};
use crate::{
	error_code::*, methods::pumpx::PumpxRpcError, server::RpcContext, verify_auth::verify_auth,
	Deserialize, ErrorCode,
};
use executor_core::intent_executor::IntentExecutor;
use executor_core::native_task::*;
use executor_primitives::OmniAuth;
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
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
	pub fn into_native_task_wrapper(self) -> NativeTaskWrapper<NativeTask> {
		NativeTaskWrapper::new(
			NativeTask::PumpxRequestJwt(
				Identity::from_web2_account(self.user_email.as_str(), Web2IdentityType::Email), // actually unused
				self.user_email.clone(),
				self.invite_code,
				self.google_code,
				self.language,
			),
			None,
			Some(OmniAuth::Email(self.client_id.clone(), self.user_email, self.email_code)),
			self.client_id,
		)
	}
}

pub fn register_request_jwt<
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
		.register_async_method("pumpx_requestJwt", |params, ctx, _ext| async move {
			let params = params.parse::<RequestJwtParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!(
				"Received pumpx_requestJwt, user_email: {}, client_id: {}",
				params.user_email, params.client_id
			);

			let wrapper = params.into_native_task_wrapper();

			if wrapper.task.require_auth() {
				let Some(ref auth) = wrapper.auth else {
					error!("Missing auth");
					return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
						REQUIRE_AUTHENTICATION_CODE,
					)));
				};
				verify_auth(ctx.clone(), auth).await.map_err(|_| {
					error!("Failed to verify auth: {:?}", wrapper.auth);
					PumpxRpcError::from_error_code(ErrorCode::ServerError(
						AUTH_VERIFICATION_FAILED_CODE,
					))
				})?;
			}

			handle_pumpx_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::PumpxRequestJwt { access_token, id_token, backend_response } => {
					check_pumpx_api_response(backend_response.clone(), "Request pumpx jwt".into())?;
					Ok(RequestJwtResponse { access_token, id_token, backend_response })
				},
				_ => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register pumpx_requestJwt method");
}
