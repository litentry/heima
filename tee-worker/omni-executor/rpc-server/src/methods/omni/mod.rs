use crate::server::RpcContext;
use jsonrpsee::RpcModule;

mod common;
use common::*;

mod get_health;
use get_health::*;

mod get_next_intent_id;
use get_next_intent_id::*;

mod get_oauth2_google_authorization_url;
use get_oauth2_google_authorization_url::*;

mod get_shielding_key;
use get_shielding_key::*;

mod request_email_verification_code;
use request_email_verification_code::*;

mod submit_native_task;
use submit_native_task::*;

mod get_web3_sign_in_message;
use get_web3_sign_in_message::*;

mod add_wallet;
use add_wallet::*;
use executor_core::intent_executor::IntentExecutor;

mod export_wallet;
use export_wallet::*;

mod notify_limit_order_result;
use notify_limit_order_result::*;

mod request_jwt;
use request_jwt::*;

mod sign_limit_order;
use sign_limit_order::*;

mod submit_swap_order;
use submit_swap_order::*;

mod transfer_widthdraw;
use transfer_widthdraw::*;

mod get_omni_account;
use get_omni_account::*;

mod get_smart_wallet_root_signer;
use get_smart_wallet_root_signer::*;

mod submit_user_op;
use submit_user_op::*;

mod estimate_user_op_gas;
use estimate_user_op_gas::*;

mod submit_user_op_with_auth;
use submit_user_op_with_auth::*;

mod user_login;
use user_login::*;

mod get_hyperliquid_signature_data;
use get_hyperliquid_signature_data::*;

#[cfg(test)]
mod test_protected_method;

#[cfg(feature = "test-endpoints")]
mod submit_user_op_test;
#[cfg(feature = "test-endpoints")]
use submit_user_op_test::*;

pub fn register_omni<
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
	register_get_health(module);
	register_get_next_intent_id(module);
	register_get_shielding_key(module);
	register_submit_native_task(module);
	register_request_email_verification_code(module);
	register_get_oauth2_google_authorization_url(module);
	register_get_web3_sign_in_message(module);
	register_user_login(module);

	register_request_jwt(module);
	register_export_wallet(module);
	register_add_wallet(module);
	register_transfer_withdraw(module);
	register_submit_swap_order(module);
	register_sign_limit_order_params(module);
	register_notify_limit_order_result(module);
	register_get_omni_account(module);
	register_get_smart_wallet_root_signer(module);
	register_submit_user_op(module);
	register_estimate_user_op_gas(module);
	register_submit_user_op_with_auth(module);
	register_get_hyperliquid_signature_data(module);

	#[cfg(test)]
	test_protected_method::register_test_protected_method(module);

	#[cfg(feature = "test-endpoints")]
	register_submit_user_op_test(module);
}
