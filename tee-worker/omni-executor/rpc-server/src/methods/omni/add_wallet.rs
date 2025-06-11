use super::common::{check_omni_api_response, handle_omni_native_task};
use crate::{
	error_code::*,
	methods::omni::{common::check_auth, PumpxRpcError},
	server::RpcContext,
	Deserialize, ErrorCode,
};
use executor_core::native_task::*;
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use pumpx::methods::add_wallet::AddWalletResponse;
use serde::Serialize;
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct AddWalletParams {
	pub user_email: String,
}

#[derive(Serialize, Clone)]
pub struct RPCAddWalletResponse {
	pub backend_response: AddWalletResponse,
}

impl AddWalletParams {
	pub fn into_native_task_wrapper(self, client_id: String) -> NativeTaskWrapper<NativeTask> {
		let sender = Identity::from_web2_account(self.user_email.as_str(), Web2IdentityType::Email);
		NativeTaskWrapper::new(NativeTask::PumpxAddWallet(sender.clone()), None, None, client_id)
	}
}

pub fn register_add_wallet(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_addWallet", |params, ctx, ext| async move {
			let user = check_auth(&ext).map_err(|e| {
				error!("Authentication check failed: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ServerError(
					AUTH_VERIFICATION_FAILED_CODE,
				))
			})?;
			let params = params.parse::<AddWalletParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!("Received omni_addWallet, user_email: {}", params.user_email);

			let wrapper = params.into_native_task_wrapper(user.client_id);

			handle_omni_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::PumpxAddWallet(response) => {
					check_omni_api_response(response.clone(), "Add wallet".into())?;
					Ok(RPCAddWalletResponse { backend_response: response })
				},
				_ => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register omni_addWallet method");
}
