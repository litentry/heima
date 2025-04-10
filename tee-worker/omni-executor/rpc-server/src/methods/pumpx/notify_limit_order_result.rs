use crate::{
	error_code::*, oneshot, server::RpcContext, verify_auth::verify_auth, Decode, Deserialize,
	ErrorCode,
};
use executor_core::native_task::*;
use executor_primitives::Hash;
use executor_primitives::OmniAuth;
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::{types::ErrorObject, RpcModule};
use native_task_handler::{NativeTaskError, NativeTaskOk, NativeTaskResponse};
use serde::Serialize;

#[derive(Debug, Deserialize)]
pub struct NotifyLimitOrderResultParams {
	pub user_email: String,
	pub intent_id: u32,
	pub result: String,
	pub message: Option<String>,
	pub auth_token: String,
}

#[derive(Serialize, Clone)]
pub struct RPCNotifyLimitOrderResultResponse {
	extrinsic_hash: Hash,
	block_hash: Option<Hash>,
}

impl From<NotifyLimitOrderResultParams> for NativeTaskWrapper<NativeTask> {
	fn from(p: NotifyLimitOrderResultParams) -> Self {
		Self {
			task: NativeTask::PumpxNotifyLimitOrderResult(
				Identity::from_web2_account(p.user_email.as_str(), Web2IdentityType::Email),
				p.intent_id,
				p.result,
				p.message,
			),
			nonce: None,
			auth: Some(OmniAuth::AuthToken(p.auth_token)),
		}
	}
}

pub fn register_notify_limit_order_result(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("pumpx_notifyLimitOrderResult", |params, ctx, _| async move {
			let internal_error: ErrorObject = ErrorCode::InternalError.into();
			let params = params.parse::<NotifyLimitOrderResultParams>()?;

			let wrapper: NativeTaskWrapper<NativeTask> = params.into();
			if wrapper.task.require_auth() && verify_auth(ctx.clone(), &wrapper).await.is_err() {
				return Err(ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE).into());
			}

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
						Ok(NativeTaskOk::ExtrinsicReport {
							extrinsic_hash,
							block_hash,
							status: _,
						}) => Ok(RPCNotifyLimitOrderResultResponse { extrinsic_hash, block_hash }),
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
