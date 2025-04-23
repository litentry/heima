use crate::server::RpcContext;
use jsonrpsee::RpcModule;

mod common;
use common::*;

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

mod get_message_code;
use get_message_code::*;

mod add_wallet;
use add_wallet::*;

mod export_wallet;
use export_wallet::*;

mod notify_limit_order_result;
use notify_limit_order_result::*;

mod request_jwt;
use request_jwt::*;

mod sign_limit_order;
use sign_limit_order::*;

mod submit_swap_order;
use submit_swap_order::*;

mod transfer_widthdraw;
use transfer_widthdraw::*;

pub fn register_omni(module: &mut RpcModule<RpcContext>) {
	register_get_health(module);
	register_get_next_intent_id(module);
	register_get_shielding_key(module);
	register_submit_native_task(module);
	register_request_email_verification_code(module);
	register_get_oauth2_google_authorization_url(module);
	register_get_message_code(module);

	register_request_jwt(module);
	register_export_wallet(module);
	register_add_wallet(module);
	register_transfer_withdraw(module);
	register_submit_swap_order(module);
	register_sign_limit_order_params(module);
	register_notify_limit_order_result(module);
}
