use crate::methods::pumpx::PumpxRpcError;
use crate::verify_auth::verify_auth_token_authentication;
use crate::{error_code::*, server::RpcContext, Deserialize, ErrorCode};
use executor_core::native_task::*;
use executor_primitives::{utils::hex::FromHexPrefixed, OmniAuth};
use heima_authentication::constants::AUTH_TOKEN_ACCESS_TYPE;
use heima_primitives::{Address32, Identity};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use tracing::{debug, error};

use super::common::handle_pumpx_native_task;

#[derive(Debug, Deserialize)]
pub struct NotifyLimitOrderResultParams {
	pub intent_id: u32,
	pub result: String,
	pub message: Option<String>,
	pub auth_token: String,
}

pub fn register_notify_limit_order_result(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("pumpx_notifyLimitOrderResult", |params, ctx, _| async move {
			let params = params.parse::<NotifyLimitOrderResultParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!(
				"Received pumpx_notifyLimitOrderResult, intent_id: {}, result: {}, message: {:?}",
				params.intent_id, params.result, params.message
			);

			let (omni_account, client_id) = match verify_auth_token_authentication(
				&ctx.jwt_rsa_private_key,
				&params.auth_token,
				AUTH_TOKEN_ACCESS_TYPE,
				true,
			) {
				Ok(claims) => (claims.sub, claims.aud),
				Err(_) => {
					error!("Failed to verify auth token");
					return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
						AUTH_VERIFICATION_FAILED_CODE,
					)));
				},
			};

			let Ok(address) = Address32::from_hex(&omni_account) else {
				error!("Failed to parse from omni account token");
				return Err(PumpxRpcError::from_error_code(ErrorCode::InternalError));
			};

			let wrapper = NativeTaskWrapper::new(
				NativeTask::PumpxNotifyLimitOrderResult(
					Identity::Substrate(address),
					params.intent_id,
					params.result,
					params.message,
				),
				None,
				Some(OmniAuth::AuthToken(params.auth_token)),
				client_id,
			);

			handle_pumpx_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::PumpxNotifyLimitOrderResult => Ok(()),
				_ => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register pumpx_notifyLimitOrderResult method");
}
