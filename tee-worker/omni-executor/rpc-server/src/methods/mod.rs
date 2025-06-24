use crate::server::RpcContext;
use jsonrpsee::RpcModule;

mod pumpx;
use pumpx::*;

mod omni;
use omni::*;

pub const PROTECTED_METHODS: [&str; 7] = [
	"omni_testProtectedMethod",
	"omni_addWallet",
	"omni_notifyLimitOrderResult",
	"omni_signLimitOrder",
	"omni_submitSwapOrder",
	"omni_transferWithdraw",
	"omni_exportWallet",
];

pub fn register_methods(module: &mut RpcModule<RpcContext>) {
	register_omni(module);
	register_pumpx(module);
}
