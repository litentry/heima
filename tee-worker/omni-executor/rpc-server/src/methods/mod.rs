use crate::server::RpcContext;
use jsonrpsee::RpcModule;

mod pumpx;
use pumpx::*;

mod omni;
use omni::*;

// TODO: list all protected methods here once the interface has been updated
pub const PROTECTED_METHODS: [&str; 1] = ["omni_testProtectedMethod"];

pub fn register_methods(module: &mut RpcModule<RpcContext>) {
	register_omni(module);
	register_pumpx(module);
}
