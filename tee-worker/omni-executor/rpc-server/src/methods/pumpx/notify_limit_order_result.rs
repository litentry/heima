use super::common::handle_pumpx_native_task;
use crate::methods::pumpx::PumpxRpcError;
use crate::verify_auth::verify_auth_token_authentication;
use crate::{error_code::*, server::RpcContext, Deserialize, ErrorCode};
use executor_core::native_task::*;
use executor_primitives::{utils::hex::FromHexPrefixed, OmniAuth};
use heima_authentication::auth_token::AUTH_TOKEN_ACCESS_TYPE;
use heima_primitives::{Address32, Identity};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;

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
				log::error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			log::debug!(
				"Received pumpx_notifyLimitOrderResult, intent_id: {}, result: {}, message: {:?}",
				params.intent_id,
				params.result,
				params.message
			);

			let Ok(claims) = verify_auth_token_authentication(
				ctx.clone(),
				&params.auth_token,
				AUTH_TOKEN_ACCESS_TYPE,
				true, // we skip exp check for this call
			) else {
				log::error!("Failed to verify auth token");
				return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
					AUTH_VERIFICATION_FAILED_CODE,
				)));
			};

			let Ok(address) = Address32::from_hex(&claims.sub) else {
				log::error!("Failed to parse from omni account token");
				return Err(PumpxRpcError::from_error_code(ErrorCode::InternalError));
			};

			let wrapper = NativeTaskWrapper {
				task: NativeTask::PumpxNotifyLimitOrderResult(
					Identity::Substrate(address),
					params.intent_id,
					params.result,
					params.message,
				),
				nonce: None,
				auth: Some(OmniAuth::AuthToken(params.auth_token)),
			};

			handle_pumpx_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::PumpxNotifyLimitOrderResult => Ok(()),
				_ => {
					log::error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register pumpx_notifyLimitOrderResult method");
}
