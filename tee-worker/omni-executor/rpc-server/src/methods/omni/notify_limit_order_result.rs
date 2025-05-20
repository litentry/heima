use crate::methods::omni::PumpxRpcError;
use crate::{error_code::*, server::RpcContext, Deserialize, ErrorCode};
use executor_core::native_task::*;
use executor_crypto::jwt;
use executor_primitives::{utils::hex::FromHexPrefixed, OmniAuth};
use heima_authentication::auth_token::AuthTokenClaims;
use heima_primitives::{Address32, Identity};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs1::EncodeRsaPublicKey;
use rsa::RsaPrivateKey;
use tracing::{debug, error};

use super::common::handle_omni_native_task;

#[derive(Debug, Deserialize)]
pub struct NotifyLimitOrderResultParams {
	pub intent_id: u32,
	pub result: String,
	pub message: Option<String>,
	pub auth_token: String,
}

pub fn register_notify_limit_order_result(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_notifyLimitOrderResult", |params, ctx, _| async move {
			let params = params.parse::<NotifyLimitOrderResultParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!(
				"Received omni_notifyLimitOrderResult, intent_id: {}, result: {}, message: {:?}",
				params.intent_id, params.result, params.message
			);

			let private_key =
				RsaPrivateKey::from_pkcs1_der(&ctx.jwt_rsa_private_key).map_err(|e| {
					error!("Failed to parse private key: {:?}", e);
					PumpxRpcError::from_error_code(ErrorCode::InternalError)
				})?;

			let public_key = private_key.to_public_key().to_pkcs1_der().map_err(|e| {
				error!("Failed to generate public key: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::InternalError)
			})?;

			// this validates jwt - we skip exp check for this call
			let Ok(token) =
				jwt::decode::<AuthTokenClaims>(&params.auth_token, public_key.as_bytes(), true)
			else {
				return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
					AUTH_VERIFICATION_FAILED_CODE,
				)));
			};
			if token.typ != "access" {
				return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
					AUTH_VERIFICATION_FAILED_CODE,
				)));
			}

			let omni_account = token.sub;
			let Ok(address) = Address32::from_hex(&omni_account) else {
				error!("Failed to parse from omni account token");
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

			handle_omni_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::PumpxNotifyLimitOrderResult => Ok(()),
				_ => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register omni_notifyLimitOrderResult method");
}
