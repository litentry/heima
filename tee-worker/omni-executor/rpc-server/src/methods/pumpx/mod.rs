use crate::server::RpcContext;
use jsonrpsee::RpcModule;

mod request_jwt;
use request_jwt::*;

mod export_wallet;
use export_wallet::*;

mod get_next_intent_id;
use get_next_intent_id::*;

mod sign_limit_order;
use sign_limit_order::*;

pub fn register_pumpx(module: &mut RpcModule<RpcContext>) {
	register_request_jwt(module);
	register_export_wallet(module);
	register_get_next_intent_id(module);
	register_sign_limit_order_params(module);
}
