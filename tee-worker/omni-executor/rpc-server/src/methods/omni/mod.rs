use crate::detailed_error::DetailedError;
use crate::server::RpcContext;
use crate::RpcResult;
use jsonrpsee::RpcModule;
use parity_scale_codec::Codec;
use pumpx::methods::common::ApiResponse;
use tracing::error;

mod get_health;
use get_health::*;

mod get_next_intent_id;
use get_next_intent_id::*;

mod get_oauth2_authorization_data;
use get_oauth2_authorization_data::*;

mod get_shielding_key;
use get_shielding_key::*;

mod request_email_verification_code;
use request_email_verification_code::*;

mod get_web3_sign_in_message;
use get_web3_sign_in_message::*;

mod add_wallet;
use add_wallet::*;
use executor_core::intent_executor::IntentExecutor;

mod export_wallet;
use export_wallet::*;

mod export_bundler_private_key;
use export_bundler_private_key::*;

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

mod get_smart_wallet_root_signer;
use get_smart_wallet_root_signer::*;

mod estimate_user_op_gas;
use estimate_user_op_gas::*;

mod submit_user_op_with_auth;
use submit_user_op_with_auth::*;

mod user_login;
use user_login::*;

mod login_with_oauth2;
use login_with_oauth2::*;

mod get_hyperliquid_signature_data;
use get_hyperliquid_signature_data::*;

mod attach_passkey;
use attach_passkey::*;

mod remove_passkey;
use remove_passkey::*;

mod list_passkey;
use list_passkey::*;

mod request_passkey_challenge;
use request_passkey_challenge::*;

mod rename_passkey_alias;
use rename_passkey_alias::*;

#[cfg(test)]
mod test_protected_method;

#[cfg(feature = "test-endpoints")]
mod submit_user_op_test;
#[cfg(feature = "test-endpoints")]
use submit_user_op_test::*;

#[cfg(feature = "test-endpoints")]
mod verify_email_verification_code_test;
#[cfg(feature = "test-endpoints")]
use verify_email_verification_code_test::*;

#[cfg(feature = "test-endpoints")]
mod request_loan_test;
#[cfg(feature = "test-endpoints")]
use request_loan_test::*;

#[cfg(feature = "test-endpoints")]
mod query_loan_test;
#[cfg(feature = "test-endpoints")]
use query_loan_test::*;

#[cfg(feature = "test-endpoints")]
mod payback_loan_test;
#[cfg(feature = "test-endpoints")]
use payback_loan_test::*;

#[cfg(feature = "test-endpoints")]
mod open_position_test;
#[cfg(feature = "test-endpoints")]
use open_position_test::*;

#[cfg(feature = "test-endpoints")]
mod close_position_test;
#[cfg(feature = "test-endpoints")]
use close_position_test::*;

pub fn register_omni<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	register_get_health(module);
	register_get_next_intent_id(module);
	register_get_shielding_key(module);
	register_request_email_verification_code(module);
	register_get_oauth2_authorization_data(module);
	register_get_web3_sign_in_message(module);
	register_user_login(module);
	register_request_passkey_challenge(module);
	register_attach_passkey(module);
	register_remove_passkey(module);
	register_list_passkey(module);
	register_rename_passkey_alias(module);
	register_login_with_oauth2(module);

	register_request_jwt(module);
	register_export_wallet(module);
	register_export_bundler_private_key(module);
	register_add_wallet(module);
	register_transfer_withdraw(module);
	register_submit_swap_order(module);
	register_sign_limit_order_params(module);
	register_notify_limit_order_result(module);
	register_get_smart_wallet_root_signer(module);
	register_estimate_user_op_gas(module);
	register_submit_user_op_with_auth(module);
	register_get_hyperliquid_signature_data(module);

	#[cfg(test)]
	test_protected_method::register_test_protected_method(module);

	#[cfg(feature = "test-endpoints")]
	register_submit_user_op_test(module);

	#[cfg(feature = "test-endpoints")]
	register_verify_email_verification_code_test(module);

	#[cfg(feature = "test-endpoints")]
	register_request_loan_test(module);

	#[cfg(feature = "test-endpoints")]
	register_query_loan_test(module);

	#[cfg(feature = "test-endpoints")]
	register_payback_loan_test(module);

	#[cfg(feature = "test-endpoints")]
	register_open_position_test(module);

	#[cfg(feature = "test-endpoints")]
	register_close_position_test(module);
}

pub fn check_backend_response<T: Codec>(response: &ApiResponse<T>, op: &str) -> RpcResult<()> {
	if response.code != 10000 {
		error!("{} failed: code={}, message={}", op, response.code, response.message);
		return Err(DetailedError::invalid_backend_response(response, op).to_rpc_error());
	}
	Ok(())
}
