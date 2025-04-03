use crate::{
	error_code::*, hex_encode, oneshot, server::RpcContext, verify_auth::verify_auth, Deserialize,
	ErrorCode,
};
use executor_core::native_task::*;
use executor_primitives::OmniAuth;
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::{types::ErrorObject, RpcModule};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};

#[derive(Debug, Deserialize)]
pub struct RequestJwtParams {
	pub user_email: String,
	pub invite_code: Option<String>,
	pub google_code: MaybeGoogleCode,
	pub email_code: String,
}

impl From<RequestJwtParams> for NativeTaskWrapper<NativeTask> {
	fn from(p: RequestJwtParams) -> Self {
		Self {
			task: NativeTask::PumpxRequestJwt(
				Identity::from_web2_account(p.user_email.as_str(), Web2IdentityType::Email),
				p.invite_code,
				p.google_code,
				Some("en".to_string()),
			),
			nonce: None,
			auth: Some(OmniAuth::Email(p.email_code)),
		}
	}
}

pub fn register_request_jwt<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<Header, RpcClient, RpcClientFactory>>,
) {
	module
		.register_async_method("pumpx_requestJwt", |params, ctx, _| async move {
			let params = params.parse::<RequestJwtParams>()?;
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
				// TODO: we should use json field for better readability, either a defined structure, or serde_json::Value
				Ok(response) => Ok::<String, ErrorObject>(hex_encode(response.as_slice())),
				Err(e) => {
					log::error!("Failed to receive response from native call handler: {:?}", e);
					Err(ErrorCode::InternalError.into())
				},
			}
		})
		.expect("Failed to register pumpx_requestJwt method");
}
