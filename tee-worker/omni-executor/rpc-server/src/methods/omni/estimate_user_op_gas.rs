// Copyright 2020-2024 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

use super::common::{handle_omni_native_task, PumpxRpcError};
use crate::detailed_error::DetailedError;
use crate::server::RpcContext;
use crate::validation_helpers::{
	validate_ethereum_address, validate_omni_account_hex, validate_omni_account_length,
};
use alloy::primitives::utils::format_units;
use executor_core::intent_executor::IntentExecutor;
use executor_core::native_task::{NativeTask, NativeTaskWrapper};
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::{AccountId, ChainId};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use parity_scale_codec::Decode;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

/// Format a token amount with decimals to a human-readable string
fn format_token_amount(amount: u128, decimals: u8) -> String {
	match format_units(amount, decimals) {
		Ok(mut value) => {
			if value.contains('.') {
				while value.ends_with('0') {
					value.pop();
				}
				if value.ends_with('.') {
					value.pop();
				}
			}
			if value.is_empty() {
				"0".to_string()
			} else {
				value
			}
		},
		Err(_) => amount.to_string(),
	}
}

#[derive(Debug, Deserialize)]
pub struct EstimateUserOpGasParams {
	pub user_operation: SerializablePackedUserOperation,
	pub chain_id: ChainId,
	pub wallet_index: u32,
	pub omni_account: String,
	pub client_id: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenCostInfo {
	pub token_address: String,
	pub amount: String,     // Decimal string
	pub amount_hex: String, // Hex string
	pub decimals: u8,
	pub exchange_rate: String,    // Decimal string
	pub formatted_amount: String, // Human readable (e.g., "2.5")
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EstimateUserOpGasResponse {
	pub call_gas_limit: String,
	pub verification_gas_limit: String,
	pub pre_verification_gas: String,
	pub paymaster_verification_gas_limit: String,
	pub paymaster_post_op_gas_limit: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub estimated_token_cost: Option<TokenCostInfo>,
}

pub fn register_estimate_user_op_gas<
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
	module
		.register_async_method("omni_estimateUserOpGas", |params, ctx, _ext| async move {
			let params = params.parse::<EstimateUserOpGasParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(
						crate::error_code::MISSING_REQUIRED_FIELD_CODE,
						"Failed to parse request parameters",
					)
					.with_reason(format!("Parse error: {}", e)),
				)
			})?;

			debug!("Received omni_estimateUserOpGas, params: {:?}", params);

			let account_bytes = validate_omni_account_hex(&params.omni_account, "omni_account")
				.map_err(PumpxRpcError::from)?;
			validate_omni_account_length(&account_bytes, "omni_account")
				.map_err(PumpxRpcError::from)?;
			let account_id = AccountId::decode(&mut &account_bytes[..]).map_err(|e| {
				PumpxRpcError::from(DetailedError::account_parse_error(
					&params.omni_account,
					&e.to_string(),
				))
			})?;

			validate_ethereum_address(&params.user_operation.sender, "user_operation.sender")
				.map_err(PumpxRpcError::from)?;

			let wrapper = NativeTaskWrapper::new(
				NativeTask::EstimateUserOpGas(
					account_id,
					params.user_operation.clone(),
					params.chain_id,
					params.wallet_index,
				),
				None,
				None,
				params.client_id,
			);

			handle_omni_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::EstimateUserOpGas {
					call_gas_limit,
					verification_gas_limit,
					pre_verification_gas,
					paymaster_verification_gas_limit,
					paymaster_post_op_gas_limit,
					estimated_token_cost,
				} => {
					// Convert token cost estimate to RPC format if present
					let token_cost_info = estimated_token_cost.map(|cost| {
						// Format the amount as a human-readable value
						let formatted_amount = format_token_amount(cost.amount, cost.decimals);

						TokenCostInfo {
							token_address: cost.token_address,
							amount: cost.amount.to_string(),
							amount_hex: format!("0x{:x}", cost.amount),
							decimals: cost.decimals,
							exchange_rate: cost.exchange_rate.to_string(),
							formatted_amount,
						}
					});

					Ok(EstimateUserOpGasResponse {
						call_gas_limit: call_gas_limit.to_string(),
						verification_gas_limit: verification_gas_limit.to_string(),
						pre_verification_gas: pre_verification_gas.to_string(),
						paymaster_verification_gas_limit: paymaster_verification_gas_limit
							.to_string(),
						paymaster_post_op_gas_limit: paymaster_post_op_gas_limit.to_string(),
						estimated_token_cost: token_cost_info,
					})
				},
				_ => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from(DetailedError::unexpected_response_type(
						"EstimateUserOpGas response",
						"Unknown response type",
					)))
				},
			})
			.await
		})
		.expect("Failed to register omni_estimateUserOpGas method");
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_estimate_params_valid() {
		// Valid params JSON
		let json = serde_json::json!({
			"user_operation": {
				"sender": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb9",
				"nonce": 42,
				"init_code": "0xdeadbeef",
				"call_data": "0xcafebabe",
				"account_gas_limits": "0x0000000000000000000000000030d4000000000000000000000000000000c350",
				"pre_verification_gas": 21000,
				"gas_fees": "0x000000000000000000000003b9aca0000000000000000000000000000b2d05e0",
				"paymaster_and_data": "0x",
				"signature": "0x1234"
			},
			"chain_id": 1,
			"wallet_index": 0,
			"omni_account": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
			"client_id": "test-client-123"
		});

		let params: EstimateUserOpGasParams = serde_json::from_value(json).unwrap();
		assert_eq!(params.chain_id, 1);
		assert_eq!(params.wallet_index, 0);
		assert_eq!(
			params.omni_account,
			"0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
		);
		assert_eq!(params.client_id, "test-client-123");
		assert_eq!(params.user_operation.sender, "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb9");
		assert_eq!(params.user_operation.nonce, 42);
	}

	#[test]
	fn test_format_token_amount() {
		// Test with 6 decimals (USDC)
		assert_eq!(format_token_amount(1_000_000, 6), "1");
		assert_eq!(format_token_amount(1_500_000, 6), "1.5");
		assert_eq!(format_token_amount(1_234_567, 6), "1.234567");
		assert_eq!(format_token_amount(1_230_000, 6), "1.23");
		assert_eq!(format_token_amount(500_000, 6), "0.5");
		assert_eq!(format_token_amount(0, 6), "0");

		// Test with 18 decimals (DAI)
		assert_eq!(format_token_amount(1_000_000_000_000_000_000, 18), "1");
		assert_eq!(format_token_amount(1_500_000_000_000_000_000, 18), "1.5");
		assert_eq!(format_token_amount(500_000_000_000_000_000, 18), "0.5");

		// Test with 0 decimals
		assert_eq!(format_token_amount(100, 0), "100");
		assert_eq!(format_token_amount(0, 0), "0");
	}

	#[test]
	fn test_parse_estimate_params_missing_fields() {
		// Missing required field
		let json_missing_chain = serde_json::json!({
			"user_operation": {
				"sender": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb9",
				"nonce": 42,
				"init_code": "0x",
				"call_data": "0x",
				"account_gas_limits": "0x0000000000000000000000000030d4000000000000000000000000000000c350",
				"pre_verification_gas": 21000,
				"gas_fees": "0x000000000000000000000003b9aca0000000000000000000000000000b2d05e0",
				"paymaster_and_data": "0x",
				"signature": null
			},
			"wallet_index": 0,
			"omni_account": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
			"client_id": "test-client-123"
			// Missing chain_id
		});

		let result: Result<EstimateUserOpGasParams, _> = serde_json::from_value(json_missing_chain);
		assert!(result.is_err(), "Should fail when missing required fields");
	}
}
