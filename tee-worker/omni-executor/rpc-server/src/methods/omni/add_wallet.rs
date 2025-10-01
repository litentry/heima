use super::common::{check_omni_api_response, handle_omni_native_task};
use crate::{
	detailed_error::DetailedError,
	error_code::*,
	methods::omni::{common::check_auth, PumpxRpcError},
	server::RpcContext,
};
use executor_core::intent_executor::IntentExecutor;
use executor_core::native_task::*;
use executor_primitives::{utils::hex::FromHexPrefixed, AccountId};
use heima_primitives::Address32;
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
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

			let wrapper = NativeTaskWrapper::new(
				NativeTask::PumpxAddWallet(AccountId::from(address)),
				None,
				None,
				user.client_id,
			);

			handle_omni_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::PumpxAddWallet(response) => {
					check_omni_api_response(response.clone(), "Add wallet".into())?;
					Ok(RPCAddWalletResponse { backend_response: response })
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
		})
		.expect("Failed to register omni_addWallet method");
}
