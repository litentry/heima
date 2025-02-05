mod get_oauth2_google_authorization_url;
mod get_shielding_key;
mod request_email_verification_code;
mod submit_aes_request;
mod submit_plain_request;

use crate::server::RpcContext;
use get_oauth2_google_authorization_url::register_get_oauth2_google_authorization_url;
use get_shielding_key::register_get_shielding_key;
use jsonrpsee::RpcModule;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use request_email_verification_code::register_request_email_verification_code;
use submit_aes_request::register_submit_aes_request;
use submit_plain_request::register_submit_plain_request;

pub fn register_methods<
	AccountId: Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<AccountId, Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<AccountId, Header, RpcClient, RpcClientFactory>>,
) {
	register_get_shielding_key(module);
	register_submit_aes_request(module);
	register_request_email_verification_code(module);
	register_get_oauth2_google_authorization_url(module);
	register_submit_plain_request(module);
}
