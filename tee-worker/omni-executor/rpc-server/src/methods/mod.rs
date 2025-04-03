mod get_health;
mod get_oauth2_google_authorization_url;
mod get_shielding_key;
mod request_email_verification_code;
mod submit_native_task;

use crate::server::RpcContext;
use get_health::register_get_health;
use get_oauth2_google_authorization_url::register_get_oauth2_google_authorization_url;
use get_shielding_key::register_get_shielding_key;
use jsonrpsee::RpcModule;
use request_email_verification_code::register_request_email_verification_code;
use submit_native_task::register_submit_native_task;

pub fn register_methods(module: &mut RpcModule<RpcContext>) {
	register_get_health(module);
	register_get_shielding_key(module);
	register_submit_native_task(module);
	register_request_email_verification_code(module);
	register_get_oauth2_google_authorization_url(module);
}
