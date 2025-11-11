use crate::{
	server::RpcContext, utils::types::RpcResultExt, utils::validation::parse_rpc_params,
	verify_auth::verify_auth, Deserialize, Serialize,
};

use executor_core::intent_executor::IntentExecutor;
use executor_primitives::{to_omni_auth, UserAuth, UserId};
use executor_storage::PasskeyStorage;
use jsonrpsee::{types::ErrorObject, RpcModule};
use tracing::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RenamePasskeyAliasParams {
	pub user_id: UserId,
	pub user_auth: UserAuth,
	pub client_id: String,
	pub credential_id: String,
	pub new_alias_name: String,
}

#[derive(Serialize, Clone)]
pub struct RenamePasskeyAliasResponse {
	pub success: bool,
	pub message: String,
}

pub fn register_rename_passkey_alias<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_renamePasskeyAliasName", |params, ctx, _| async move {
			let params = parse_rpc_params::<RenamePasskeyAliasParams>(params)?;

			debug!("Received omni_renamePasskeyAliasName, params: {:?}", params);

			let omni_account = params
				.user_id
				.to_omni_account(&params.client_id)
				.map_err_parse("Failed to convert to omni_account")?;

			let auth = to_omni_auth(&params.user_auth, &params.user_id, &params.client_id)
				.map_err_parse("Failed to convert to OmniAuth")?;

			verify_auth(ctx.clone(), &auth).await.map_err(|e| {
				error!("Failed to verify user authentication: {:?}", e);
				e.to_detailed_error().to_rpc_error()
			})?;

			let passkey_storage = PasskeyStorage::new(ctx.storage_db.clone());

			passkey_storage
				.rename_passkey_alias(
					&omni_account,
					&params.credential_id,
					params.new_alias_name.clone(),
				)
				.map_err_internal("Failed to rename passkey alias")?;

			Ok::<RenamePasskeyAliasResponse, ErrorObject>(RenamePasskeyAliasResponse {
				success: true,
				message: format!(
					"Passkey alias for credential {} successfully renamed to {}",
					params.credential_id, params.new_alias_name
				),
			})
		})
		.expect("Failed to register omni_renamePasskeyAliasName method");
}
