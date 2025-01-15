mod get_shielding_key;
mod submit_native_request;

use crate::server::RpcContext;
use get_shielding_key::register_get_shielding_key;
use jsonrpsee::RpcModule;
use submit_native_request::register_submit_native_request;

pub fn register_methods(module: &mut RpcModule<RpcContext>) {
	register_get_shielding_key(module);
	register_submit_native_request(module);
}
