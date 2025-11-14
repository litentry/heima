use crate::{
	server::RpcContext, utils::types::RpcResultExt, utils::validation::parse_rpc_params,
	Deserialize, Serialize,
};

use jsonrpsee::{types::ErrorObject, RpcModule};
use oe_core::intent::executor::IntentExecutor;
use oe_primitives::UserId;
use oe_storage::PasskeyStorage;
use tracing::debug;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ListPasskeyParams {
	pub user_id: UserId,
	pub client_id: String,
}

#[derive(Serialize, Clone)]
pub struct PasskeyInfo {
	pub credential_id: String,
	pub alias_name: String,
	pub created_at: u64,
	pub last_used: u64,
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
			let params = parse_rpc_params::<ListPasskeyParams>(params)?;

			debug!("Received omni_listPasskey, params: {:?}", params);

			let omni_account = params
				.user_id
				.to_omni_account(&params.client_id)
				.map_err_parse("Failed to convert to omni_account")?;

			let passkey_storage = PasskeyStorage::new(ctx.storage_db.clone());

			let passkeys = passkey_storage
				.list_passkeys(&omni_account)
				.map_err_internal("Failed to list passkeys")?;

			let passkey_infos = passkeys
				.into_iter()
				.map(|(credential_id, alias_name, created_at, last_used)| PasskeyInfo {
					credential_id,
					alias_name,
					created_at,
					last_used,
				})
				.collect();

			Ok::<ListPasskeyResponse, ErrorObject>(ListPasskeyResponse { passkeys: passkey_infos })
		})
		.expect("Failed to register omni_listPasskey method");
}
