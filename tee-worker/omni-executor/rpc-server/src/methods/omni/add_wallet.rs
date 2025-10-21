use super::common::check_omni_api_response;
use crate::{
	detailed_error::DetailedError,
	error_code::*,
	methods::omni::{common::check_auth, PumpxRpcError},
	server::RpcContext,
};
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::{utils::hex::FromHexPrefixed, AccountId};
use executor_storage::{HeimaJwtStorage, Storage};
use heima_authentication::constants::AUTH_TOKEN_ACCESS_TYPE;
use heima_primitives::Address32;
use jsonrpsee::RpcModule;
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

			// Inlined handler logic from handle_pumpx_add_wallet
			let storage = HeimaJwtStorage::new(ctx.storage_db.clone());
			let Ok(Some(access_token)) = storage.get(&(omni_account, AUTH_TOKEN_ACCESS_TYPE))
			else {
				error!("Failed to get pumpx_{}_jwt_token", AUTH_TOKEN_ACCESS_TYPE);
				return Err(PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason("Failed to get access token"),
				));
			};

			// Call Pumpx API to add wallet
			debug!("Calling pumpx add_wallet");
			let backend_response =
				ctx.pumpx_api.add_wallet(&access_token, None).await.map_err(|e| {
					error!("Failed to add wallet through Pumpx API: {:?}", e);
					PumpxRpcError::from(
						DetailedError::new(
							PUMPX_API_ADD_WALLET_FAILED_CODE,
							"Failed to add wallet through Pumpx API",
						)
						.with_reason(format!("{:?}", e)),
					)
				})?;

			check_omni_api_response(backend_response.clone(), "Add wallet".into())?;
			Ok(RPCAddWalletResponse { backend_response })
		})
		.expect("Failed to register omni_addWallet method");
}
