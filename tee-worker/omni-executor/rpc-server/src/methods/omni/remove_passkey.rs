use crate::{
	detailed_error::DetailedError, server::RpcContext, utils::validation::parse_rpc_params,
	verify_auth::verify_auth, Deserialize, Serialize,
};

use executor_core::intent_executor::IntentExecutor;
use executor_primitives::{to_omni_auth, UserAuth, UserId};
use executor_storage::{PasskeyError, PasskeyStorage};
use heima_primitives::Identity;
use jsonrpsee::{types::ErrorObject, RpcModule};
use tracing::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RemovePasskeyParams {
	pub user_id: UserId,
	pub user_auth: UserAuth,
	pub client_id: String,
	pub credential_id: String,
}

#[derive(Serialize, Clone)]
pub struct RemovePasskeyResponse {
	pub success: bool,
	pub message: String,
}

pub fn register_remove_passkey<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_removePasskey", |params, ctx, _| async move {
			let params = parse_rpc_params::<RemovePasskeyParams>(params)?;

			debug!("Received omni_removePasskey, params: {:?}", params);

			let identity = Identity::try_from(params.user_id.clone()).map_err(|_| {
				error!("Invalid user ID format");
				DetailedError::parse_error("Invalid user ID format").to_rpc_error()
			})?;

			let auth = to_omni_auth(&params.user_auth, &params.user_id, &params.client_id)
				.map_err(|e| {
					error!("Failed to convert to OmniAuth: {:?}", e);
					DetailedError::parse_error("Failed to convert to OmniAuth").to_rpc_error()
				})?;

			verify_auth(ctx.clone(), &auth).await.map_err(|e| {
				error!("Failed to verify user authentication: {:?}", e);
				e.to_detailed_error().to_rpc_error()
			})?;

			let omni_account = identity.to_omni_account(&params.client_id);
			let passkey_storage = PasskeyStorage::new(ctx.storage_db.clone());

			if !passkey_storage.exists_passkey(&omni_account, &params.credential_id) {
				error!("Passkey not found for this account and credential");
				return Err(DetailedError::passkey_not_found(&params.credential_id).to_rpc_error());
			}

			passkey_storage
				.remove_passkey(&omni_account, &params.credential_id)
				.map_err(|e| match e {
					PasskeyError::StorageError => {
						error!("Failed to remove passkey: storage error");
						DetailedError::storage_service_error("passkey removal").to_rpc_error()
					},
					_ => {
						error!("Failed to remove passkey: {:?}", e);
						DetailedError::storage_service_error("passkey removal").to_rpc_error()
					},
				})?;

			Ok::<RemovePasskeyResponse, ErrorObject>(RemovePasskeyResponse {
				success: true,
				message: format!("Passkey {} successfully removed", params.credential_id),
			})
		})
		.expect("Failed to register omni_removePasskey method");
}
