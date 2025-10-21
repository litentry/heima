use super::common::check_omni_api_response;
use crate::{
	detailed_error::DetailedError,
	error_code::*,
	methods::omni::{common::check_auth, PumpxRpcError},
	server::RpcContext,
};
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::{utils::hex::FromHexPrefixed, AccountId};
use heima_primitives::Address32;
use jsonrpsee::RpcModule;
use native_task_handler::{handle_pumpx_add_wallet, NativeTaskError, NativeTaskOk};
use pumpx::methods::add_wallet::AddWalletResponse;
use serde::Serialize;
use tracing::{debug, error};

#[derive(Serialize, Clone)]
pub struct RPCAddWalletResponse {
	pub backend_response: AddWalletResponse,
}

pub fn register_add_wallet<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>,
	>,
) {
	module
		.register_async_method("omni_addWallet", |_params, ctx, ext| async move {
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

			debug!("Received omni_addWallet");

			let Ok(address) = Address32::from_hex(&user.omni_account) else {
				error!("Failed to parse from omni account token");
				return Err(PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason("Failed to parse omni account from authentication token"),
				));
			};
			let omni_account = AccountId::from(address);

			let result = handle_pumpx_add_wallet(
				ctx.to_task_handler_context(),
				omni_account,
				user.client_id,
			)
			.await;

			match result {
				Ok(NativeTaskOk::PumpxAddWallet(response)) => {
					check_omni_api_response(response.clone(), "Add wallet".into())?;
					Ok(RPCAddWalletResponse { backend_response: response })
				},
				Ok(_) => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from(
						DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
							.with_reason("Unexpected response type from native task handler"),
					))
				},
				Err(NativeTaskError::PumpxApiError(api_error)) => {
					error!("Pumpx API error: {:?}", api_error);
					Err(PumpxRpcError::from(
						DetailedError::new(INTERNAL_ERROR_CODE, "Pumpx API error")
							.with_reason(format!("{:?}", api_error)),
					))
				},
				Err(NativeTaskError::InternalError(msg)) => {
					error!("Internal error: {:?}", msg);
					Err(PumpxRpcError::from(
						DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(
							msg.unwrap_or_else(|| "Unknown internal error".to_string()),
						),
					))
				},
				Err(e) => {
					error!("Failed to add wallet: {:?}", e);
					Err(PumpxRpcError::from(
						DetailedError::new(INTERNAL_ERROR_CODE, "Failed to add wallet")
							.with_reason(format!("{:?}", e)),
					))
				},
			}
		})
		.expect("Failed to register omni_addWallet method");
}
