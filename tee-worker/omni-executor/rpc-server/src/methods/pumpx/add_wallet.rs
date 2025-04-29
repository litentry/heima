use crate::{
	error_code::*, methods::pumpx::PumpxRpcError, server::RpcContext, verify_auth::verify_auth,
	Deserialize, ErrorCode,
};
use executor_core::native_task::*;
use executor_primitives::OmniAuth;
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use pumpx::pumpx_types::add_wallet::AddWalletResponse;
use serde::Serialize;

use super::common::{check_pumpx_api_response, handle_pumpx_native_task};

#[derive(Debug, Deserialize)]
pub struct AddWalletParams {
	pub user_id: String,
	pub auth_token: String,
}

#[derive(Serialize, Clone)]
pub struct RPCAddWalletResponse {
	pub backend_response: AddWalletResponse,
}

impl From<AddWalletParams> for NativeTaskWrapper<NativeTask> {
	fn from(p: AddWalletParams) -> Self {
		Self {
			task: NativeTask::PumpxAddWallet(Identity::from_web2_account(
				p.user_id.as_str(),
				Web2IdentityType::Pumpx,
			)),
			nonce: None,
			auth: Some(OmniAuth::AuthToken(p.auth_token)),
		}
	}
}

pub fn register_add_wallet(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("pumpx_addWallet", |params, ctx, _| async move {
			let params = params.parse::<AddWalletParams>().map_err(|e| {
				log::error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			log::debug!("Received pumpx_addWallet, user_id: {}", params.user_id);

			let wrapper: NativeTaskWrapper<NativeTask> = params.into();

			if wrapper.task.require_auth() && verify_auth(ctx.clone(), &wrapper).await.is_err() {
				return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
					AUTH_VERIFICATION_FAILED_CODE,
				)));
			}

			handle_pumpx_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::PumpxAddWallet(response) => {
					check_pumpx_api_response(response.clone(), "Add wallet".into())?;
					Ok(RPCAddWalletResponse { backend_response: response })
				},
				_ => {
					log::error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register pumpx_addWallet method");
}
