use crate::{
	error_code::*, oneshot, server::RpcContext, verify_auth::verify_auth, Decode, Deserialize,
	ErrorCode,
};
use executor_core::native_task::*;
use executor_primitives::OmniAuth;
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::{types::ErrorObject, RpcModule};
use native_task_handler::{NativeTaskError, NativeTaskOk, NativeTaskResponse};
use pumpx::types::UserConnectResponse;
use serde::Serialize;

#[derive(Debug, Deserialize)]
pub struct RequestJwtParams {
	pub user_email: String,
	pub invite_code: Option<String>,
	pub google_code: MaybeGoogleCode,
	pub language: Option<String>,
	pub email_code: String,
}

#[derive(Serialize, Clone)]
pub struct RequestJwtResponse {
	pub access_token: String,
	pub id_token: String,
	pub user_connect_response: UserConnectResponse,
}

impl From<RequestJwtParams> for NativeTaskWrapper<NativeTask> {
	fn from(p: RequestJwtParams) -> Self {
		Self {
			task: NativeTask::PumpxRequestJwt(
				Identity::from_web2_account(p.user_email.as_str(), Web2IdentityType::Email),
				p.invite_code,
				p.google_code,
				p.language,
			),
			nonce: None,
			auth: Some(OmniAuth::Email(p.email_code)),
		}
	}
}

pub fn register_request_jwt(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("pumpx_requestJwt", |params, ctx, _| async move {
			let internal_error: ErrorObject = ErrorCode::InternalError.into();
			let params = params.parse::<RequestJwtParams>().map_err(|_| ErrorCode::ParseError)?;
			let wrapper: NativeTaskWrapper<NativeTask> = params.into();

			if wrapper.task.require_auth() && verify_auth(ctx.clone(), &wrapper).await.is_err() {
				return Err(ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE).into());
			}

			let (response_sender, response_receiver) = oneshot::channel();

			if ctx.native_task_sender.send((wrapper, response_sender)).await.is_err() {
				log::error!("Failed to send request to native call executor");
				return Err(internal_error);
			}
			match response_receiver.await {
				Ok(response) => {
					let native_task_response: NativeTaskResponse =
						Decode::decode(&mut response.as_slice())
							.map_err(|_| internal_error.clone())?;
					match native_task_response {
						Ok(NativeTaskOk::PumpxRequestJwt {
							access_token,
							id_token,
							user_connect_response,
						}) => {
							Ok(RequestJwtResponse { access_token, id_token, user_connect_response })
						},
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
		.expect("Failed to register pumpx_requestJwt method");
}
