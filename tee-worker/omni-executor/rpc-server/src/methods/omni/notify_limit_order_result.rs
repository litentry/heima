use crate::{
	detailed_error::DetailedError, server::RpcContext, utils::omni::extract_omni_account,
	utils::validation::parse_rpc_params, Deserialize,
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
			debug!("Received omni_notifyLimitOrderResult, params: {:?}", params);

			let params = parse_rpc_params::<NotifyLimitOrderResultParams>(params)?;
			let _ = extract_omni_account(&ext)?;

			// Inline handle_pumpx_notify_limit_order_result logic
			if params.result != "ok" && params.result != "nok" {
				error!("Invalid result value: {}. Must be 'ok' or 'nok'", params.result);
				return Err(
					DetailedError::invalid_params("result", "must be ok or nok").to_rpc_error()
				);
			}

			if let Some(msg) = &params.message {
				info!("Limit order result message for intent_id {}: {}", params.intent_id, msg);
			}

			Ok(())
		})
		.expect("Failed to register omni_notifyLimitOrderResult method");
}
