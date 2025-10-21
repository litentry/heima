use super::common::handle_omni_native_task;
use crate::detailed_error::DetailedError;
use crate::error_code::{INTERNAL_ERROR_CODE, PARSE_ERROR_CODE};
use crate::methods::omni::PumpxRpcError;
use crate::server::RpcContext;
use alloy::primitives::Address;
use executor_core::intent_executor::IntentExecutor;
use executor_core::native_task::{NativeTask, NativeTaskWrapper};
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::{AccountId, ChainId};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use parity_scale_codec::Decode;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct RequestLoanTestParams {
	pub user_operation: SerializablePackedUserOperation,
	pub chain_id: ChainId,
	pub wallet_index: u32,
	pub omni_account: String,
	pub client_id: String,
	pub collateral_ticker: String,
	pub collateral_size: String,
	pub lending_ratio: u32,
}

#[derive(Serialize, Clone)]
pub struct RequestLoanTestResponse {
	pub spot_sell_cloid: String,
	pub hedge_open_cloid: String,
	pub usdc_received: String,
	pub spot_sell_tx_hash: Option<String>,
	pub hedge_open_tx_hash: Option<String>,
}

pub fn register_request_loan_test<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>,
	>,
) {
	module
		.register_async_method("omni_requestLoanTest", |params, ctx, _ext| async move {
			let params = params.parse::<RequestLoanTestParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Parse error")
						.with_reason("Invalid JSON format or missing required fields"),
				)
			})?;

			debug!("Received omni_requestLoanTest, params: {:?}", params);

			let address_bytes =
				hex::decode(params.omni_account.strip_prefix("0x").unwrap_or(&params.omni_account))
					.map_err(|_| {
						error!("Failed to decode omni account hex string");
						PumpxRpcError::from(
							DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
								.with_reason("Failed to decode omni account hex string"),
						)
					})?;

			if address_bytes.len() != 32 {
				error!(
					"Invalid omni account length: expected 32 bytes, got {}",
					address_bytes.len()
				);
				return Err(PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(format!(
						"Invalid omni account length: expected 32 bytes, got {}",
						address_bytes.len()
					)),
				));
			}

			// Validate sender address
			params.user_operation.sender.parse::<Address>().map_err(|e| {
				error!("Invalid sender address '{}': {}", params.user_operation.sender, e);
				PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Parse error")
						.with_field("sender")
						.with_reason(format!(
							"Invalid sender address '{}': {}",
							params.user_operation.sender, e
						)),
				)
			})?;

			// Validate collateral ticker is not empty and normalize to uppercase
			if params.collateral_ticker.is_empty() {
				return Err(PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Collateral ticker cannot be empty")
						.with_field("collateral_ticker")
						.with_suggestion("Provide a valid ticker like ETH, PURR, etc."),
				));
			}
			let collateral_ticker = params.collateral_ticker.to_uppercase();

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

			// Validate lending_ratio is between 0 and 100 (0-100%)
			if params.lending_ratio > 100 {
				return Err(PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Lending ratio must be between 0 and 100")
						.with_field("lending_ratio")
						.with_received(params.lending_ratio.to_string())
						.with_expected("0-100 (percentage)"),
				));
			}

			let wrapper = NativeTaskWrapper::new(
				NativeTask::RequestLoanTest(
					AccountId::decode(&mut &address_bytes[..]).map_err(|_| {
						error!("Failed to decode AccountId from bytes");
						PumpxRpcError::from(
							DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
								.with_reason("Failed to decode AccountId from bytes"),
						)
					})?,
					params.user_operation.clone(),
					params.chain_id,
					params.wallet_index,
					collateral_ticker,
					params.collateral_size,
					params.lending_ratio,
				),
				None,
				None,
				params.client_id,
			);

			handle_omni_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::RequestLoan {
					spot_sell_cloid,
					hedge_open_cloid,
					usdc_received,
					spot_sell_tx_hash,
					hedge_open_tx_hash,
				} => Ok(RequestLoanTestResponse {
					spot_sell_cloid,
					hedge_open_cloid,
					usdc_received,
					spot_sell_tx_hash,
					hedge_open_tx_hash,
				}),
				_ => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from(
						DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
							.with_reason("Unexpected response type from native task handler"),
					))
				},
			})
			.await
		})
		.expect("Failed to register omni_requestLoanTest method");
}
