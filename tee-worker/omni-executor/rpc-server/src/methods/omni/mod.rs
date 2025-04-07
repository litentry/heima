use crate::server::RpcContext;
use jsonrpsee::RpcModule;

mod get_health;
use get_health::*;

mod get_next_intent_id;
use get_next_intent_id::*;

mod get_oauth2_google_authorization_url;
use get_oauth2_google_authorization_url::*;

mod get_shielding_key;
use get_shielding_key::*;

mod request_email_verification_code;
use request_email_verification_code::*;

mod submit_native_task;
use submit_native_task::*;

pub fn register_omni(module: &mut RpcModule<RpcContext>) {
	register_get_health(module);
	register_get_next_intent_id(module);
	register_get_shielding_key(module);
	register_submit_native_task(module);
	register_request_email_verification_code(module);
	register_get_oauth2_google_authorization_url(module);
}
