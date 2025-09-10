use super::common::{check_omni_api_response, handle_omni_native_task};
use crate::{
	detailed_error::DetailedError,
	error_code::{INTERNAL_ERROR_CODE, PARSE_ERROR_CODE, *},
	methods::omni::PumpxRpcError,
	server::RpcContext,
	verify_auth::verify_auth,
	Deserialize,
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
use std::sync::Arc;
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

#[tracing::instrument(skip(params, ctx), fields(
	client_id = %params.client_id,
	user_email = %params.user_email,
	invite_code = ?params.invite_code,
	language = ?params.language
))]
async fn handle_request_jwt_request<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	params: RequestJwtParams,
	ctx: Arc<
		RpcContext<
			Header,
			RpcClient,
			RpcClientFactory,
			EthereumIntentExecutor,
			SolanaIntentExecutor,
			CrossChainIntentExecutor,
		>,
	>,
) -> Result<RequestJwtResponse, PumpxRpcError> {
	debug!("Processing omni_requestJwt request");

	let wrapper = params.into_native_task_wrapper();

	if wrapper.task.require_auth() {
		let Some(ref auth) = wrapper.auth else {
			error!("Missing auth token");
			return Err(PumpxRpcError::from(
				DetailedError::new(REQUIRE_AUTHENTICATION_CODE, "Authentication required")
					.with_suggestion("Please provide authentication credentials"),
			));
		};
		verify_auth(ctx.clone(), auth).await.map_err(|e| {
			error!("Failed to verify auth: {:?}, reason: {:?}", wrapper.auth, e);
			PumpxRpcError::from(
				DetailedError::new(
					AUTH_VERIFICATION_FAILED_CODE,
					"Authentication verification failed",
				)
				.with_suggestion("Please check your authentication credentials"),
			)
		})?;
	}

	handle_omni_native_task(&ctx, wrapper, |task_ok| match task_ok {
		NativeTaskOk::PumpxRequestJwt { access_token, id_token, backend_response } => {
			check_omni_api_response(backend_response.clone(), "Request pumpx jwt".into())?;
			Ok(RequestJwtResponse { access_token, id_token, backend_response })
		},
		_ => {
			error!("Unexpected response type");
			Err(PumpxRpcError::from(
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason("Unexpected response type from native task handler"),
			))
		},
	})
	.await
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
		.register_async_method("omni_requestJwt", |params, ctx, _ext| async move {
			let params = params.parse::<RequestJwtParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Parse error")
						.with_reason("Invalid JSON format or missing required fields"),
				)
			})?;

			handle_request_jwt_request(params, ctx).await
		})
		.expect("Failed to register omni_requestJwt method");
}
