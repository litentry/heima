use super::common::handle_omni_native_task;
use crate::methods::omni::{common::check_auth, PumpxRpcError};
use crate::{error_code::*, server::RpcContext, Deserialize, ErrorCode};
use executor_core::intent_executor::IntentExecutor;
use executor_core::native_task::*;
use executor_primitives::{utils::hex::FromHexPrefixed, AccountId};
use heima_primitives::Address32;
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct NotifyLimitOrderResultParams {
	pub intent_id: u32,
	pub result: String,
	pub message: Option<String>,
}

pub fn register_notify_limit_order_result<
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
		.register_async_method("omni_notifyLimitOrderResult", |params, ctx, ext| async move {
			let user = check_auth(&ext).map_err(|e| {
				error!("Authentication check failed: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ServerError(
					AUTH_VERIFICATION_FAILED_CODE,
				))
			})?;

			let params = params.parse::<NotifyLimitOrderResultParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!(
				"Received omni_notifyLimitOrderResult, intent_id: {}, result: {}, message: {:?}",
				params.intent_id, params.result, params.message
			);

			let Ok(address) = Address32::from_hex(&user.omni_account) else {
				error!("Failed to parse from omni account token");
				return Err(PumpxRpcError::from_error_code(ErrorCode::InternalError));
			};

			let wrapper = NativeTaskWrapper::new(
				NativeTask::PumpxNotifyLimitOrderResult(
					AccountId::from(address),
					params.intent_id,
					params.result,
					params.message,
				),
				None,
				None,
				user.client_id,
			);

			handle_omni_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::PumpxNotifyLimitOrderResult => Ok(()),
				_ => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register omni_notifyLimitOrderResult method");
}
