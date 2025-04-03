use crate::server::RpcContext;
use jsonrpsee::RpcModule;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};

mod get_health;
use get_health::*;

mod get_oauth2_google_authorization_url;
use get_oauth2_google_authorization_url::*;

mod get_shielding_key;
use get_shielding_key::*;

mod request_email_verification_code;
use request_email_verification_code::*;

mod submit_native_task;
use submit_native_task::*;

pub fn register_omni<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<Header, RpcClient, RpcClientFactory>>,
) {
	register_get_health(module);
	register_get_shielding_key(module);
	register_submit_native_task(module);
	register_request_email_verification_code(module);
	register_get_oauth2_google_authorization_url(module);
}
