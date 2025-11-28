use crate::detailed_error::DetailedError;
use crate::error_code::{AUTH_VERIFICATION_FAILED_CODE, INVALID_USEROP_CODE};
use crate::server::RpcContext;
use crate::utils::auth::{
	verify_payload_timestamp, verify_wildmeta_backend_signature, verify_wildmeta_signature,
};
use crate::utils::types::RpcResultExt;
use crate::utils::user_op::submit_user_ops;
use crate::utils::validation::{
	parse_rpc_params, validate_chain_id, validate_user_operations, validate_wallet_index,
};
use crate::RpcResult;
use alloy::primitives::{hex, Address};
use jsonrpsee::RpcModule;
use oe_client_pumpx::pubkey_to_address;
use oe_client_signer::ChainType;
use oe_core::intent::executor::IntentExecutor;
use oe_core::types::SerializablePackedUserOperation;
use oe_primitives::{ChainId, ClientAuth, UserAuth, UserId};
use oe_storage::WildmetaTimestampStorage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error};

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
			INVALID_USEROP_CODE,
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
			INVALID_USEROP_CODE,
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
			INVALID_USEROP_CODE,
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
		return Err(DetailedError::new(INVALID_USEROP_CODE, "Invalid payload version")
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
			INVALID_USEROP_CODE,
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
	let call_bytes = hex::decode(call_data.strip_prefix("0x").unwrap_or(call_data))
		.map_err_parse("Failded to decode call_data")?;

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
		Err(DetailedError::new(
			AUTH_VERIFICATION_FAILED_CODE,
			"Call data is not an OmniAccount execute or executeBatch function call",
		)
		.with_field("method_signature")
		.with_received(format!("0x{}", hex::encode(method_sig)))
		.with_expected("0xb61d27f6 (execute) or 0x18dfeb3c (executeBatch)")
		.to_rpc_error())
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
			let params = parse_rpc_params::<SubmitUserOpWithAuthParams>(params)?;

			debug!("Received omni_submitUserOpWithAuth, params: {:?}", params);

			// Validate common parameters
			validate_chain_id(params.chain_id as u32)?;
			validate_wallet_index(params.wallet_index)?;
			validate_user_operations(&params.user_operations)?;

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
						.map_err_internal("Failed to parse business_json")?;

					let timestamp = business_data
						.get("timestamp")
						.and_then(|v| v.as_u64())
						.ok_or_else(|| {
							error!("Missing timestamp in business_json");
							DetailedError::invalid_params("business_json", "missing timestamp")
								.to_rpc_error()
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
						.map_err(|_| {
							let msg = "Failed to verify hyperliquid link";
							error!(msg);
							DetailedError::wildmeta_service_error("verify_hyperliquid_link")
								.to_rpc_error()
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
							DetailedError::invalid_chain_id(params.chain_id).to_rpc_error()
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
					let msg = "Invalid client auth type";
					error!(msg);
					return Err(DetailedError::parse_error(msg).to_rpc_error());
				},
			};

			// Only validate main_address if it's provided (not None)
			if let Some(main_addr) = &main_address {
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
				} else {
					let omni_account = params
						.user_id
						.to_omni_account(&params.client_id)
						.map_err_parse("Failed to convert to omni_account")?;

					let derived_address = ctx
						.signer_client
						.request_wallet(
							ChainType::Evm,
							params.wallet_index,
							*omni_account.as_ref(),
						)
						.await
						.map_err(|_| {
							DetailedError::signer_service_error().to_rpc_error()
						})
						.and_then(|pk| pubkey_to_address(ChainType::Evm, &pk).map_err_internal("Failed to convert pubkey to address"))?;

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
				}
			}

			let account_id = params
				.user_id
				.to_omni_account(&params.client_id)
				.map_err_parse("Failed to convert to omni_account")?;

			// Call the common submission logic
			let transaction_hash = submit_user_ops(
				&ctx,
				params.user_operations,
				params.chain_id,
				params.wallet_index,
				&account_id,
			).await?;

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

	#[test]
	fn test_validate_arbitrum_usdc_transfer_valid() {
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
		use oe_core::types::SerializablePackedUserOperation;

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
		use oe_core::types::SerializablePackedUserOperation;

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
		use oe_core::types::SerializablePackedUserOperation;

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
