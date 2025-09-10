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
use pumpx::methods::add_wallet::AddWalletResponse;
use serde::Serialize;
use std::sync::Arc;
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct AddWalletParams {
	pub client_id: String,
	pub user_id: String,
	pub auth_token: String,
}

#[derive(Serialize, Clone)]
pub struct RPCAddWalletResponse {
	pub backend_response: AddWalletResponse,
}

impl AddWalletParams {
	pub fn into_native_task_wrapper(self) -> NativeTaskWrapper<NativeTask> {
		let identity = Identity::from_web2_account(self.user_id.as_str(), Web2IdentityType::Pumpx);
		let omni_account = identity.to_omni_account(&self.client_id);
		NativeTaskWrapper::new(
			NativeTask::PumpxAddWallet(omni_account),
			None,
			Some(OmniAuth::AuthToken(self.auth_token)),
			self.client_id,
		)
	}
}

#[tracing::instrument(skip(ctx), fields(user_id = %params.user_id, client_id = %params.client_id))]
async fn handle_add_wallet_request<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	params: AddWalletParams,
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
) -> Result<RPCAddWalletResponse, PumpxRpcError> {
	debug!("Processing pumpx_addWallet request");

	let wrapper = params.into_native_task_wrapper();

	if wrapper.task.require_auth() {
		let Some(ref auth) = wrapper.auth else {
			error!("Missing auth token");
			return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
				REQUIRE_AUTHENTICATION_CODE,
			)));
		};
		verify_auth(ctx.clone(), auth).await.map_err(|_| {
			error!("Failed to verify auth: {:?}", wrapper.auth);
			PumpxRpcError::from_error_code(ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE))
		})?;
	}

	handle_pumpx_native_task(&ctx, wrapper, |task_ok| match task_ok {
		NativeTaskOk::PumpxAddWallet(response) => {
			check_pumpx_api_response(response.clone(), "Add wallet".into())?;
			Ok(RPCAddWalletResponse { backend_response: response })
		},
		_ => {
			error!("Unexpected response type");
			Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
		},
	})
	.await
}

pub fn register_add_wallet<
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
		.register_async_method("pumpx_addWallet", |params, ctx, _ext| async move {
			let params = params.parse::<AddWalletParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			handle_add_wallet_request(params, ctx).await
		})
		.expect("Failed to register pumpx_addWallet method");
}
