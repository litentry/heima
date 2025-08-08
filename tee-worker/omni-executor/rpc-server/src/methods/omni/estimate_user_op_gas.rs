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

use super::common::handle_omni_native_task;
use crate::methods::omni::PumpxRpcError;
use crate::server::RpcContext;
use crate::ErrorCode;
use alloy::primitives::Address;
use executor_core::native_task::{NativeTask, NativeTaskWrapper};
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::{AccountId, ChainId};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use parity_scale_codec::Decode;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

const OMNI_ACCOUNT_BYTES_LENGTH: usize = 32;

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
pub struct EstimateUserOpGasResponse {
	pub call_gas_limit: String,
	pub verification_gas_limit: String,
	pub pre_verification_gas: String,
	pub paymaster_verification_gas_limit: String,
	pub paymaster_post_op_gas_limit: String,
}

pub fn register_estimate_user_op_gas(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_estimateUserOpGas", |params, ctx, _ext| async move {
			let params = params.parse::<EstimateUserOpGasParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!("Received omni_estimateUserOpGas, params: {:?}", params);

			let account_id =
				decode_account_id(&params.omni_account).map_err(PumpxRpcError::from_error_code)?;

			validate_sender_address(&params.user_operation.sender)
				.map_err(PumpxRpcError::from_error_code)?;

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
				} => Ok(EstimateUserOpGasResponse {
					call_gas_limit: call_gas_limit.to_string(),
					verification_gas_limit: verification_gas_limit.to_string(),
					pre_verification_gas: pre_verification_gas.to_string(),
					paymaster_verification_gas_limit: paymaster_verification_gas_limit.to_string(),
					paymaster_post_op_gas_limit: paymaster_post_op_gas_limit.to_string(),
				}),
				_ => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register omni_estimateUserOpGas method");
}

fn decode_account_id(hex_str: &str) -> Result<AccountId, ErrorCode> {
	let bytes = hex::decode(hex_str.strip_prefix("0x").unwrap_or(hex_str)).map_err(|e| {
		error!("Failed to decode omni account hex string: {}", e);
		ErrorCode::ParseError
	})?;

	if bytes.len() != OMNI_ACCOUNT_BYTES_LENGTH {
		error!(
			"Invalid omni account length: expected {} bytes, got {}",
			OMNI_ACCOUNT_BYTES_LENGTH,
			bytes.len()
		);
		return Err(ErrorCode::ParseError);
	}

	AccountId::decode(&mut &bytes[..]).map_err(|e| {
		error!("Failed to decode AccountId from bytes: {}", e);
		ErrorCode::ParseError
	})
}

fn validate_sender_address(sender: &str) -> Result<Address, ErrorCode> {
	sender.parse::<Address>().map_err(|e| {
		error!("Invalid sender address '{}': {}", sender, e);
		ErrorCode::ParseError
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_decode_account_id_valid() {
		// Valid 32-byte hex string
		let hex_str = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
		let result = decode_account_id(hex_str);
		assert!(result.is_ok(), "Should decode valid 32-byte hex");

		let _account_id = result.unwrap();
		// AccountId is decoded successfully - internal structure is opaque
	}

	#[test]
	fn test_decode_account_id_with_0x_prefix() {
		// Test with 0x prefix
		let with_prefix = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
		let without_prefix = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

		let result1 = decode_account_id(with_prefix);
		let result2 = decode_account_id(without_prefix);

		assert!(result1.is_ok(), "Should handle 0x prefix");
		assert!(result2.is_ok(), "Should handle without 0x prefix");
		assert_eq!(result1.unwrap(), result2.unwrap(), "Results should be identical");
	}

	#[test]
	fn test_decode_account_id_invalid_hex() {
		// Invalid hex characters
		let invalid_hex = "0xgggggggg90abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
		let result = decode_account_id(invalid_hex);

		assert!(result.is_err(), "Should reject invalid hex");
		assert_eq!(result.unwrap_err(), ErrorCode::ParseError);
	}

	#[test]
	fn test_decode_account_id_wrong_length() {
		// Too short (31 bytes)
		let too_short = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcd";
		let result_short = decode_account_id(too_short);
		assert!(result_short.is_err(), "Should reject too short");

		// Too long (33 bytes)
		let too_long = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef00";
		let result_long = decode_account_id(too_long);
		assert!(result_long.is_err(), "Should reject too long");

		// Empty
		let empty = "";
		let result_empty = decode_account_id(empty);
		assert!(result_empty.is_err(), "Should reject empty string");
	}

	#[test]
	fn test_validate_sender_address_valid() {
		// Valid Ethereum addresses
		let addresses = vec![
			"0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb9",
			"0x0000000000000000000000000000000000000000",
			"0xffffffffffffffffffffffffffffffffffffffff",
			"0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae",
		];

		for addr in addresses {
			let result = validate_sender_address(addr);
			assert!(result.is_ok(), "Should accept valid address: {}", addr);
		}
	}

	#[test]
	fn test_validate_sender_address_checksummed() {
		// Checksummed addresses should work
		let checksummed = "0x5aAeb6053f3E94C9b9A09f33669435E7Ef1BeAed";
		let result = validate_sender_address(checksummed);
		assert!(result.is_ok(), "Should accept checksummed address");
	}

	#[test]
	fn test_validate_sender_address_invalid_format() {
		// Invalid formats
		let invalid_addresses = vec![
			"not_an_address",
			"0x",
			"0xZZZZ35Cc6634C0532925a3b844Bc9e7595f0bEb9", // Invalid hex
			// Note: Address without 0x prefix is actually valid in alloy
			"",
		];

		for addr in invalid_addresses {
			let result = validate_sender_address(addr);
			assert!(result.is_err(), "Should reject invalid address: {}", addr);
			assert_eq!(result.unwrap_err(), ErrorCode::ParseError);
		}
	}

	#[test]
	fn test_validate_sender_address_wrong_length() {
		// Wrong length addresses
		let too_short = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bE"; // 39 chars
		let too_long = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb900"; // 43 chars

		let result_short = validate_sender_address(too_short);
		assert!(result_short.is_err(), "Should reject too short address");

		let result_long = validate_sender_address(too_long);
		assert!(result_long.is_err(), "Should reject too long address");
	}

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
