use crate::{
	error_code::*, methods::pumpx::PumpxRpcError, server::RpcContext, verify_auth::verify_auth,
	Deserialize, ErrorCode,
};
use executor_core::native_task::*;
use executor_primitives::OmniAuth;
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use pumpx::methods::user_connect::UserConnectResponse;
use serde::Serialize;
use tracing::{debug, error};

use super::common::{check_pumpx_api_response, handle_pumpx_native_task};

#[derive(Debug, Deserialize)]
pub struct RequestJwtParams {
	pub user_email: String,
	pub client_id: String,
	pub invite_code: Option<String>,
	pub google_code: String,
	pub language: Option<String>,
	pub email_code: String,
}

#[derive(Serialize, Clone)]
pub struct RequestJwtResponse {
	pub access_token: String,
	pub id_token: String,
	pub backend_response: UserConnectResponse,
}

impl From<RequestJwtParams> for NativeTaskWrapper<NativeTask> {
	fn from(p: RequestJwtParams) -> Self {
		NativeTaskWrapper::new(
			NativeTask::PumpxRequestJwt(
				Identity::from_web2_account(p.user_email.as_str(), Web2IdentityType::Email), // actually unused
				p.user_email.clone(),
				p.invite_code,
				p.google_code,
				p.language,
			),
			None,
			Some(OmniAuth::Email(p.client_id.clone(), p.user_email, p.email_code)),
		)
	}
}

pub fn register_request_jwt(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("pumpx_requestJwt", |params, ctx, _| async move {
			let params = params.parse::<RequestJwtParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!("Received pumpx_requestJwt, user_email: {}", params.user_email);

			let wrapper: NativeTaskWrapper<NativeTask> = params.into();

			if wrapper.task.require_auth() {
				let Some(ref auth) = wrapper.auth else {
					error!("Missing auth");
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
				NativeTaskOk::PumpxRequestJwt { access_token, id_token, backend_response } => {
					check_pumpx_api_response(backend_response.clone(), "Request pumpx jwt".into())?;
					Ok(RequestJwtResponse { access_token, id_token, backend_response })
				},
				_ => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register pumpx_requestJwt method");
}
