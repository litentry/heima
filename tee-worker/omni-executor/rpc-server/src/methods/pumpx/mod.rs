use crate::server::RpcContext;
use jsonrpsee::RpcModule;

mod request_jwt;
use request_jwt::*;

mod export_wallet;
use export_wallet::*;

mod add_wallet;
use add_wallet::*;

pub fn register_pumpx(module: &mut RpcModule<RpcContext>) {
	register_request_jwt(module);
	register_export_wallet(module);
	register_add_wallet(module);
}
