use crate::server::RpcContext;
use executor_core::intent_executor::IntentExecutor;
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::RpcModule;

mod omni;
use omni::*;

pub type RpcResult<T> = Result<T, ErrorObjectOwned>;

pub const PROTECTED_METHODS: [&str; 8] = [
	"omni_testProtectedMethod",
	"omni_addWallet",
	"omni_notifyLimitOrderResult",
	"omni_signLimitOrder",
	"omni_submitSwapOrder",
	"omni_submitUserOp",
	"omni_transferWithdraw",
	"omni_exportWallet",
];

pub fn register_methods<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	register_omni(module);
}
