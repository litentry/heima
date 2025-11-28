use crate::{
	detailed_error::DetailedError, methods::omni::check_backend_response, server::RpcContext,
	utils::omni::extract_omni_account,
};
use jsonrpsee::RpcModule;
use oe_client_pumpx::methods::add_wallet::AddWalletResponse;
use oe_core::auth::constants::AUTH_TOKEN_ACCESS_TYPE;
use oe_core::intent::executor::IntentExecutor;
use oe_storage::{HeimaJwtStorage, Storage};
use serde::Serialize;
use tracing::{debug, error};

#[derive(Serialize, Clone)]
pub struct RPCAddWalletResponse {
	pub backend_response: AddWalletResponse,
}

pub fn register_add_wallet<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_addWallet", |_params, ctx, ext| async move {
			debug!("Received omni_addWallet");

			let omni_account = extract_omni_account(&ext)?;

			// Inlined handler logic from handle_pumpx_add_wallet
			let storage = HeimaJwtStorage::new(ctx.storage_db.clone());
			let Ok(Some(access_token)) = storage.get(&(omni_account, AUTH_TOKEN_ACCESS_TYPE))
			else {
				error!("Failed to get pumpx_{}_jwt_token", AUTH_TOKEN_ACCESS_TYPE);
				return Err(DetailedError::storage_service_error("get access token").to_rpc_error());
			};

			// Call Pumpx API to add wallet
			debug!("Calling pumpx add_wallet");
			let backend_response =
				ctx.pumpx_api.add_wallet(&access_token, None).await.map_err(|e| {
					DetailedError::pumpx_service_error("add_wallet", format!("{:?}", e))
				})?;

			check_backend_response(&backend_response, "add_wallet")?;
			Ok(RPCAddWalletResponse { backend_response })
		})
		.expect("Failed to register omni_addWallet method");
}
