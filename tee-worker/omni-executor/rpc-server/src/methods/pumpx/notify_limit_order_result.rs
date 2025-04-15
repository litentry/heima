use crate::{error_code::*, oneshot, server::RpcContext, Decode, Deserialize, ErrorCode};
use executor_core::native_task::*;
use executor_crypto::jwt;
use executor_primitives::{utils::hex::FromHexPrefixed, OmniAuth};
use heima_authentication::auth_token::AuthTokenClaims;
use heima_primitives::{Address32, Identity};
use jsonrpsee::{types::ErrorObject, RpcModule};
use native_task_handler::{NativeTaskError, NativeTaskOk, NativeTaskResponse};
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs1::EncodeRsaPublicKey;
use rsa::RsaPrivateKey;

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
			let internal_error: ErrorObject = ErrorCode::InternalError.into();
			let params = params.parse::<NotifyLimitOrderResultParams>()?;

			log::debug!(
				"Received pumpx_notifyLimitOrderResult, intent_id: {}, result: {}, message: {:?}",
				params.intent_id,
				params.result,
				params.message
			);

			let private_key = RsaPrivateKey::from_pkcs1_der(&ctx.jwt_rsa_private_key)
				.map_err(|_| internal_error.clone())?;
			let public_key =
				private_key.to_public_key().to_pkcs1_der().map_err(|_| internal_error.clone())?;

			// this validates jwt - we skip exp check for this call
			let Ok(token) =
				jwt::decode::<AuthTokenClaims>(&params.auth_token, public_key.as_bytes(), true)
			else {
				return Err(ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE).into());
			};
			if token.typ != "access" {
				return Err(ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE).into());
			}

			let omni_account = token.sub;
			let Ok(address) = Address32::from_hex(&omni_account) else {
				log::error!("Not a valid address");
				return Err(internal_error);
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

			let (response_sender, response_receiver) = oneshot::channel();

			if ctx.native_task_sender.send((wrapper, response_sender)).await.is_err() {
				log::error!("Failed to send request to native call executor");
				return Err(ErrorCode::InternalError.into());
			}

			match response_receiver.await {
				Ok(response) => {
					let native_task_response: NativeTaskResponse =
						Decode::decode(&mut response.as_slice())
							.map_err(|_| internal_error.clone())?;
					match native_task_response {
						Ok(NativeTaskOk::PumpxNotifyLimitOrderResult) => Ok(()),
						Err(NativeTaskError::InternalError) => {
							log::error!("Internal error in native task");
							Err(internal_error)
						},
						Err(native_task_error) => {
							log::error!("Native task error: {:?}", native_task_error);
							Err(ErrorCode::ServerError(get_native_task_error_code(
								&native_task_error,
							))
							.into())
						},
						_ => {
							log::error!("Unexpected response type");
							Err(internal_error)
						},
					}
				},
				Err(e) => {
					log::error!("Failed to receive response from native call handler: {:?}", e);
					Err(internal_error)
				},
			}
		})
		.expect("Failed to register pumpx_notifyLimitOrderResult method");
}
