use crate::{
	error_code::*, methods::pumpx::PumpxRpcError, server::RpcContext, verify_auth::verify_auth,
	Deserialize, ErrorCode,
};
use executor_core::native_task::*;
use executor_primitives::OmniAuth;
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use pumpx::methods::add_wallet::AddWalletResponse;
use serde::Serialize;
use tracing::{debug, error};

use super::common::{check_pumpx_api_response, handle_pumpx_native_task};

#[derive(Debug, Deserialize)]
pub struct AddWalletParams {
	pub client_id: String,
	pub user_id: String,
	pub auth_token: String,
}

#[derive(Serialize, Clone)]
pub struct RPCAddWalletResponse {
	pub backend_response: AddWalletResponse,
}

impl AddWalletParams {
	pub fn into_native_task_wrapper(self) -> NativeTaskWrapper<NativeTask> {
		let sender = Identity::from_web2_account(self.user_id.as_str(), Web2IdentityType::Pumpx);
		NativeTaskWrapper::new(
			NativeTask::PumpxAddWallet(sender.clone()),
			None,
			Some(OmniAuth::AuthToken(self.auth_token)),
			self.client_id,
		)
	}
}

pub fn register_add_wallet(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("pumpx_addWallet", |params, ctx, _ext| async move {
			let params = params.parse::<AddWalletParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!("Received pumpx_addWallet, user_id: {}", params.user_id);

			let wrapper = params.into_native_task_wrapper();

			if wrapper.task.require_auth() {
				let Some(ref auth) = wrapper.auth else {
					error!("Missing auth token");
					return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
						REQUIRE_AUTHENTICATION_CODE,
					)));
				};
				verify_auth(ctx.clone(), auth).await.map_err(|_| {
					error!("Failed to verify auth: {:?}", wrapper.auth);
					PumpxRpcError::from_error_code(ErrorCode::ServerError(
						AUTH_VERIFICATION_FAILED_CODE,
					))
				})?;
			}

			handle_pumpx_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::PumpxAddWallet(response) => {
					check_pumpx_api_response(response.clone(), "Add wallet".into())?;
					Ok(RPCAddWalletResponse { backend_response: response })
				},
				_ => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register pumpx_addWallet method");
}
