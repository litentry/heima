use crate::server::RpcContext;
use jsonrpsee::RpcModule;

mod request_jwt;
use request_jwt::*;

mod export_wallet;
use export_wallet::*;

mod add_wallet;
use add_wallet::*;

mod transfer_widthdraw;
use transfer_widthdraw::*;

mod submit_swap_order;
use submit_swap_order::*;

mod get_next_intent_id;
use get_next_intent_id::*;

pub fn register_pumpx(module: &mut RpcModule<RpcContext>) {
	register_request_jwt(module);
	register_export_wallet(module);
	register_add_wallet(module);
	register_transfer_withdraw(module);
	register_submit_swap_order(module);
	register_get_next_intent_id(module);
}
