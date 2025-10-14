use super::common::handle_omni_native_task;
use crate::detailed_error::DetailedError;
use crate::error_code::{AUTH_VERIFICATION_FAILED_CODE, PARSE_ERROR_CODE};
use crate::methods::omni::common::check_auth;
use crate::methods::omni::PumpxRpcError;
use crate::server::RpcContext;
use crate::validation_helpers::{
	validate_chain_id, validate_omni_account_hex, validate_omni_account_length,
	validate_wallet_index,
};
use executor_core::intent_executor::IntentExecutor;
use executor_core::native_task::{NativeTask, NativeTaskWrapper};
use executor_primitives::AccountId;
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use parity_scale_codec::Decode;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct RequestLoanParams {
	pub chain_id: u64,
	pub wallet_index: u32,
	pub smart_wallet_address: String,
	pub collateral_ticker: String,
	pub lending_ratio: u32, // percentage of total collateral sold as spot and returned as lending, e.g., 50 = 50%
	pub collateral_size: String, // size (balance) of collateral in smart wallet spot
}

#[derive(Serialize, Clone)]
pub struct RequestLoanResponse {
	pub spot_sell_cloid: String,
	pub hedge_open_cloid: String,
	pub usdc_received: String,
	pub spot_sell_tx_hash: Option<String>,
	pub hedge_open_tx_hash: Option<String>,
}

pub fn register_request_loan<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>,
	>,
) {
	module
		.register_async_method("omni_requestLoan", |params, ctx, ext| async move {
			let user = check_auth(&ext).map_err(|e| {
				error!("Authentication check failed: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(AUTH_VERIFICATION_FAILED_CODE, "Authentication failed")
						.with_suggestion("Please provide valid authentication credentials"),
				)
			})?;

			let params = params.parse::<RequestLoanParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Failed to parse request parameters")
						.with_suggestion(format!("Invalid JSON structure: {}", e)),
				)
			})?;

			debug!("Received omni_requestLoan, params: {:?}", params);

			validate_chain_id(params.chain_id as u32, Some("evm")).map_err(PumpxRpcError::from)?;

			validate_wallet_index(params.wallet_index).map_err(PumpxRpcError::from)?;

			// Validate smart wallet address
			let smart_wallet_address_bytes =
				validate_omni_account_hex(&params.smart_wallet_address, "smart_wallet_address")
					.map_err(PumpxRpcError::from)?;

			if smart_wallet_address_bytes.len() != 20 {
				return Err(PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Smart wallet address must be 20 bytes")
						.with_field("smart_wallet_address")
						.with_received(format!("{} bytes", smart_wallet_address_bytes.len()))
						.with_expected("20 bytes (Ethereum address)"),
				));
			}

			// Validate collateral ticker is not empty
			if params.collateral_ticker.is_empty() {
				return Err(PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Collateral ticker cannot be empty")
						.with_field("collateral_ticker")
						.with_suggestion("Provide a valid ticker like ETH, PURR, etc."),
				));
			}

			// Validate lending_ratio is between 0 and 100 (0-100%)
			if params.lending_ratio > 100 {
				return Err(PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Lending ratio must be between 0 and 100")
						.with_field("lending_ratio")
						.with_received(params.lending_ratio.to_string())
						.with_expected("0-100 (percentage)"),
				));
			}

			// Validate collateral_size is not empty
			if params.collateral_size.is_empty() {
				return Err(PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Collateral size cannot be empty")
						.with_field("collateral_size")
						.with_suggestion(
							"Provide a valid collateral size in human-readable format",
						),
				));
			}

			let address_bytes = validate_omni_account_hex(&user.omni_account, "omni_account")
				.map_err(PumpxRpcError::from)?;

			validate_omni_account_length(&address_bytes, "omni_account")
				.map_err(PumpxRpcError::from)?;

			let wrapper = NativeTaskWrapper::new(
				NativeTask::RequestLoan(
					AccountId::decode(&mut &address_bytes[..]).map_err(|e| {
						error!("Failed to decode AccountId from bytes: {:?}", e);
						PumpxRpcError::from(DetailedError::account_parse_error(
							&user.omni_account,
							&format!("Failed to decode account: {:?}", e),
						))
					})?,
					params.chain_id,
					params.wallet_index,
					params.smart_wallet_address,
					params.collateral_ticker,
					params.lending_ratio,
					params.collateral_size,
				),
				None,
				None,
				user.client_id,
			);

			handle_omni_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::RequestLoan {
					spot_sell_cloid,
					hedge_open_cloid,
					usdc_received,
					spot_sell_tx_hash,
					hedge_open_tx_hash,
				} => Ok(RequestLoanResponse {
					spot_sell_cloid,
					hedge_open_cloid,
					usdc_received,
					spot_sell_tx_hash,
					hedge_open_tx_hash,
				}),
				_ => {
					error!("Unexpected response type from native task handler");
					Err(DetailedError::unexpected_response_type("RequestLoan", "Unknown").into())
				},
			})
			.await
		})
		.expect("Failed to register omni_requestLoan method");
}
