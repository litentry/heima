use crate::server::RpcContext;
use jsonrpsee::RpcModule;

mod pumpx;
use pumpx::*;

mod omni;
use omni::*;

pub const PROTECTED_METHODS: [&str; 4] = [
	"omni_testProtectedMethod",
	"omni_addWallet",
	"omni_notifyLimitOrderResult",
	"omni_signLimitOrder",
];

pub fn register_methods(module: &mut RpcModule<RpcContext>) {
	register_omni(module);
	register_pumpx(module);
}
