use crate::server::RpcContext;
use jsonrpsee::RpcModule;

mod common;
use common::*;

mod request_jwt;
use request_jwt::*;

mod export_wallet;
use export_wallet::*;

mod add_wallet;
use add_wallet::*;
use executor_core::intent_executor::IntentExecutor;

mod transfer_widthdraw;
use transfer_widthdraw::*;

mod submit_swap_order;
use submit_swap_order::*;

mod get_next_intent_id;
use get_next_intent_id::*;

mod sign_limit_order;
use sign_limit_order::*;

mod notify_limit_order_result;
use notify_limit_order_result::*;

mod get_omni_account;
use get_omni_account::*;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};

pub fn register_pumpx<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<
			Header,
			RpcClient,
			RpcClientFactory,
			EthereumIntentExecutor,
			SolanaIntentExecutor,
			CrossChainIntentExecutor,
		>,
	>,
) {
	register_request_jwt(module);
	register_export_wallet(module);
	register_add_wallet(module);
	register_transfer_withdraw(module);
	register_submit_swap_order(module);
	register_get_next_intent_id(module);
	register_get_omni_account(module);
	register_sign_limit_order_params(module);
	register_notify_limit_order_result(module);
}
