mod get_shielding_key;

use crate::server::RpcContext;
use get_shielding_key::register_get_shielding_key;
use jsonrpsee::RpcModule;

pub fn register_methods(module: &mut RpcModule<RpcContext>) {
	register_get_shielding_key(module);
}
