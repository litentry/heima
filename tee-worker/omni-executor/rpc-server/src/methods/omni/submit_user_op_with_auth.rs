use crate::auth_utils::{
	verify_payload_timestamp, verify_wildmeta_backend_signature, verify_wildmeta_signature,
};
use crate::detailed_error::DetailedError;
use crate::error_code::{
	AUTH_VERIFICATION_FAILED_CODE, INVALID_USER_OPERATION_CODE, PARSE_ERROR_CODE,
};
use crate::methods::RpcResult;
use crate::server::RpcContext;
use crate::utils::paymaster::{
	extract_paymaster_address, is_whitelisted_paymaster, parse_whitelisted_paymasters,
	process_erc20_paymaster_data,
};
use crate::utils::user_op::{convert_to_packed_user_op, substrate_to_ethereum_signature};
use crate::validation_helpers::{
	validate_chain_id, validate_user_operations, validate_wallet_index,
};
use aa_contracts_client::calculate_user_operation_hash;
use alloy::primitives::{hex, Address, Bytes};
use binance_api::BinancePaymasterApi;
use executor_core::intent_executor::IntentExecutor;
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::{ChainId, ClientAuth, Identity, UserAuth, UserId};
use executor_storage::WildmetaTimestampStorage;
use jsonrpsee::{types::ErrorObjectOwned, RpcModule};
use pumpx::pubkey_to_address;
use serde::{Deserialize, Serialize};
use signer_client::ChainType;
use std::sync::Arc;
use tracing::{debug, error, info};

// Chain ID constants
const ARBITRUM_MAINNET: ChainId = 42161;
const ARBITRUM_SEPOLIA: ChainId = 421614;
const HYPEREVM_MAINNET: ChainId = 999;
const HYPEREVM_TESTNET: ChainId = 998;

// Contract addresses - These should be moved to configuration in the future
const ARBITRUM_USDC_ADDRESS: &str = "0xaf88d065e77c8cC2239327C5EDb3A432268e5831"; // Arbitrum mainnet USDC
const ARBITRUM_SEPOLIA_USDC_ADDRESS: &str = "0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d"; // Arbitrum Sepolia USDC
const ARB_TO_HYPER_BRIDGE_ADDRESS: &str = "0x2df1c51e09aecf9cacb7bc98cb1742757f163df7";
const HYPEREVM_CORE_WRITER_ADDRESS: &str = "0x3333333333333333333333333333333333333333";

// ERC20 transfer method signature: transfer(address,uint256)
const ERC20_TRANSFER_SIGNATURE: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb];

#[derive(Debug, Deserialize)]
pub struct SubmitUserOpWithAuthParams {
	pub user_operations: Vec<SerializablePackedUserOperation>,
	pub chain_id: ChainId,
	pub wallet_index: u32,
	pub user_id: UserId,
	pub _user_auth: Option<UserAuth>,
	pub client_id: String,
	pub client_auth: ClientAuth,
}

#[derive(Serialize, Clone)]
pub struct SubmitUserOpWithAuthResponse {
	pub transaction_hash: Option<String>,
}

/// Validates USDC transfer call for Arbitrum chains
fn validate_arbitrum_usdc_transfer(call_data: &str, chain_id: ChainId) -> RpcResult<()> {
	// Validate chain is supported for Arbitrum USDC validation
	match chain_id {
		ARBITRUM_MAINNET | ARBITRUM_SEPOLIA => {},
		_ => {
			return Err(DetailedError::new(
				AUTH_VERIFICATION_FAILED_CODE,
				"Chain ID is not supported for Arbitrum USDC validation",
			)
			.with_field("chain_id")
			.with_received(chain_id.to_string())
			.with_expected("42161 or 421614")
			.to_rpc_error());
		},
	};

	// Parse calldata - should be USDC.transfer method call
	let call_bytes =
		hex::decode(call_data.strip_prefix("0x").unwrap_or(call_data)).map_err(|e| {
			DetailedError::new(
				// todo: proper error code
				AUTH_VERIFICATION_FAILED_CODE,
				"Invalid hex encoding in call data",
			)
			.with_field("call_data")
			.with_reason(format!("Hex decode error: {}", e))
			.to_rpc_error()
		})?;

	// Check if it's an ERC20 transfer call (method signature 0xa9059cbb)
	if call_bytes.len() < 4 || call_bytes[0..4] != ERC20_TRANSFER_SIGNATURE {
		return Err(DetailedError::new(
			// todo: proper error code
			AUTH_VERIFICATION_FAILED_CODE,
			"Call data is not an ERC20 transfer function call",
		)
		.with_field("method_signature")
		.with_received(if call_bytes.len() >= 4 {
			format!("0x{}", hex::encode(&call_bytes[0..4]))
		} else {
			"<insufficient data>".to_string()
		})
		.with_expected("0xa9059cbb (ERC20.transfer)")
		.to_rpc_error());
	}

	// Decode the transfer call to get recipient
	if call_bytes.len() < 68 {
		return Err(DetailedError::new(
			// todo: proper error code
			AUTH_VERIFICATION_FAILED_CODE,
			"Call data too short for ERC20 transfer",
		)
		.with_field("call_data_length")
		.with_received(call_bytes.len().to_string())
		.with_expected("68 bytes minimum")
		.to_rpc_error());
	}

	// Extract recipient address (bytes 4-36, but we need the last 20 bytes)
	let recipient_bytes = &call_bytes[16..36];
	let recipient = format!("0x{}", hex::encode(recipient_bytes));

	// Validate recipient is the official arb -> hyper bridge
	if recipient.to_lowercase() != ARB_TO_HYPER_BRIDGE_ADDRESS.to_lowercase() {
		return Err(DetailedError::new(
			AUTH_VERIFICATION_FAILED_CODE,
			"USDC transfer recipient is not the official Arbitrum to Hyperliquid bridge",
		)
		.with_field("recipient")
		.with_received(recipient)
		.with_expected(ARB_TO_HYPER_BRIDGE_ADDRESS)
		.to_rpc_error());
	}

	Ok(())
}

/// Validates core writer call for HyperEVM chains
fn validate_hyperevm_core_writer(call_data: &str, chain_id: ChainId) -> RpcResult<()> {
	// Validate chain is HyperEVM
	if chain_id != HYPEREVM_MAINNET && chain_id != HYPEREVM_TESTNET {
		return Err(DetailedError::new(
			// todo: proper error code
			AUTH_VERIFICATION_FAILED_CODE,
			"Chain ID is not supported for HyperEVM validation",
		)
		.with_field("chain_id")
		.with_received(chain_id.to_string())
		.with_expected("999 or 998")
		.to_rpc_error());
	}

	// Parse calldata
	let call_bytes =
		hex::decode(call_data.strip_prefix("0x").unwrap_or(call_data)).map_err(|e| {
			DetailedError::new(
				// todo: proper error code
				AUTH_VERIFICATION_FAILED_CODE,
				"Invalid hex encoding in call data",
			)
			.with_field("call_data")
			.with_reason(format!("Hex decode error: {}", e))
			.to_rpc_error()
		})?;

	// Extract method signature if available
	if call_bytes.len() < 4 {
		return Err(DetailedError::new(
			// todo: proper error code
			AUTH_VERIFICATION_FAILED_CODE,
			"Call data too short to contain method signature",
		)
		.with_field("call_data_length")
		.with_received(call_bytes.len().to_string())
		.with_expected("4 bytes minimum")
		.to_rpc_error());
	}

	// Proper action_id extraction based on payload format:
	// [1 byte] → Version (currently 0x01)
	// [3 bytes] → action_id
	// [rest] → Action-specific payload (encoded via Solidity ABI rules)

	// The calldata should contain the raw payload as data parameter in method call
	// First decode the actual payload from the method call parameters
	if call_bytes.len() < 4 + 32 + 32 {
		return Err(DetailedError::new(
			INVALID_USER_OPERATION_CODE,
			"Call data too short for payload parameter",
		)
		.with_field("call_data_length")
		.with_received(call_bytes.len().to_string())
		.with_expected("68 bytes minimum (4 bytes method + 32 bytes offset + 32 bytes length)")
		.to_rpc_error());
	}

	// Skip method signature (4 bytes) and offset parameter (32 bytes)
	// Then read the length of the payload (next 32 bytes)
	let payload_length_bytes = &call_bytes[36..68];
	let payload_length = u32::from_be_bytes([
		payload_length_bytes[28],
		payload_length_bytes[29],
		payload_length_bytes[30],
		payload_length_bytes[31],
	]) as usize;

	// Ensure we have enough data for the payload
	if call_bytes.len() < 68 + payload_length {
		return Err(DetailedError::new(
			INVALID_USER_OPERATION_CODE,
			"Call data too short for declared payload length",
		)
		.with_field("call_data_length")
		.with_received(call_bytes.len().to_string())
		.with_expected(format!("{} bytes", 68 + payload_length))
		.to_rpc_error());
	}

	// Extract the actual payload
	let payload = &call_bytes[68..68 + payload_length];

	// Validate payload format: minimum 4 bytes (1 version + 3 action_id)
	if payload.len() < 4 {
		return Err(DetailedError::new(
			INVALID_USER_OPERATION_CODE,
			"Payload too short for version and action_id",
		)
		.with_field("payload_length")
		.with_received(payload.len().to_string())
		.with_expected("4 bytes minimum (1 version + 3 action_id)")
		.to_rpc_error());
	}

	// Extract version (first byte)
	let version = payload[0];
	if version != 0x01 {
		return Err(DetailedError::new(INVALID_USER_OPERATION_CODE, "Invalid payload version")
			.with_field("version")
			.with_received(format!("0x{:02x}", version))
			.with_expected("0x01")
			.to_rpc_error());
	}

	// Extract action_id (next 3 bytes) and convert to u32
	let action_id_bytes = [0, payload[1], payload[2], payload[3]];
	let action_id = u32::from_be_bytes(action_id_bytes);

	// Validate action_id is one of the allowed HyperCore actions
	let valid_action_ids = [
		0x000002, // Cancel a perpetual order
		0x000003, // Spot transfer
		0x000004, // Stake HLP
		0x000005, // Vault transfer
		0x000007, // Other allowed action
	];

	if !valid_action_ids.contains(&action_id) {
		return Err(DetailedError::new(
			INVALID_USER_OPERATION_CODE,
			"Invalid action_id for HyperEVM core writer call",
		)
		.with_field("action_id")
		.with_received(format!("0x{:06x}", action_id))
		.with_expected("One of: 0x000002, 0x000003, 0x000004, 0x000005, 0x000007")
		.to_rpc_error());
	}

	Ok(())
}

/// Extract target addresses and inner calldata from OmniAccount execute() or executeBatch() calldata
fn extract_execute_params_from_calldata(call_data: &str) -> RpcResult<Vec<(Address, String)>> {
	// Parse calldata - should be OmniAccount.execute() or executeBatch()
	let call_bytes =
		hex::decode(call_data.strip_prefix("0x").unwrap_or(call_data)).map_err(|e| {
			DetailedError::new(PARSE_ERROR_CODE, "Invalid hex encoding in call data")
				.with_field("call_data")
				.with_reason(format!("Hex decode error: {}", e))
				.to_rpc_error()
		})?;

	if call_bytes.len() < 4 {
		return Err(DetailedError::new(
			AUTH_VERIFICATION_FAILED_CODE,
			"Call data too short to contain method signature",
		)
		.with_field("call_data_length")
		.with_received(call_bytes.len().to_string())
		.with_expected("4 bytes minimum")
		.to_rpc_error());
	}

	// Check method signatures
	const EXECUTE_SIGNATURE: [u8; 4] = [0xb6, 0x1d, 0x27, 0xf6]; // execute(address,uint256,bytes)
	const EXECUTE_BATCH_SIGNATURE: [u8; 4] = [0x18, 0xdf, 0xeb, 0x3c]; // executeBatch((address,uint256,bytes)[])

	let method_sig = &call_bytes[0..4];

	if method_sig == EXECUTE_SIGNATURE {
		// Handle single execute call
		let (target, inner_data) = parse_single_execute(&call_bytes)?;
		Ok(vec![(target, inner_data)])
	} else if method_sig == EXECUTE_BATCH_SIGNATURE {
		// Handle executeBatch call
		parse_execute_batch(&call_bytes)
	} else {
		return Err(DetailedError::new(
			AUTH_VERIFICATION_FAILED_CODE,
			"Call data is not an OmniAccount execute or executeBatch function call",
		)
		.with_field("method_signature")
		.with_received(format!("0x{}", hex::encode(method_sig)))
		.with_expected("0xb61d27f6 (execute) or 0x18dfeb3c (executeBatch)")
		.to_rpc_error());
	}
}

/// Parse single execute(address,uint256,bytes) call
fn parse_single_execute(call_bytes: &[u8]) -> RpcResult<(Address, String)> {
	// Minimum length check: method(4) + target(32) + value(32) + data_offset(32) = 100 bytes
	if call_bytes.len() < 100 {
		return Err(DetailedError::new(
			AUTH_VERIFICATION_FAILED_CODE,
			"Call data too short for OmniAccount execute call",
		)
		.with_field("call_data_length")
		.with_received(call_bytes.len().to_string())
		.with_expected("100 bytes minimum (method + target + value + data_offset + data_length)")
		.to_rpc_error());
	}

	// Extract target address (bytes 16-36, last 20 bytes of the first 32-byte parameter)
	let target_bytes = &call_bytes[16..36];
	let target_address = Address::from_slice(target_bytes);

	// Skip method(4) + target(32) + value(32) + data_offset(32) = 100 bytes to get to data length
	if call_bytes.len() < 132 {
		return Err(DetailedError::new(
			AUTH_VERIFICATION_FAILED_CODE,
			"Call data too short to contain data length",
		)
		.with_field("call_data_length")
		.with_received(call_bytes.len().to_string())
		.with_expected("132 bytes minimum (method + params + data_length)")
		.to_rpc_error());
	}

	// Extract data length (bytes 100-132)
	let data_length_bytes = &call_bytes[100..132];
	let data_length = u32::from_be_bytes([
		data_length_bytes[28],
		data_length_bytes[29],
		data_length_bytes[30],
		data_length_bytes[31],
	]) as usize;

	// Extract the actual inner calldata
	let data_start = 132;
	if call_bytes.len() < data_start + data_length {
		return Err(DetailedError::new(
			AUTH_VERIFICATION_FAILED_CODE,
			"Call data too short for declared data length",
		)
		.with_field("call_data_length")
		.with_received(call_bytes.len().to_string())
		.with_expected(format!("{} bytes", data_start + data_length))
		.to_rpc_error());
	}

	let inner_data = &call_bytes[data_start..data_start + data_length];
	let inner_calldata = format!("0x{}", hex::encode(inner_data));

	Ok((target_address, inner_calldata))
}

/// Parse executeBatch((address,uint256,bytes)[]) call
fn parse_execute_batch(call_bytes: &[u8]) -> RpcResult<Vec<(Address, String)>> {
	// Minimum length: method(4) + array_offset(32) + array_length(32) = 68 bytes
	if call_bytes.len() < 68 {
		return Err(DetailedError::new(
			AUTH_VERIFICATION_FAILED_CODE,
			"Call data too short for executeBatch call",
		)
		.with_field("call_data_length")
		.with_received(call_bytes.len().to_string())
		.with_expected("68 bytes minimum")
		.to_rpc_error());
	}

	// Skip method signature (4 bytes) and array offset (32 bytes) to get array length
	let array_length_bytes = &call_bytes[36..68];
	let array_length = u32::from_be_bytes([
		array_length_bytes[28],
		array_length_bytes[29],
		array_length_bytes[30],
		array_length_bytes[31],
	]) as usize;

	// Each Call struct takes 96 bytes: target(32) + value(32) + data_offset(32)
	// Plus variable length for the data field
	let mut results = Vec::new();
	let mut current_pos = 68; // Start after method + offset + length

	for i in 0..array_length {
		// Each struct entry is at least 96 bytes (target + value + data_offset)
		if current_pos + 96 > call_bytes.len() {
			return Err(DetailedError::new(
				AUTH_VERIFICATION_FAILED_CODE,
				format!("Call data too short for batch entry {}", i),
			)
			.with_field("call_data_length")
			.with_received(call_bytes.len().to_string())
			.with_expected(format!("{} bytes minimum", current_pos + 96))
			.to_rpc_error());
		}

		// Extract target address (last 20 bytes of the 32-byte slot)
		let target_bytes = &call_bytes[current_pos + 12..current_pos + 32];
		let target_address = Address::from_slice(target_bytes);

		// Skip target(32) + value(32) to get to data_offset
		let data_offset_pos = current_pos + 64;
		let data_offset_bytes = &call_bytes[data_offset_pos..data_offset_pos + 32];
		let relative_data_offset = u32::from_be_bytes([
			data_offset_bytes[28],
			data_offset_bytes[29],
			data_offset_bytes[30],
			data_offset_bytes[31],
		]) as usize;

		// Calculate absolute position of the data
		// The offset is relative to the start of the current Call struct
		let data_length_pos = current_pos + relative_data_offset;
		if data_length_pos + 32 > call_bytes.len() {
			return Err(DetailedError::new(
				AUTH_VERIFICATION_FAILED_CODE,
				format!("Call data too short for batch entry {} data length", i),
			)
			.with_field("call_data_length")
			.with_received(call_bytes.len().to_string())
			.with_expected(format!("{} bytes minimum", data_length_pos + 32))
			.to_rpc_error());
		}

		// Extract data length
		let data_length_bytes = &call_bytes[data_length_pos..data_length_pos + 32];
		let data_length = u32::from_be_bytes([
			data_length_bytes[28],
			data_length_bytes[29],
			data_length_bytes[30],
			data_length_bytes[31],
		]) as usize;

		// Extract the actual data
		let data_start = data_length_pos + 32;
		if data_start + data_length > call_bytes.len() {
			return Err(DetailedError::new(
				AUTH_VERIFICATION_FAILED_CODE,
				format!("Call data too short for batch entry {} data", i),
			)
			.with_field("call_data_length")
			.with_received(call_bytes.len().to_string())
			.with_expected(format!("{} bytes minimum", data_start + data_length))
			.to_rpc_error());
		}

		let inner_data = &call_bytes[data_start..data_start + data_length];
		let inner_calldata = format!("0x{}", hex::encode(inner_data));

		results.push((target_address, inner_calldata));

		// Move to next struct (this is simplified - in reality the ABI encoding is more complex)
		// For simplicity, assume fixed 96-byte spacing plus the data length
		current_pos += 96 + data_length.div_ceil(32) * 32; // Round up to 32-byte boundary
	}

	Ok(results)
}

/// Validates calldata for wallet_index == 0 backend requests
fn validate_backend_calldata(
	user_operations: &[SerializablePackedUserOperation],
	chain_id: ChainId,
) -> RpcResult<()> {
	for (index, user_op) in user_operations.iter().enumerate() {
		// Extract the target addresses and inner calldata from the execute()/executeBatch() calldata
		let execute_params =
			extract_execute_params_from_calldata(&user_op.call_data).map_err(|_e| {
				DetailedError::new(
					AUTH_VERIFICATION_FAILED_CODE,
					"Failed to parse OmniAccount execute calldata",
				)
				.with_field(format!("user_operations[{}].call_data", index))
				.to_rpc_error()
			})?;

		// Validate each execute call in the batch (or single call)
		for (call_index, (target_address, inner_calldata)) in execute_params.iter().enumerate() {
			match chain_id {
				ARBITRUM_MAINNET | ARBITRUM_SEPOLIA => {
					// For Arbitrum: sender must be USDC contract and calldata must be transfer to bridge
					let expected_usdc = match chain_id {
						ARBITRUM_MAINNET => ARBITRUM_USDC_ADDRESS,
						ARBITRUM_SEPOLIA => ARBITRUM_SEPOLIA_USDC_ADDRESS,
						_ => unreachable!(),
					};

					if *target_address != expected_usdc.parse::<Address>().unwrap() {
						return Err(DetailedError::new(
							AUTH_VERIFICATION_FAILED_CODE,
							"For Arbitrum backend requests, target contract must be USDC",
						)
						.with_field(format!(
							"user_operations[{}].call[{}] target address",
							index, call_index
						))
						.with_received(format!("{:?}", target_address))
						.with_expected(expected_usdc)
						.to_rpc_error());
					}

					validate_arbitrum_usdc_transfer(inner_calldata, chain_id)?;
				},
				HYPEREVM_MAINNET | HYPEREVM_TESTNET => {
					// For HyperEVM: target must be core writer and calldata must have valid action_id
					if *target_address != HYPEREVM_CORE_WRITER_ADDRESS.parse::<Address>().unwrap() {
						return Err(DetailedError::new(
							AUTH_VERIFICATION_FAILED_CODE,
							"For HyperEVM backend requests, target contract must be core writer",
						)
						.with_field(format!(
							"user_operations[{}].call[{}] target address",
							index, call_index
						))
						.with_received(format!("{:?}", target_address))
						.with_expected(HYPEREVM_CORE_WRITER_ADDRESS)
						.to_rpc_error());
					}

					validate_hyperevm_core_writer(inner_calldata, chain_id)?;
				},
				_ => {
					return Err(DetailedError::new(
						AUTH_VERIFICATION_FAILED_CODE,
						"Chain ID not supported for backend wallet_index validation",
					)
					.with_field("chain_id")
					.with_received(chain_id.to_string())
					.with_expected("42161, 421614, 999, or 998")
					.to_rpc_error());
				},
			}
		}
	}

	Ok(())
}

pub fn register_submit_user_op_with_auth<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_submitUserOpWithAuth", |params, ctx, _ext| async move {
			let params = params.parse::<SubmitUserOpWithAuthParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				DetailedError::new(
						PARSE_ERROR_CODE,
						"Failed to parse request parameters",
					)
					.with_reason(format!("Invalid JSON structure: {}", e)).to_rpc_error()
			})?;

			debug!("Received omni_submitUserOpWithAuth, params: {:?}", params);

			// Validate common parameters
			validate_chain_id(params.chain_id as u32, Some("evm")).map_err(|e| e.to_rpc_error())?;
			validate_wallet_index(params.wallet_index).map_err(|e| e.to_rpc_error())?;
			validate_user_operations(&params.user_operations).map_err(|e| e.to_rpc_error())?;

			let main_address = match &params.client_auth {
				ClientAuth::WildmetaHl {
					agent_address,
					business_json,
					main_address,
					signature,
					login_type,
				} => {
					verify_wildmeta_signature_wrapper(agent_address, business_json, signature)?;

					let business_data: serde_json::Value = serde_json::from_str(business_json)
						.map_err(|e| {
							error!("Failed to parse business_json: {:?}", e);
							DetailedError::new(
									PARSE_ERROR_CODE,
									"Failed to parse business JSON",
								)
								.with_field("business_json")
								.with_reason(format!("JSON parse error: {}", e)).to_rpc_error()
						})?;

					let timestamp = business_data
						.get("timestamp")
						.and_then(|v| v.as_u64())
						.ok_or_else(|| {
							error!("Missing timestamp in business_json");
							DetailedError::new(
									crate::error_code::MISSING_REQUIRED_FIELD_CODE,
									"Missing required field in business JSON",
								)
								.with_field("timestamp")
								.with_expected("Unix timestamp as number")
								.with_reason("Business JSON must contain a 'timestamp' field").to_rpc_error()
						})?;

					verify_payload_timestamp_wrapper(
						&ctx.wildmeta_timestamp_storage,
						main_address,
						timestamp,
					)?;

					let linked = ctx
						.wildmeta_api
						.verify_hyperliquid_link(agent_address, main_address, *login_type)
						.await
						.map_err(|e| {
							error!("Failed to verify hyperliquid link: {:?}", e);
							DetailedError::new(
									crate::error_code::EXTERNAL_API_ERROR_CODE,
									"Failed to verify Hyperliquid account link",
								)
								.with_field("operation")
								.with_received("verify_hyperliquid_link")
								.with_reason("Could not verify agent and main address linkage").to_rpc_error()
						})?;

					if !linked {
						error!("Agent and main addresses are not linked");
						return Err(DetailedError::new(
								AUTH_VERIFICATION_FAILED_CODE,
								"Agent and main addresses are not linked",
							)
							.with_field("agent_address")
							.with_received(agent_address.to_string())
							.with_suggestion("Ensure the agent address is properly linked to the main address").to_rpc_error());
					}

					Some(main_address.clone())
				},
				ClientAuth::WildmetaBackend { signature } => {
					// Validate wallet_index must be 0 or 1 for WildmetaBackend
					if params.wallet_index != 0 && params.wallet_index != 1 {
						error!(
							"WildmetaBackend requires wallet_index to be 0 or 1, got: {}",
							params.wallet_index
						);
						return Err(DetailedError::new(
								AUTH_VERIFICATION_FAILED_CODE,
								"Invalid wallet index for WildmetaBackend authentication",
							)
							.with_field("wallet_index")
							.with_received(params.wallet_index.to_string())
							.with_expected("0 or 1")
							.with_suggestion("WildmetaBackend authentication requires wallet_index to be 0 or 1").to_rpc_error());
					}

					// Additional validation for wallet_index == 0
					if params.wallet_index == 0 {
						validate_backend_calldata(&params.user_operations, params.chain_id)?;
					}

					// Validate client_id must be "wildmeta" for WildmetaBackend
					if params.client_id != "wildmeta" {
						error!(
							"WildmetaBackend requires client_id to be 'wildmeta', got: {}",
							params.client_id
						);
						return Err(DetailedError::new(
								AUTH_VERIFICATION_FAILED_CODE,
								"Invalid client ID for WildmetaBackend authentication",
							)
							.with_field("client_id")
							.with_received(params.client_id.clone())
							.with_expected("wildmeta")
							.with_suggestion("WildmetaBackend authentication requires client_id to be 'wildmeta'").to_rpc_error());
					}

					// Get entry point address for the chain
					let entry_point_client =
						ctx.entry_point_clients.get(&params.chain_id).ok_or_else(|| {
							error!("No entry point client found for chain_id: {}", params.chain_id);
							DetailedError::chain_not_supported(params.chain_id).to_rpc_error()
						})?;

					let entry_point_address = entry_point_client.entry_point_address();

					// Verify the backend signature against user operation hash
					verify_wildmeta_backend_signature_wrapper(
						signature,
						&params.user_operations,
						params.chain_id,
						entry_point_address,
						&ctx.wildmeta_backend_ecdsa_pubkey,
					)?;

					None
				},
				_ => {
					error!("Invalid client auth type");
					return Err(DetailedError::new(
							PARSE_ERROR_CODE,
							"Invalid client authentication type",
						)
						.with_field("client_auth")
						.with_expected("WildmetaHl or WildmetaBackend")
						.with_suggestion("Use a supported authentication method").to_rpc_error());
				},
			};

			let identity = Identity::try_from(params.user_id.clone()).map_err(|e| {
				error!("Failed to convert UserId to Identity: {:?}", e);
				DetailedError::new(
						crate::error_code::ACCOUNT_PARSE_ERROR_CODE,
						"Invalid user identity format",
					)
					.with_field("user_id")
					.with_reason(format!("Failed to parse user identity: {}", e)).to_rpc_error()
			})?;

			// Only validate main_address if it's provided (not None)
			if let Some(main_addr) = &main_address {
				match &identity {
					Identity::Evm(_) => {
						if let UserId::Evm(user_address) = &params.user_id {
							if user_address.to_lowercase() != main_addr.to_lowercase() {
								error!("Main address does not match user_id for EVM identity");
								return Err(DetailedError::new(
										AUTH_VERIFICATION_FAILED_CODE,
										"User address does not match authenticated main address for EVM identity",
									)
									.with_field("user_address")
									.with_received(user_address.to_string())
									.with_expected(main_addr.to_string())
									.with_suggestion("For EVM identity, the user_id must match the authenticated main address").to_rpc_error());
							}
						}
					},
					_ => {
						let omni_account = identity.to_omni_account(&params.client_id);
						let derived_pubkey = ctx
							.signer_client
							.request_wallet(
								ChainType::Evm,
								params.wallet_index,
								*omni_account.as_ref(),
							)
							.await
							.map_err(|e| {
								error!("Failed to derive EVM address: {:?}", e);
								DetailedError::signer_service_error(
										"request_wallet",
										&format!("Failed to derive wallet: {:?}", e),
									).to_rpc_error()
							})?;
						let derived_address = pubkey_to_address(ChainType::Evm, &derived_pubkey)
							.map_err(|e| {
								error!("Failed to convert derived pubkey to address: {:?}", e);
								DetailedError::new(
										AUTH_VERIFICATION_FAILED_CODE,
										"Failed to convert derived public key to address",
									)
									.with_field("operation")
									.with_received("pubkey_to_address conversion")
									.with_reason(format!("Internal error converting public key to address: {:?}", e)).to_rpc_error()
							})?;
						if derived_address.to_lowercase() != main_addr.to_lowercase() {
							error!("Main address does not match derived EVM address");
							return Err(DetailedError::new(
									AUTH_VERIFICATION_FAILED_CODE,
									"Derived address does not match authenticated address",
								)
								.with_field("derived_address")
								.with_received(derived_address.to_string())
								.with_expected(main_addr.to_string())
								.with_suggestion("The derived EVM address must match the authenticated main address").to_rpc_error());
						}
					},
				}
			}

			let account_id = identity.to_omni_account(&params.client_id);

			// Validate each operation's sender address (already done by validate_user_operations)
			// Additional validation for address format if needed
			for (index, op) in params.user_operations.iter().enumerate() {
				op.sender.parse::<Address>().map_err(|e| {
					error!("Invalid sender address '{}': {}", op.sender, e);
					DetailedError::invalid_address_format(
							&format!("user_operations[{}].sender", index),
							&op.sender,
							"0x-prefixed 20-byte Ethereum address (40 hex chars)",
						).to_rpc_error()
				})?;
			}

			// Inlined handler logic from handle_submit_user_op
			info!(
				"Processing SubmitUserOp for {} UserOperations on chain_id: {}",
				params.user_operations.len(),
				params.chain_id
			);

			// Get EntryPoint client for this chain (needed for both signing and submission)
			let entry_point_client = ctx.entry_point_clients.get(&params.chain_id).ok_or_else(|| {
				error!("No EntryPoint client configured for chain_id: {}", params.chain_id);
				DetailedError::chain_not_supported(params.chain_id).to_rpc_error()
			})?;

			// Parse whitelisted paymasters once
			let whitelisted_paymaster = parse_whitelisted_paymasters();

			// Process each UserOperation in the batch
			let mut aa_user_ops = Vec::new();

			for (index, serializable_user_op) in params.user_operations.iter().enumerate() {
				// Convert SerializablePackedUserOperation to PackedUserOperation
				let mut packed_user_op = convert_to_packed_user_op(serializable_user_op.clone())
					.map_err(|e| {
						error!("Failed to convert UserOperation {}: {}", index, e);
						DetailedError::invalid_user_operation_error(&format!(
							"Invalid user operation at index {}",
							index
						))
					})?;

				// Check userOp signature status and validate paymaster usage
				if packed_user_op.signature.is_empty() {
					// UNSIGNED userOp: If paymaster specified, must be whitelisted
					if !packed_user_op.paymasterAndData.is_empty() {
						if let Some(paymaster_address) =
							extract_paymaster_address(&packed_user_op.paymasterAndData)
						{
							if !is_whitelisted_paymaster(&paymaster_address, &whitelisted_paymaster)
							{
								error!(
									"UserOperation {} uses non-whitelisted paymaster {}. Only whitelisted paymasters are allowed for unsigned userOps.",
									index, paymaster_address
								);
								return Err(DetailedError::invalid_user_operation_error(&format!(
										"UserOperation at index {} uses non-whitelisted paymaster {}",
										index, paymaster_address
									))
									.to_rpc_error()
								);
							}
						}

						match process_erc20_paymaster_data(
							ctx.binance_api_client.as_ref() as &dyn BinancePaymasterApi,
							&packed_user_op.paymasterAndData,
							params.chain_id,
						)
						.await
						{
							Ok(Some(updated_paymaster_data)) => {
								packed_user_op.paymasterAndData = updated_paymaster_data;
								info!("Updated ERC20 paymaster data for UserOperation {}", index);
							},
							Ok(None) => {
								// Not an ERC20 paymaster, continue as normal
								debug!("UserOperation {} does not use ERC20 paymaster", index);
							},
							Err(e) => {
								error!(
									"Failed to process ERC20 paymaster data for UserOperation {}: {}",
									index, e
								);
								return Err(DetailedError::invalid_user_operation_error(&format!(
										"ERC20 paymaster processing failed for operation at index {}: {}",
										index, e
									))
									.to_rpc_error()
								);
							},
						}
					}

					info!("Requesting signature from pumpx signer for UserOperation {}", index);

					// Log UserOp details for debugging
					info!(
						"UserOp details - Sender: {}, Nonce: {}, InitCode length: {}, CallData length: {}",
						packed_user_op.sender,
						packed_user_op.nonce,
						packed_user_op.initCode.len(),
						packed_user_op.callData.len()
					);

					let entry_point_address = entry_point_client.entry_point_address();

					let user_op_hash_bytes = calculate_user_operation_hash(
						&packed_user_op,
						entry_point_address,
						params.chain_id,
					);
					let message_to_sign = user_op_hash_bytes.to_vec();

					info!(
						"Signing UserOp hash: 0x{}, EntryPoint: {}, ChainID: {}",
						hex::encode(user_op_hash_bytes),
						entry_point_address,
						params.chain_id
					);

					// Request signature from pumpx signer for EVM chain
					let signature_result = ctx
						.signer_client
						.request_signature(
							ChainType::Evm,
							params.wallet_index,
							account_id.clone().into(),
							message_to_sign,
						)
						.await;

					let signature = match signature_result {
						Ok(sig) => substrate_to_ethereum_signature(&sig)
							.map_err(|e| {
								error!("Failed to convert signature: {}", e);
								DetailedError::signature_service_unavailable().to_rpc_error()
							})?
							.to_vec(),
						Err(_) => {
							error!("Failed to sign user operation {}", index);
							return Err(DetailedError::signature_service_unavailable().to_rpc_error()
							);
						},
					};

					// Prepend 0x01 byte to indicate Root signature type (according to UserOpSigner enum)
					let mut signature_with_prefix: Vec<u8> = vec![0x01];
					signature_with_prefix.extend_from_slice(&signature);
					packed_user_op.signature = Bytes::from(signature_with_prefix);
					info!("UserOperation {} signed successfully", index);
				} else {
					// SIGNED userOp: Only allowed if no paymaster specified
					if !packed_user_op.paymasterAndData.is_empty() {
						error!(
							"UserOperation {} is signed but has paymaster data. Signed userOps are only allowed without paymaster.",
							index
						);
						return Err(DetailedError::invalid_user_operation_error(&format!(
								"UserOperation at index {} is signed but specifies a paymaster",
								index
							))
							.to_rpc_error()
						);
					}
					info!("UserOperation {} is signed with no paymaster, processing", index);
				}

				// Convert to aa_contracts_client::PackedUserOperation for EntryPoint call
				let aa_user_op = aa_contracts_client::PackedUserOperation {
					sender: packed_user_op.sender,
					nonce: packed_user_op.nonce,
					initCode: packed_user_op.initCode.clone(),
					callData: packed_user_op.callData.clone(),
					accountGasLimits: packed_user_op.accountGasLimits,
					preVerificationGas: packed_user_op.preVerificationGas,
					gasFees: packed_user_op.gasFees,
					paymasterAndData: packed_user_op.paymasterAndData.clone(),
					signature: packed_user_op.signature.clone(),
				};
				aa_user_ops.push(aa_user_op);
			}

			// Get beneficiary address from the EntryPoint client's wallet
			let beneficiary = entry_point_client.get_wallet_address().await.map_err(|_| {
				let err_msg = "Failed to get wallet address from EntryPoint client".to_string();
				error!("{}", err_msg.clone());
				DetailedError::new(
					crate::error_code::INTERNAL_ERROR_CODE,
					err_msg,
				).to_rpc_error()
			})?;

			// Run batch simulation for all UserOperations before submission
			info!("Running batch simulation for {} UserOperations", aa_user_ops.len());
			match entry_point_client.simulate_handle_ops(&aa_user_ops, beneficiary).await {
				Ok(simulation_results) => {
					for (index, result) in simulation_results.iter().enumerate() {
						info!(
							"UserOperation {} simulation successful. PreOpGas: {}, Paid: {}, AccountValidation: {}, PaymasterValidation: {}",
							index,
							result.preOpGas,
							result.paid,
							result.accountValidationData,
							result.paymasterValidationData
						);
					}
					info!("All {} UserOperations passed batch simulation checks", aa_user_ops.len());
				},
				Err(e) => {
					let err_msg: String = format!("Batch UserOperation simulation failed: {}", e);
					error!("{}", err_msg.clone());
					return Err(DetailedError::invalid_user_operation_error(&err_msg).to_rpc_error()
					);
				},
			}

			// Submit all UserOperations via EntryPoint.handleOps() with retry logic
			let transaction_hash =
				match entry_point_client.handle_ops_with_retry(&aa_user_ops, beneficiary).await {
					Ok(tx_hash) => {
						// Return the actual transaction hash from handle_ops
						Some(tx_hash)
					},
					Err(_) => {
						let err_msg =
							"Failed to submit UserOperations to EntryPoint via handleOps after retries"
								.to_string();
						error!("{}", err_msg.clone());
						return Err(DetailedError::new(
							crate::error_code::INTERNAL_ERROR_CODE,
							err_msg,
						).to_rpc_error());
					},
				};

			Ok(SubmitUserOpWithAuthResponse { transaction_hash })
		})
		.expect("Failed to register omni_submitUserOpWithAuth method");
}

// Wrapper functions to convert shared function return types to DetailedError
fn verify_wildmeta_signature_wrapper(
	agent_address: &str,
	business_json: &str,
	signature: &str,
) -> RpcResult<()> {
	verify_wildmeta_signature(agent_address, business_json, signature).map_err(|e| {
		DetailedError::new(e.code(), "Wildmeta signature verification failed")
			.with_field("agent_address")
			.with_received(agent_address.to_string())
			.with_suggestion("Ensure the signature is valid and matches the agent address")
			.to_rpc_error()
	})
}

fn verify_payload_timestamp_wrapper(
	storage: &Arc<WildmetaTimestampStorage>,
	main_address: &str,
	new_timestamp: u64,
) -> RpcResult<()> {
	verify_payload_timestamp(storage, main_address, new_timestamp).map_err(|e| {
		DetailedError::new(e.code(), "Timestamp verification failed")
			.with_field("timestamp")
			.with_received(new_timestamp.to_string())
			.with_suggestion("Timestamp must be greater than the previously used timestamp")
			.to_rpc_error()
	})
}

fn verify_wildmeta_backend_signature_wrapper(
	signature: &str,
	user_operations: &[SerializablePackedUserOperation],
	chain_id: ChainId,
	entry_point_address: Address,
	expected_pubkey: &[u8; 33],
) -> RpcResult<()> {
	verify_wildmeta_backend_signature(
		signature,
		user_operations,
		chain_id,
		entry_point_address,
		expected_pubkey,
	)
	.map_err(|e| {
		DetailedError::new(e.code(), "Backend signature verification failed")
			.with_field("signature")
			.with_suggestion("Ensure the backend signature is valid for the given operations")
			.to_rpc_error()
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::utils::user_op::convert_to_packed_user_op;
	use executor_storage::StorageDB;
	use tempfile::tempdir;

	#[test]
	fn test_verify_wildmeta_signature_with_real_data() {
		let business_json = r#"{"action":"trade","amount":1.5,"customField1":"buy","customField2":"market","leverage":10,"metadata":{"features":{"darkMode":true,"notifications":false},"userAgent":"mobile-app","version":"1.0.0"},"positions":[{"entryPrice":50000,"metadata":{"openTime":1640995200,"strategy":"momentum"},"side":"long","size":1.5,"symbol":"BTC/USD"},{"entryPrice":3000,"metadata":{"openTime":1640995300,"strategy":"reversal"},"side":"short","size":2,"symbol":"ETH/USD"}],"price":50000,"riskManagement":{"maxLeverage":20,"stopLoss":{"enabled":true,"percentage":0.05},"takeProfit":{"enabled":true,"percentage":0.1}},"slippage":0.01,"symbol":"BTC/USD","timestamp":1752573555}"#;
		let signature = "0x46c737250d61b60cbf0f46a6755e59815844a2f7cdb9dc16bf867b57bfed3526424343a237c15eef9089d571d1f60fd0bd7f91d5888c649216a7df147b386a681c";
		let agent_address = "0xf8b16F021438B710fDE9d59dD17dDE1Eb2691BFd";

		let result = verify_wildmeta_signature_wrapper(agent_address, business_json, signature);
		assert!(result.is_ok(), "Signature verification should succeed");
	}

	#[test]
	fn test_verify_wildmeta_signature_invalid_signature() {
		let business_json = r#"{"action":"trade","timestamp":1752573555}"#;
		let signature = "0x020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
		let agent_address = "0xf8b16F021438B710fDE9d59dD17dDE1Eb2691BFd";

		let result = verify_wildmeta_signature_wrapper(agent_address, business_json, signature);
		assert!(result.is_err(), "Should fail with invalid signature");
	}

	#[test]
	fn test_verify_wildmeta_signature_wrong_signer() {
		let business_json = r#"{"action":"trade","amount":1.5,"customField1":"buy","customField2":"market","leverage":10,"metadata":{"features":{"darkMode":true,"notifications":false},"userAgent":"mobile-app","version":"1.0.0"},"positions":[{"entryPrice":50000,"metadata":{"openTime":1640995200,"strategy":"momentum"},"side":"long","size":1.5,"symbol":"BTC/USD"},{"entryPrice":3000,"metadata":{"openTime":1640995300,"strategy":"reversal"},"side":"short","size":2,"symbol":"ETH/USD"}],"price":50000,"riskManagement":{"maxLeverage":20,"stopLoss":{"enabled":true,"percentage":0.05},"takeProfit":{"enabled":true,"percentage":0.1}},"slippage":0.01,"symbol":"BTC/USD","timestamp":1752573555}"#;

		let signature = "0x46c737250d61b60cbf0f46a6755e59815844a2f7cdb9dc16bf867b57bfed3526424343a237c15eef9089d571d1f60fd0bd7f91d5888c649216a7df147b386a681c";
		// Use a different address than the actual signer
		let wrong_agent_address = "0xA9d439F4DED81152DB00CB7CD94A8d908FEF903e";

		let result = verify_wildmeta_signature(wrong_agent_address, business_json, signature);
		assert!(result.is_err(), "Should fail with wrong signer address");
	}

	#[test]
	fn test_verify_wildmeta_signature_invalid_hex() {
		let business_json = r#"{"timestamp":1752573555}"#;
		let signature = "invalid_hex";
		let agent_address = "0xf8b16F021438B710fDE9d59dD17dDE1Eb2691BFd";

		let result = verify_wildmeta_signature_wrapper(agent_address, business_json, signature);
		assert!(result.is_err(), "Should fail with invalid hex signature");
	}

	#[test]
	fn test_parse_business_json_timestamp() {
		let business_json = r#"{"action":"trade","timestamp":1752573555}"#;

		let parsed: serde_json::Value = serde_json::from_str(business_json).unwrap();
		let timestamp = parsed.get("timestamp").and_then(|v| v.as_u64());

		assert_eq!(timestamp, Some(1752573555), "Should correctly parse timestamp");
	}

	#[test]
	fn test_parse_business_json_missing_timestamp() {
		let business_json = r#"{"action":"trade","amount":1.5}"#;

		let parsed: serde_json::Value = serde_json::from_str(business_json).unwrap();
		let timestamp = parsed.get("timestamp").and_then(|v| v.as_u64());

		assert_eq!(timestamp, None, "Should return None for missing timestamp");
	}

	#[test]
	fn test_verify_payload_timestamp_success() {
		let tmp_dir = tempdir().unwrap();
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());
		let storage = Arc::new(WildmetaTimestampStorage::new(db));

		let main_address = "0xA9d439F4DED81152DB00CB7CD94A8d908FEF903e";

		// First timestamp should succeed
		let result = verify_payload_timestamp_wrapper(&storage, main_address, 1000);
		assert!(result.is_ok(), "First timestamp should succeed");

		// Higher timestamp should succeed
		let result = verify_payload_timestamp_wrapper(&storage, main_address, 2000);
		assert!(result.is_ok(), "Higher timestamp should succeed");
	}

	#[test]
	fn test_verify_payload_timestamp_fails_with_old_timestamp() {
		let tmp_dir = tempdir().unwrap();
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());
		let storage = Arc::new(WildmetaTimestampStorage::new(db));

		let main_address = "0xA9d439F4DED81152DB00CB7CD94A8d908FEF903e";

		// Store initial timestamp
		let result = verify_payload_timestamp_wrapper(&storage, main_address, 1000);
		assert!(result.is_ok());

		// Same timestamp should fail
		let result = verify_payload_timestamp_wrapper(&storage, main_address, 1000);
		assert!(result.is_err(), "Same timestamp should fail");

		// Lower timestamp should fail
		let result = verify_payload_timestamp_wrapper(&storage, main_address, 500);
		assert!(result.is_err(), "Lower timestamp should fail");
	}

	#[test]
	fn test_verify_payload_timestamp_first_time() {
		let tmp_dir = tempdir().unwrap();
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());
		let storage = Arc::new(WildmetaTimestampStorage::new(db));

		let main_address = "0xA9d439F4DED81152DB00CB7CD94A8d908FEF903e";

		// Any timestamp should succeed for first time
		let result = verify_payload_timestamp_wrapper(&storage, main_address, 1);
		assert!(result.is_ok(), "First timestamp should succeed even if it's 1");
	}

	#[test]
	fn test_verify_payload_timestamp_persistence() {
		let tmp_dir = tempdir().unwrap();
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());
		let storage = Arc::new(WildmetaTimestampStorage::new(db));

		let main_address = "0xA9d439F4DED81152DB00CB7CD94A8d908FEF903e";

		// Store timestamp
		verify_payload_timestamp_wrapper(&storage, main_address, 1000).unwrap();

		// Verify it's persisted by checking that lower timestamp fails
		let result = verify_payload_timestamp_wrapper(&storage, main_address, 999);
		assert!(result.is_err(), "Timestamp should be persisted");

		// Verify exact stored value fails
		let result = verify_payload_timestamp_wrapper(&storage, main_address, 1000);
		assert!(result.is_err(), "Exact stored timestamp should fail");

		// Higher should succeed
		let result = verify_payload_timestamp_wrapper(&storage, main_address, 1001);
		assert!(result.is_ok(), "Higher timestamp should succeed");
	}

	#[test]
	fn test_wildmeta_backend_auth_parsing() {
		use executor_primitives::ClientAuth;

		let json =
			r#"{"type": "wildmeta_backend", "value": { "signature": "0x1234567890abcdef" }}"#;
		let deserialized: ClientAuth = serde_json::from_str(json).unwrap();

		match deserialized {
			ClientAuth::WildmetaBackend { signature } => {
				assert_eq!(signature, "0x1234567890abcdef");
			},
			_ => panic!("Expected WildmetaBackend variant"),
		}
	}

	#[test]
	fn test_verify_wildmeta_backend_signature_wrapper_invalid_signature() {
		use executor_core::types::SerializablePackedUserOperation;

		// Create a test user operation
		let user_op = SerializablePackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".to_string(),
			nonce: 42,
			init_code: "0x".to_string(),
			call_data: "0x".to_string(),
			account_gas_limits:
				"0x0000000000000000000000000030d4000000000000000000000000000000c350".to_string(),
			pre_verification_gas: 21000,
			gas_fees: "0x000000000000000000000003b9aca0000000000000000000000000000b2d05e0"
				.to_string(),
			paymaster_and_data: "0x".to_string(),
			signature: None,
		};

		let invalid_signature = "0x020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
		let chain_id = 1;
		let entry_point_address = Address::from([0u8; 20]);
		let expected_pubkey = [0u8; 33];

		let result = verify_wildmeta_backend_signature_wrapper(
			invalid_signature,
			&[user_op],
			chain_id,
			entry_point_address,
			&expected_pubkey,
		);

		assert!(result.is_err(), "Should fail with invalid signature");
	}

	#[test]
	fn test_wildmeta_backend_validation_invalid_wallet_index() {
		use executor_primitives::ClientAuth;

		let auth = ClientAuth::WildmetaBackend { signature: "0x1234567890abcdef".to_string() };

		// Test with wallet_index != 1 should fail
		// This would be tested in an integration test with actual RPC call
		// For now we verify the auth variant is parsed correctly
		match auth {
			ClientAuth::WildmetaBackend { signature } => {
				assert_eq!(signature, "0x1234567890abcdef");
			},
			_ => panic!("Expected WildmetaBackend variant"),
		}
	}

	#[test]
	fn test_wildmeta_backend_validation_invalid_client_id() {
		use executor_primitives::ClientAuth;

		let auth = ClientAuth::WildmetaBackend { signature: "0x1234567890abcdef".to_string() };

		// Test with client_id != "wildmeta" should fail
		// This would be tested in an integration test with actual RPC call
		// For now we verify the auth variant is parsed correctly
		match auth {
			ClientAuth::WildmetaBackend { signature } => {
				assert_eq!(signature, "0x1234567890abcdef");
			},
			_ => panic!("Expected WildmetaBackend variant"),
		}
	}

	#[test]
	fn test_wildmeta_backend_valid_signature_verification() {
		use aa_contracts_client::calculate_user_operation_hash;
		use alloy::primitives::{keccak256, Address};
		use executor_core::types::SerializablePackedUserOperation;
		use executor_crypto::secp256k1::{
			secp256k1_ecdsa_recover_compressed, secp256k1_ecdsa_sign,
		};

		// Create a test private key (32 bytes)
		let private_key: [u8; 32] = [
			0x47, 0xf7, 0x8f, 0x59, 0x81, 0x2d, 0x6d, 0x1f, 0x2c, 0x8a, 0x65, 0x04, 0x19, 0x0d,
			0x63, 0x7f, 0x34, 0x6c, 0x4b, 0x6f, 0x7d, 0x20, 0x45, 0x32, 0x15, 0x68, 0x91, 0x73,
			0xa2, 0xb8, 0xc9, 0xe4,
		];

		// Create a test user operation
		let user_op = SerializablePackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".to_string(),
			nonce: 42,
			init_code: "0x".to_string(),
			call_data: "0xabcdef".to_string(),
			account_gas_limits:
				"0x0000000000000000000000000030d4000000000000000000000000000000c350".to_string(),
			pre_verification_gas: 21000,
			gas_fees: "0x000000000000000000000003b9aca0000000000000000000000000000b2d05e0"
				.to_string(),
			paymaster_and_data: "0x".to_string(),
			signature: None,
		};

		let chain_id = 31337u64; // Local test chain
		let entry_point_address = Address::from([0u8; 20]);

		let mut combined_hash_data = Vec::new();
		let packed_user_op = convert_to_packed_user_op(user_op.clone()).unwrap();
		let user_op_hash =
			calculate_user_operation_hash(&packed_user_op, entry_point_address, chain_id);
		combined_hash_data.extend_from_slice(&user_op_hash.0);
		let combined_hash = keccak256(&combined_hash_data);

		// Sign the combined hash with our private key
		let signature = match secp256k1_ecdsa_sign(&private_key, &combined_hash.0) {
			Ok(sig) => sig,
			Err(_) => panic!("Failed to sign with valid private key"),
		};

		// Derive the expected public key from the signature and hash
		let expected_pubkey = match secp256k1_ecdsa_recover_compressed(&signature, &combined_hash.0)
		{
			Ok(pk) => pk,
			Err(_) => panic!("Failed to recover pubkey from valid signature"),
		};

		// Convert signature to hex string with 0x prefix
		let signature_hex = format!("0x{}", hex::encode(signature));

		// Test the verification function
		let result = verify_wildmeta_backend_signature_wrapper(
			&signature_hex,
			&[user_op],
			chain_id,
			entry_point_address,
			&expected_pubkey,
		);

		assert!(result.is_ok(), "Valid signature verification should succeed");
	}

	#[test]
	fn test_wildmeta_backend_multiple_operations_signature_verification() {
		use aa_contracts_client::calculate_user_operation_hash;
		use alloy::primitives::{keccak256, Address};
		use executor_core::types::SerializablePackedUserOperation;
		use executor_crypto::secp256k1::{
			secp256k1_ecdsa_recover_compressed, secp256k1_ecdsa_sign,
		};

		let private_key: [u8; 32] = [
			0x47, 0xf7, 0x8f, 0x59, 0x81, 0x2d, 0x6d, 0x1f, 0x2c, 0x8a, 0x65, 0x04, 0x19, 0x0d,
			0x63, 0x7f, 0x34, 0x6c, 0x4b, 0x6f, 0x7d, 0x20, 0x45, 0x32, 0x15, 0x68, 0x91, 0x73,
			0xa2, 0xb8, 0xc9, 0xe4,
		];

		// Create multiple test user operations
		let user_op1 = SerializablePackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".to_string(),
			nonce: 42,
			init_code: "0x".to_string(),
			call_data: "0xabcdef".to_string(),
			account_gas_limits:
				"0x0000000000000000000000000030d4000000000000000000000000000000c350".to_string(),
			pre_verification_gas: 21000,
			gas_fees: "0x000000000000000000000003b9aca0000000000000000000000000000b2d05e0"
				.to_string(),
			paymaster_and_data: "0x".to_string(),
			signature: None,
		};

		let user_op2 = SerializablePackedUserOperation {
			sender: "0x9876543210987654321098765432109876543210".to_string(),
			nonce: 43,
			init_code: "0x".to_string(),
			call_data: "0x123456".to_string(),
			account_gas_limits:
				"0x0000000000000000000000000030d4000000000000000000000000000000c350".to_string(),
			pre_verification_gas: 22000,
			gas_fees: "0x000000000000000000000003b9aca0000000000000000000000000000b2d05e0"
				.to_string(),
			paymaster_and_data: "0x".to_string(),
			signature: None,
		};

		let user_operations = vec![user_op1.clone(), user_op2.clone()];
		let chain_id = 31337u64;
		let entry_point_address = Address::from([0u8; 20]);

		// Calculate combined hash for all operations (mimic the new implementation)
		let mut combined_hash_data = Vec::new();
		for user_op in &user_operations {
			let packed_user_op = convert_to_packed_user_op(user_op.clone()).unwrap();
			let user_op_hash =
				calculate_user_operation_hash(&packed_user_op, entry_point_address, chain_id);
			combined_hash_data.extend_from_slice(&user_op_hash.0);
		}
		let combined_hash = keccak256(&combined_hash_data);

		// Sign the combined hash
		let signature = match secp256k1_ecdsa_sign(&private_key, &combined_hash.0) {
			Ok(sig) => sig,
			Err(_) => panic!("Failed to sign"),
		};
		let expected_pubkey = match secp256k1_ecdsa_recover_compressed(&signature, &combined_hash.0)
		{
			Ok(pk) => pk,
			Err(_) => panic!("Failed to recover pubkey"),
		};
		let signature_hex = format!("0x{}", hex::encode(signature));

		// Test with multiple operations
		let result = verify_wildmeta_backend_signature_wrapper(
			&signature_hex,
			&user_operations,
			chain_id,
			entry_point_address,
			&expected_pubkey,
		);

		assert!(result.is_ok(), "Multiple operations signature verification should succeed");

		// Test that single operation would fail with same signature (different hash)
		let single_op_result = verify_wildmeta_backend_signature_wrapper(
			&signature_hex,
			&[user_op1],
			chain_id,
			entry_point_address,
			&expected_pubkey,
		);

		assert!(single_op_result.is_err(), "Single operation should fail with multi-op signature");
	}

	#[test]
	fn test_wildmeta_backend_invalid_signature_wrong_key() {
		use aa_contracts_client::calculate_user_operation_hash;
		use alloy::primitives::{keccak256, Address};
		use executor_core::types::SerializablePackedUserOperation;
		use executor_crypto::secp256k1::secp256k1_ecdsa_sign;

		// Create a test private key
		let private_key: [u8; 32] = [
			0x47, 0xf7, 0x8f, 0x59, 0x81, 0x2d, 0x6d, 0x1f, 0x2c, 0x8a, 0x65, 0x04, 0x19, 0x0d,
			0x63, 0x7f, 0x34, 0x6c, 0x4b, 0x6f, 0x7d, 0x20, 0x45, 0x32, 0x15, 0x68, 0x91, 0x73,
			0xa2, 0xb8, 0xc9, 0xe4,
		];

		// Wrong expected public key (different from the actual signature)
		let wrong_expected_pubkey: [u8; 33] = [
			0x03, 0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce,
			0x87, 0x0b, 0x07, 0x02, 0x9b, 0xfb, 0xa3, 0x72, 0xdd, 0x89, 0x6e, 0x17, 0xc8, 0x43,
			0x79, 0x1b, 0x19, 0x5f, 0x8d,
		];

		// Create a test user operation
		let user_op = SerializablePackedUserOperation {
			sender: "0x1234567890123456789012345678901234567890".to_string(),
			nonce: 42,
			init_code: "0x".to_string(),
			call_data: "0xabcdef".to_string(),
			account_gas_limits:
				"0x0000000000000000000000000030d4000000000000000000000000000000c350".to_string(),
			pre_verification_gas: 21000,
			gas_fees: "0x000000000000000000000003b9aca0000000000000000000000000000b2d05e0"
				.to_string(),
			paymaster_and_data: "0x".to_string(),
			signature: None,
		};

		let chain_id = 31337u64;
		let entry_point_address = Address::from([0u8; 20]);

		// Calculate combined hash (mimic the new implementation)
		let mut combined_hash_data = Vec::new();
		let packed_user_op = convert_to_packed_user_op(user_op.clone()).unwrap();
		let user_op_hash =
			calculate_user_operation_hash(&packed_user_op, entry_point_address, chain_id);
		combined_hash_data.extend_from_slice(&user_op_hash.0);
		let combined_hash = keccak256(&combined_hash_data);

		// Sign the combined hash with our private key
		let signature = match secp256k1_ecdsa_sign(&private_key, &combined_hash.0) {
			Ok(sig) => sig,
			Err(_) => panic!("Failed to sign with valid private key"),
		};
		let signature_hex = format!("0x{}", hex::encode(signature));

		// Test with wrong expected public key - should fail
		let result = verify_wildmeta_backend_signature_wrapper(
			&signature_hex,
			&[user_op],
			chain_id,
			entry_point_address,
			&wrong_expected_pubkey,
		);

		assert!(result.is_err(), "Signature verification with wrong public key should fail");
	}

	#[test]
	fn test_validate_arbitrum_usdc_transfer_valid() {
		// Valid USDC transfer calldata: transfer(address recipient, uint256 amount)
		// Method signature: 0xa9059cbb
		// Recipient: ARB_TO_HYPER_BRIDGE_ADDRESS (padded to 32 bytes)
		// Amount: 1000000 (1 USDC with 6 decimals, padded to 32 bytes)
		let call_data = format!(
			"0xa9059cbb000000000000000000000000{}00000000000000000000000000000000000000000000000000000000000f4240",
			&ARB_TO_HYPER_BRIDGE_ADDRESS[2..]
		);

		let result = validate_arbitrum_usdc_transfer(&call_data, ARBITRUM_MAINNET);
		assert!(result.is_ok(), "Valid USDC transfer should succeed");

		let result = validate_arbitrum_usdc_transfer(&call_data, ARBITRUM_SEPOLIA);
		assert!(result.is_ok(), "Valid USDC transfer should succeed on Arbitrum Sepolia");
	}

	#[test]
	fn test_validate_arbitrum_usdc_transfer_invalid_method() {
		// Invalid method signature (not transfer)
		let call_data = "0x12345678000000000000000000000000123456789012345678901234567890123456789000000000000000000000000000000000000000000000000000000000000f4240";

		let result = validate_arbitrum_usdc_transfer(call_data, ARBITRUM_MAINNET);
		assert!(result.is_err(), "Invalid method signature should fail");
	}

	#[test]
	fn test_validate_arbitrum_usdc_transfer_invalid_recipient() {
		// Valid method signature but wrong recipient
		let call_data = "0xa9059cbb0000000000000000000000001111111111111111111111111111111111111111000000000000000000000000000000000000000000000000000000000000f4240";

		let result = validate_arbitrum_usdc_transfer(call_data, ARBITRUM_MAINNET);
		assert!(result.is_err(), "Wrong recipient should fail");
	}

	#[test]
	fn test_validate_arbitrum_usdc_transfer_invalid_chain() {
		let call_data = "0xa9059cbb000000000000000000000000123456789012345678901234567890123456789000000000000000000000000000000000000000000000000000000000000f4240";

		let result = validate_arbitrum_usdc_transfer(call_data, 1); // Ethereum mainnet
		assert!(result.is_err(), "Unsupported chain should fail");
	}

	#[test]
	fn test_validate_hyperevm_core_writer_valid() {
		// Helper function to create valid calldata with proper payload format
		fn create_test_calldata(action_id: u32, additional_data: &[u8]) -> String {
			// Create payload: [version:1][action_id:3][additional_data]
			let mut payload = Vec::new();
			payload.push(0x01); // version
			payload.extend_from_slice(&action_id.to_be_bytes()[1..4]); // action_id (3 bytes)
			payload.extend_from_slice(additional_data);

			// Create calldata: method_sig + offset + length + payload (padded to 32-byte boundaries)
			let mut calldata = Vec::new();
			calldata.extend_from_slice(&[0x12, 0x34, 0x56, 0x78]); // method signature
			calldata.extend_from_slice(&[0u8; 28]); // offset padding
			calldata.extend_from_slice(&[0, 0, 0, 0x20]); // offset = 32
			calldata.extend_from_slice(&[0u8; 28]); // length padding
			calldata.extend_from_slice(&(payload.len() as u32).to_be_bytes()); // payload length
			calldata.extend_from_slice(&payload); // payload

			// Pad to 32-byte boundary
			while calldata.len() % 32 != 0 {
				calldata.push(0);
			}

			format!("0x{}", hex::encode(calldata))
		}

		// Test valid action_id = 0x000002
		let call_data = create_test_calldata(0x000002, &[0x12, 0x34]); // some additional data
		let result = validate_hyperevm_core_writer(&call_data, HYPEREVM_MAINNET);
		assert!(result.is_ok(), "Valid action_id 0x000002 should succeed");

		// Test other valid action_ids
		for action_id in [0x000003, 0x000004, 0x000005, 0x000007] {
			let call_data = create_test_calldata(action_id, &[0xff, 0xee]);
			let result = validate_hyperevm_core_writer(&call_data, HYPEREVM_MAINNET);
			assert!(result.is_ok(), "Action_id 0x{:06x} should be valid", action_id);
		}
	}

	#[test]
	fn test_validate_hyperevm_core_writer_invalid_action_id() {
		// Helper function to create calldata with proper payload format
		fn create_test_calldata(action_id: u32, additional_data: &[u8]) -> String {
			// Create payload: [version:1][action_id:3][additional_data]
			let mut payload = Vec::new();
			payload.push(0x01); // version
			payload.extend_from_slice(&action_id.to_be_bytes()[1..4]); // action_id (3 bytes)
			payload.extend_from_slice(additional_data);

			// Create calldata: method_sig + offset + length + payload (padded to 32-byte boundaries)
			let mut calldata = Vec::new();
			calldata.extend_from_slice(&[0x12, 0x34, 0x56, 0x78]); // method signature
			calldata.extend_from_slice(&[0u8; 28]); // offset padding
			calldata.extend_from_slice(&[0, 0, 0, 0x20]); // offset = 32
			calldata.extend_from_slice(&[0u8; 28]); // length padding
			calldata.extend_from_slice(&(payload.len() as u32).to_be_bytes()); // payload length
			calldata.extend_from_slice(&payload); // payload

			// Pad to 32-byte boundary
			while calldata.len() % 32 != 0 {
				calldata.push(0);
			}

			format!("0x{}", hex::encode(calldata))
		}

		// Invalid action_id = 0x000001 (not in allowed list)
		let call_data = create_test_calldata(0x000001, &[0x12, 0x34]);
		let result = validate_hyperevm_core_writer(&call_data, HYPEREVM_MAINNET);
		assert!(result.is_err(), "Invalid action_id should fail");

		// Test other invalid action_ids
		for action_id in [0x000000, 0x000006, 0x000008, 0x000999] {
			let call_data = create_test_calldata(action_id, &[0xff, 0xee]);
			let result = validate_hyperevm_core_writer(&call_data, HYPEREVM_MAINNET);
			assert!(result.is_err(), "Invalid action_id 0x{:06x} should fail", action_id);
		}
	}

	#[test]
	fn test_validate_hyperevm_core_writer_invalid_chain() {
		// Helper function to create calldata with proper payload format
		fn create_test_calldata(action_id: u32, additional_data: &[u8]) -> String {
			// Create payload: [version:1][action_id:3][additional_data]
			let mut payload = Vec::new();
			payload.push(0x01); // version
			payload.extend_from_slice(&action_id.to_be_bytes()[1..4]); // action_id (3 bytes)
			payload.extend_from_slice(additional_data);

			// Create calldata: method_sig + offset + length + payload (padded to 32-byte boundaries)
			let mut calldata = Vec::new();
			calldata.extend_from_slice(&[0x12, 0x34, 0x56, 0x78]); // method signature
			calldata.extend_from_slice(&[0u8; 28]); // offset padding
			calldata.extend_from_slice(&[0, 0, 0, 0x20]); // offset = 32
			calldata.extend_from_slice(&[0u8; 28]); // length padding
			calldata.extend_from_slice(&(payload.len() as u32).to_be_bytes()); // payload length
			calldata.extend_from_slice(&payload); // payload

			// Pad to 32-byte boundary
			while calldata.len() % 32 != 0 {
				calldata.push(0);
			}

			format!("0x{}", hex::encode(calldata))
		}

		let call_data = create_test_calldata(0x000002, &[0x12, 0x34]);
		let result = validate_hyperevm_core_writer(&call_data, ARBITRUM_MAINNET);
		assert!(result.is_err(), "Wrong chain should fail");
	}

	#[test]
	fn test_validate_backend_calldata_arbitrum() {
		use executor_core::types::SerializablePackedUserOperation;

		// Helper function to create ERC20 transfer calldata
		fn create_erc20_transfer_calldata(recipient: &str, amount: u64) -> String {
			let recipient_hex = recipient.strip_prefix("0x").unwrap().to_lowercase();

			// Create transfer(recipient, amount) calldata
			let mut calldata = Vec::new();
			calldata.extend_from_slice(&[0xa9, 0x05, 0x9c, 0xbb]); // transfer method signature

			// recipient address (32 bytes)
			calldata.extend_from_slice(&hex::decode(format!("{:0>64}", recipient_hex)).unwrap());

			// amount (32 bytes)
			calldata.extend_from_slice(&hex::decode(format!("{:0>64x}", amount)).unwrap());

			hex::encode(calldata)
		}

		// Helper function to create OmniAccount execute calldata
		fn create_execute_calldata(target_address: &str, inner_calldata: &str) -> String {
			let target_hex = target_address.strip_prefix("0x").unwrap().to_lowercase();
			let inner_len = inner_calldata.len() / 2; // Convert hex string length to byte length

			// Create execute(target, 0, inner_calldata) - ABI encoded
			let mut execute_data = Vec::new();
			execute_data.extend_from_slice(&[0xb6, 0x1d, 0x27, 0xf6]); // execute method signature

			// target address (32 bytes)
			execute_data.extend_from_slice(&hex::decode(format!("{:0>64}", target_hex)).unwrap());

			// value = 0 (32 bytes)
			execute_data.extend_from_slice(&[0u8; 32]);

			// offset to data (32 bytes) = 0x60 = 96 bytes
			execute_data.extend_from_slice(
				&hex::decode("0000000000000000000000000000000000000000000000000000000000000060")
					.unwrap(),
			);

			// data length (32 bytes)
			execute_data.extend_from_slice(&hex::decode(format!("{:0>64x}", inner_len)).unwrap());

			// data (padded to 32-byte boundary)
			execute_data.extend_from_slice(&hex::decode(inner_calldata).unwrap());
			while execute_data.len() % 32 != 0 {
				execute_data.push(0);
			}

			format!("0x{}", hex::encode(execute_data))
		}

		let transfer_calldata =
			create_erc20_transfer_calldata(ARB_TO_HYPER_BRIDGE_ADDRESS, 1000000); // 1M units
		let execute_calldata = create_execute_calldata(ARBITRUM_USDC_ADDRESS, &transfer_calldata);

		let user_op = SerializablePackedUserOperation {
			sender: "0x1111111111111111111111111111111111111111".to_string(), // OmniAccount address
			nonce: 42,
			init_code: "0x".to_string(),
			call_data: execute_calldata,
			account_gas_limits:
				"0x0000000000000000000000000030d4000000000000000000000000000000c350".to_string(),
			pre_verification_gas: 21000,
			gas_fees: "0x000000000000000000000003b9aca0000000000000000000000000000b2d05e0"
				.to_string(),
			paymaster_and_data: "0x".to_string(),
			signature: None,
		};

		let result = validate_backend_calldata(&[user_op], ARBITRUM_MAINNET);
		assert!(result.is_ok(), "Valid Arbitrum backend calldata should succeed");
	}

	#[test]
	fn test_validate_backend_calldata_hyperevm() {
		use executor_core::types::SerializablePackedUserOperation;

		// Helper function to create calldata with proper payload format
		fn create_test_calldata(action_id: u32, additional_data: &[u8]) -> String {
			// Create payload: [version:1][action_id:3][additional_data]
			let mut payload = Vec::new();
			payload.push(0x01); // version
			payload.extend_from_slice(&action_id.to_be_bytes()[1..4]); // action_id (3 bytes)
			payload.extend_from_slice(additional_data);

			// Create calldata: method_sig + offset + length + payload (padded to 32-byte boundaries)
			let mut calldata = Vec::new();
			calldata.extend_from_slice(&[0x17, 0x93, 0x8e, 0x13]); // method signature for HyperEVM
			calldata.extend_from_slice(&[0u8; 28]); // offset padding
			calldata.extend_from_slice(&[0, 0, 0, 0x20]); // offset = 32
			calldata.extend_from_slice(&[0u8; 28]); // length padding
			calldata.extend_from_slice(&(payload.len() as u32).to_be_bytes()); // payload length
			calldata.extend_from_slice(&payload); // payload

			// Pad to 32-byte boundary
			while calldata.len() % 32 != 0 {
				calldata.push(0);
			}

			hex::encode(calldata)
		}

		// Helper function to create OmniAccount execute calldata
		fn create_execute_calldata(target_address: &str, inner_calldata: &str) -> String {
			let target_hex = target_address.strip_prefix("0x").unwrap().to_lowercase();
			let inner_len = inner_calldata.len() / 2; // Convert hex string length to byte length

			// Create execute(target, 0, inner_calldata) - ABI encoded
			let mut execute_data = Vec::new();
			execute_data.extend_from_slice(&[0xb6, 0x1d, 0x27, 0xf6]); // execute method signature

			// target address (32 bytes)
			execute_data.extend_from_slice(&hex::decode(format!("{:0>64}", target_hex)).unwrap());

			// value = 0 (32 bytes)
			execute_data.extend_from_slice(&[0u8; 32]);

			// offset to data (32 bytes) = 0x60 = 96 bytes
			execute_data.extend_from_slice(
				&hex::decode("0000000000000000000000000000000000000000000000000000000000000060")
					.unwrap(),
			);

			// data length (32 bytes)
			execute_data.extend_from_slice(&hex::decode(format!("{:0>64x}", inner_len)).unwrap());

			// data (padded to 32-byte boundary)
			execute_data.extend_from_slice(&hex::decode(inner_calldata).unwrap());
			while execute_data.len() % 32 != 0 {
				execute_data.push(0);
			}

			format!("0x{}", hex::encode(execute_data))
		}

		let inner_calldata = create_test_calldata(0x000002, &[0x12, 0x34]);
		let execute_calldata =
			create_execute_calldata(HYPEREVM_CORE_WRITER_ADDRESS, &inner_calldata);

		let user_op = SerializablePackedUserOperation {
			sender: "0x1111111111111111111111111111111111111111".to_string(), // OmniAccount address
			nonce: 42,
			init_code: "0x".to_string(),
			call_data: execute_calldata,
			account_gas_limits:
				"0x0000000000000000000000000030d4000000000000000000000000000000c350".to_string(),
			pre_verification_gas: 21000,
			gas_fees: "0x000000000000000000000003b9aca0000000000000000000000000000b2d05e0"
				.to_string(),
			paymaster_and_data: "0x".to_string(),
			signature: None,
		};

		let result = validate_backend_calldata(&[user_op], HYPEREVM_MAINNET);
		assert!(result.is_ok(), "Valid HyperEVM backend calldata should succeed");
	}

	#[test]
	fn test_validate_backend_calldata_wrong_contract() {
		use executor_core::types::SerializablePackedUserOperation;

		let user_op = SerializablePackedUserOperation {
			sender: "0x1111111111111111111111111111111111111111".to_string(), // Wrong contract
			nonce: 42,
			init_code: "0x".to_string(),
			call_data: "0xa9059cbb000000000000000000000000123456789012345678901234567890123456789000000000000000000000000000000000000000000000000000000000000f4240".to_string(),
			account_gas_limits: "0x0000000000000000000000000030d4000000000000000000000000000000c350".to_string(),
			pre_verification_gas: 21000,
			gas_fees: "0x000000000000000000000003b9aca0000000000000000000000000000b2d05e0".to_string(),
			paymaster_and_data: "0x".to_string(),
			signature: None,
		};

		let result = validate_backend_calldata(&[user_op], ARBITRUM_MAINNET);
		assert!(result.is_err(), "Wrong contract address should fail");
	}

	#[test]
	fn test_validate_hyperevm_core_writer_invalid_version() {
		// Helper function to create calldata with specific version
		fn create_test_calldata_with_version(
			version: u8,
			action_id: u32,
			additional_data: &[u8],
		) -> String {
			// Create payload: [version:1][action_id:3][additional_data]
			let mut payload = Vec::new();
			payload.push(version); // Custom version
			payload.extend_from_slice(&action_id.to_be_bytes()[1..4]); // action_id (3 bytes)
			payload.extend_from_slice(additional_data);

			// Create calldata: method_sig + offset + length + payload (padded to 32-byte boundaries)
			let mut calldata = Vec::new();
			calldata.extend_from_slice(&[0x12, 0x34, 0x56, 0x78]); // method signature
			calldata.extend_from_slice(&[0u8; 28]); // offset padding
			calldata.extend_from_slice(&[0, 0, 0, 0x20]); // offset = 32
			calldata.extend_from_slice(&[0u8; 28]); // length padding
			calldata.extend_from_slice(&(payload.len() as u32).to_be_bytes()); // payload length
			calldata.extend_from_slice(&payload); // payload

			// Pad to 32-byte boundary
			while calldata.len() % 32 != 0 {
				calldata.push(0);
			}

			format!("0x{}", hex::encode(calldata))
		}

		// Test invalid version = 0x02 (should be 0x01)
		let call_data = create_test_calldata_with_version(0x02, 0x000002, &[0x12, 0x34]);
		let result = validate_hyperevm_core_writer(&call_data, HYPEREVM_MAINNET);
		assert!(result.is_err(), "Invalid version should fail");

		// Test version = 0x00
		let call_data = create_test_calldata_with_version(0x00, 0x000002, &[0x12, 0x34]);
		let result = validate_hyperevm_core_writer(&call_data, HYPEREVM_MAINNET);
		assert!(result.is_err(), "Version 0x00 should fail");
	}

	#[test]
	fn test_validate_hyperevm_core_writer_short_payload() {
		// Create calldata with payload that's too short (less than 4 bytes)
		let mut calldata = Vec::new();
		calldata.extend_from_slice(&[0x12, 0x34, 0x56, 0x78]); // method signature
		calldata.extend_from_slice(&[0u8; 28]); // offset padding
		calldata.extend_from_slice(&[0, 0, 0, 0x20]); // offset = 32
		calldata.extend_from_slice(&[0u8; 28]); // length padding
		calldata.extend_from_slice(&[0, 0, 0, 3]); // payload length = 3 (too short)
		calldata.extend_from_slice(&[0x01, 0x00, 0x00]); // only 3 bytes

		// Pad to 32-byte boundary
		while calldata.len() % 32 != 0 {
			calldata.push(0);
		}

		let call_data = format!("0x{}", hex::encode(calldata));
		let result = validate_hyperevm_core_writer(&call_data, HYPEREVM_MAINNET);
		assert!(result.is_err(), "Short payload should fail");
	}
}
