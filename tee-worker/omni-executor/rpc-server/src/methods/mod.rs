use crate::server::RpcContext;
use executor_core::intent_executor::IntentExecutor;
use jsonrpsee::RpcModule;

mod omni;
use omni::*;

pub const PROTECTED_METHODS: [&str; 9] = [
	"omni_testProtectedMethod",
	"omni_addWallet",
	"omni_notifyLimitOrderResult",
	"omni_signLimitOrder",
	"omni_submitSwapOrder",
	"omni_submitUserOp",
	"omni_transferWithdraw",
	"omni_exportWallet",
	"omni_requestLoan",
];

pub fn register_methods<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>,
	>,
) {
	register_omni(module);
}
