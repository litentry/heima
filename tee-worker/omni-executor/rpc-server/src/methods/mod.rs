use crate::server::RpcContext;
use jsonrpsee::RpcModule;

mod pumpx;
use pumpx::*;

mod omni;
use omni::*;

pub fn register_methods(module: &mut RpcModule<RpcContext>) {
	register_omni(module);
	register_pumpx(module);
}
