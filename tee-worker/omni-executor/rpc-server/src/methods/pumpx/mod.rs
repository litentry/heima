use crate::server::RpcContext;
use jsonrpsee::RpcModule;

mod request_jwt;
use request_jwt::*;

mod export_wallet;
use export_wallet::*;

mod submit_swap_order;
use submit_swap_order::*;

pub fn register_pumpx(module: &mut RpcModule<RpcContext>) {
	register_request_jwt(module);
	register_export_wallet(module);
	register_submit_swap_order(module);
}
