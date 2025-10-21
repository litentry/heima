use crate::methods::omni::{common::check_auth, PumpxRpcError};
use crate::{
	detailed_error::DetailedError,
	error_code::{INTERNAL_ERROR_CODE, PARSE_ERROR_CODE, *},
	server::RpcContext,
	Deserialize,
};
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::{utils::hex::FromHexPrefixed, AccountId};
use heima_primitives::Address32;
use jsonrpsee::RpcModule;
use native_task_handler::{handle_pumpx_notify_limit_order_result, NativeTaskError, NativeTaskOk};
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
>(
	module: &mut RpcModule<
		RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>,
	>,
) {
	module
		.register_async_method("omni_notifyLimitOrderResult", |params, ctx, ext| async move {
			let user = check_auth(&ext).map_err(|e| {
				error!("Authentication check failed: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(
						AUTH_VERIFICATION_FAILED_CODE,
						"Authentication verification failed",
					)
					.with_suggestion("Please check your authentication credentials"),
				)
			})?;

			let params = params.parse::<NotifyLimitOrderResultParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Parse error")
						.with_reason("Invalid JSON format or missing required fields"),
				)
			})?;

			debug!(
				"Received omni_notifyLimitOrderResult, intent_id: {}, result: {}, message: {:?}",
				params.intent_id, params.result, params.message
			);

			let Ok(address) = Address32::from_hex(&user.omni_account) else {
				error!("Failed to parse from omni account token");
				return Err(PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason("Failed to parse omni account from authentication token"),
				));
			};
			let omni_account = AccountId::from(address);

			let result = handle_pumpx_notify_limit_order_result(
				ctx.to_task_handler_context(),
				omni_account,
				params.intent_id,
				params.result,
				params.message,
				user.client_id,
			)
			.await;

			match result {
				Ok(NativeTaskOk::PumpxNotifyLimitOrderResult) => Ok(()),
				Ok(_) => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from(
						DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
							.with_reason("Unexpected response type from native task handler"),
					))
				},
				Err(NativeTaskError::PumpxApiError(api_error)) => {
					error!("Pumpx API error: {:?}", api_error);
					match api_error {
						native_task_handler::PumpxApiError::InvalidInput => {
							Err(PumpxRpcError::from(
								DetailedError::new(INVALID_PARAMS_CODE, "Invalid input")
									.with_reason("Result must be 'ok' or 'nok'"),
							))
						},
						_ => Err(PumpxRpcError::from(
							DetailedError::new(INTERNAL_ERROR_CODE, "Pumpx API error")
								.with_reason(format!("{:?}", api_error)),
						)),
					}
				},
				Err(e) => {
					error!("Failed to notify limit order result: {:?}", e);
					Err(PumpxRpcError::from(
						DetailedError::new(
							INTERNAL_ERROR_CODE,
							"Failed to notify limit order result",
						)
						.with_reason(format!("{:?}", e)),
					))
				},
			}
		})
		.expect("Failed to register omni_notifyLimitOrderResult method");
}
