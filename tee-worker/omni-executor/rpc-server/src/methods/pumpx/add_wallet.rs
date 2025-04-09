use crate::{
	error_code::*, oneshot, server::RpcContext, verify_auth::verify_auth, Deserialize, ErrorCode,
};
use executor_core::native_task::*;
use executor_primitives::OmniAuth;
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::{types::ErrorObject, RpcModule};

#[derive(Debug, Deserialize)]
pub struct AddWalletParams {
	pub user_email: String,
	pub auth_token: String,
}

impl From<AddWalletParams> for NativeTaskWrapper<NativeTask> {
	fn from(p: AddWalletParams) -> Self {
		Self {
			task: NativeTask::PumpxAddWallet(Identity::from_web2_account(
				p.user_email.as_str(),
				Web2IdentityType::Email,
			)),
			nonce: None,
			auth: Some(OmniAuth::AuthToken(p.auth_token)),
		}
	}
}

pub fn register_add_wallet(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("pumpx_addWallet", |params, ctx, _| async move {
			let params = params.parse::<AddWalletParams>()?;

			let wrapper: NativeTaskWrapper<NativeTask> = params.into();

			// Verify JWT auth
			if wrapper.task.require_auth() && verify_auth(ctx.clone(), &wrapper).await.is_err() {
				return Err(ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE).into());
			}

			let (response_sender, response_receiver) = oneshot::channel();

			if ctx.native_task_sender.send((wrapper, response_sender)).await.is_err() {
				log::error!("Failed to send request to native call executor");
				return Err(ErrorCode::InternalError.into());
			}

			match response_receiver.await {
				Ok(_response) => Ok::<(), ErrorObject>(()),
				Err(e) => {
					log::error!("Failed to receive response from native call handler: {:?}", e);
					Err(ErrorCode::InternalError.into())
				},
			}
		})
		.expect("Failed to register pumpx_addWallet method");
}
