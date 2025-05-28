use super::common::{check_omni_api_response, handle_omni_native_task};
use crate::{
	error_code::*, methods::omni::PumpxRpcError, server::RpcContext, verify_auth::verify_auth,
	Deserialize, ErrorCode,
};
use executor_core::native_task::*;
use executor_primitives::UserInfo;
use heima_primitives::Identity;
use jsonrpsee::RpcModule;
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct UserLoginParams {
	pub user_info: UserInfo,
}

impl TryFrom<UserLoginParams> for NativeTaskWrapper<NativeTask> {
	type Error = PumpxRpcError;

	fn try_from(p: UserLoginParams) -> Result<Self, Self::Error> {
		let user_info = p.user_info;
		let user_identity = Identity::try_from(user_info.id).map_err(|_| {
			error!("Invalid identity format");
			PumpxRpcError::from_error_code(ErrorCode::ParseError)
		})?;

		todo!()
	}
}

pub fn register_user_login(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_userLogin", |params, ctx, _| async move {
			todo!("Implement user login logic");
		})
		.expect("Failed to register omni_requestJwt method");
}
