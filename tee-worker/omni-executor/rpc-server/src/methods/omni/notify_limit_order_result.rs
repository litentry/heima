use crate::methods::omni::check_auth;
use crate::{
	detailed_error::DetailedError,
	error_code::{PARSE_ERROR_CODE, *},
	server::RpcContext,
	Deserialize,
};
use executor_core::intent_executor::IntentExecutor;
use jsonrpsee::RpcModule;
use tracing::{debug, error, info};

#[derive(Debug, Deserialize)]
pub struct NotifyLimitOrderResultParams {
	pub intent_id: u32,
	pub result: String,
	pub message: Option<String>,
}

pub fn register_notify_limit_order_result<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_notifyLimitOrderResult", |params, _ctx, ext| async move {
			let _user = check_auth(&ext).map_err(|e| {
				error!("Authentication check failed: {:?}", e);
				DetailedError::new(
					AUTH_VERIFICATION_FAILED_CODE,
					"Authentication verification failed",
				)
				.with_suggestion("Please check your authentication credentials")
			})?;

			let params = params.parse::<NotifyLimitOrderResultParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				DetailedError::new(PARSE_ERROR_CODE, "Parse error")
					.with_reason("Invalid JSON format or missing required fields")
			})?;

			debug!(
				"Received omni_notifyLimitOrderResult, intent_id: {}, result: {}, message: {:?}",
				params.intent_id, params.result, params.message
			);

			// Inline handle_pumpx_notify_limit_order_result logic
			if params.result != "ok" && params.result != "nok" {
				error!("Invalid result value: {}. Must be 'ok' or 'nok'", params.result);
				return Err(DetailedError::new(INVALID_PARAMS_CODE, "Invalid input")
					.with_reason("Result must be 'ok' or 'nok'")
					.into());
			}

			if let Some(msg) = &params.message {
				info!("Limit order result message for intent_id {}: {}", params.intent_id, msg);
			}

			Ok(())
		})
		.expect("Failed to register omni_notifyLimitOrderResult method");
}
