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

use crate::detailed_error::DetailedError;
use crate::server::RpcContext;
use crate::utils::gas_estimation::estimate_user_op_gas;
use crate::utils::omni::to_omni_account;
use crate::utils::user_op::convert_to_packed_user_op;
use crate::utils::validation::{parse_rpc_params, validate_evm_address};
use alloy::primitives::utils::format_units;
use jsonrpsee::RpcModule;
use oe_core::intent::executor::IntentExecutor;
use oe_core::types::SerializablePackedUserOperation;
use oe_primitives::ChainId;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

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
	#[allow(dead_code)]
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
	pub max_fee_per_gas: String,
	pub max_priority_fee_per_gas: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub estimated_token_cost: Option<TokenCostInfo>,
}

pub fn register_estimate_user_op_gas<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_estimateUserOpGas", |params, ctx, _ext| async move {
			let params = parse_rpc_params::<EstimateUserOpGasParams>(params)?;

			debug!("Received omni_estimateUserOpGas, params: {:?}", params);

			let omni_account = to_omni_account(&params.omni_account)?;
			validate_evm_address(&params.user_operation.sender, "user_operation.sender")?;

			// Inlined handler logic from handle_estimate_user_op_gas
			info!(
				"Processing EstimateUserOpGas for account {:?}, wallet_index: {}, chain_id: {}",
				omni_account, params.wallet_index, params.chain_id
			);

			// Get EntryPoint client for this chain
			let entry_point_client =
				ctx.entry_point_clients.get(&params.chain_id).ok_or_else(|| {
					error!("No EntryPoint client configured for chain_id: {}", params.chain_id);
					DetailedError::invalid_chain_id(params.chain_id).to_rpc_error()
				})?;

			// Convert SerializablePackedUserOperation to PackedUserOperation
			let packed_user_op =
				convert_to_packed_user_op(params.user_operation.clone()).map_err(|e| {
					error!("Failed to convert UserOperation: {}", e);
					DetailedError::invalid_user_op(&e).to_rpc_error()
				})?;

			// Perform gas estimation
			let result = estimate_user_op_gas(
				entry_point_client.clone(),
				packed_user_op,
				params.chain_id,
				ctx.oe_client_binance_client.as_ref(),
			)
			.await;

			// Process response
			match result {
				Ok(gas_estimate) => {
					// Convert token cost estimate to RPC format if present
					let token_cost_info = gas_estimate.estimated_token_cost.map(|cost| {
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
						call_gas_limit: gas_estimate.call_gas_limit.to_string(),
						verification_gas_limit: gas_estimate.verification_gas_limit.to_string(),
						pre_verification_gas: gas_estimate.pre_verification_gas.to_string(),
						paymaster_verification_gas_limit: gas_estimate
							.paymaster_verification_gas_limit
							.to_string(),
						paymaster_post_op_gas_limit: gas_estimate
							.paymaster_post_op_gas_limit
							.to_string(),
						max_fee_per_gas: gas_estimate.max_fee_per_gas.to_string(),
						max_priority_fee_per_gas: gas_estimate.max_priority_fee_per_gas.to_string(),
						estimated_token_cost: token_cost_info,
					})
				},
				Err(e) => {
					error!("Gas estimation failed: {}", e);
					Err(DetailedError::gas_estimation_failed().to_rpc_error())
				},
			}
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
