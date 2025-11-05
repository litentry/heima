use crate::{detailed_error::DetailedError, server::RpcContext, Deserialize, ErrorCode, Serialize};

use executor_core::intent_executor::IntentExecutor;
use executor_primitives::UserId;
use executor_storage::PasskeyStorage;
use heima_primitives::Identity;
use jsonrpsee::{types::ErrorObject, RpcModule};
use tracing::error;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ListPasskeyParams {
	pub user_id: UserId,
	pub client_id: String,
}

#[derive(Serialize, Clone)]
pub struct PasskeyInfo {
	pub credential_id: String,
	pub created_at: u64,
}

#[derive(Serialize, Clone)]
pub struct ListPasskeyResponse {
	pub passkeys: Vec<PasskeyInfo>,
}

pub fn register_list_passkey<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_listPasskey", |params, ctx, _| async move {
			let params = params.parse::<ListPasskeyParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				ErrorCode::ParseError
			})?;

			let identity = Identity::try_from(params.user_id.clone()).map_err(|_| {
				error!("Invalid user ID format");
				ErrorCode::ParseError
			})?;

			let omni_account = identity.to_omni_account(&params.client_id);
			let passkey_storage = PasskeyStorage::new(ctx.storage_db.clone());

			let passkeys = passkey_storage.list_passkeys(&omni_account).map_err(|e| {
				error!("Failed to list passkeys: {:?}", e);
				DetailedError::storage_error("passkey listing").to_error_object()
			})?;

			let passkey_infos = passkeys
				.into_iter()
				.map(|(credential_id, created_at)| PasskeyInfo { credential_id, created_at })
				.collect();

			Ok::<ListPasskeyResponse, ErrorObject>(ListPasskeyResponse { passkeys: passkey_infos })
		})
		.expect("Failed to register omni_listPasskey method");
}
