use crate::{server::RpcContext, Deserialize, Serialize};
use executor_core::native_task::{NativeTask, NativeTaskWrapper};
use executor_primitives::OmniAuth;
use heima_primitives::{Identity, IdentityString};
use jsonrpsee::{types::ErrorObject, RpcModule};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestJwtParams {
	pub user_email: String,
	pub invite_code: Option<String>,
	pub google_code: Option<String>,
	pub email_code: String,
}

impl From<RequestJwtParams> for NativeTaskWrapper<NativeTask> {
	fn from(p: RequestJwtParams) -> Self {
		Self {
			task: NativeTask::PumpxRequestJwt(
				Identity::Email(IdentityString::new(p.user_email.as_bytes().to_vec())),
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
		.register_method("pumpx_requestJwt", |_, _, _| Ok::<String, ErrorObject>("OK".to_string()))
		.expect("Failed to register getHealth method");
}
