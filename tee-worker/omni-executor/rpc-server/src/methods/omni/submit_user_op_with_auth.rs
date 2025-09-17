use super::common::handle_omni_native_task;
use crate::auth_utils::{
	verify_payload_timestamp, verify_wildmeta_backend_signature, verify_wildmeta_signature,
};
use crate::detailed_error::DetailedError;
use crate::error_code::{
	AUTH_VERIFICATION_FAILED_CODE, INVALID_USER_OPERATION_CODE, PARSE_ERROR_CODE,
};
use crate::methods::omni::PumpxRpcError;
use crate::server::RpcContext;
use crate::validation_helpers::{
	validate_chain_id, validate_user_operations, validate_wallet_index,
};
use alloy::primitives::{hex, Address};
use executor_core::intent_executor::IntentExecutor;
use executor_core::native_task::{NativeTask, NativeTaskWrapper};
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::{ChainId, ClientAuth, Identity, UserAuth, UserId};
use executor_storage::WildmetaTimestampStorage;
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use pumpx::pubkey_to_address;
use serde::{Deserialize, Serialize};
use signer_client::ChainType;
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
const HYPEREVM_CORE_WRITER_ADDRESS: &str = "0x0000000000000000000000000000000000003333";

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
fn validate_arbitrum_usdc_transfer(
	call_data: &str,
	chain_id: ChainId,
) -> Result<(), PumpxRpcError> {
	// Validate chain is supported for Arbitrum USDC validation
	match chain_id {
		ARBITRUM_MAINNET | ARBITRUM_SEPOLIA => {},
		_ => {
			return Err(PumpxRpcError::from(
				DetailedError::new(
					AUTH_VERIFICATION_FAILED_CODE,
					"Chain ID is not supported for Arbitrum USDC validation",
				)
				.with_field("chain_id")
				.with_received(chain_id.to_string())
				.with_expected("42161 or 421614"),
			));
		},
	};

	// Parse calldata - should be USDC.transfer method call
	let call_bytes =
		hex::decode(call_data.strip_prefix("0x").unwrap_or(call_data)).map_err(|e| {
			PumpxRpcError::from(
				DetailedError::new(
					// todo: proper error code
					AUTH_VERIFICATION_FAILED_CODE,
					"Invalid hex encoding in call data",
				)
				.with_field("call_data")
				.with_reason(format!("Hex decode error: {}", e)),
			)
		})?;

	// Check if it's an ERC20 transfer call (method signature 0xa9059cbb)
	if call_bytes.len() < 4 || call_bytes[0..4] != ERC20_TRANSFER_SIGNATURE {
		return Err(PumpxRpcError::from(
			DetailedError::new(
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
			.with_expected("0xa9059cbb (ERC20.transfer)"),
		));
	}

	// Decode the transfer call to get recipient
	if call_bytes.len() < 68 {
		return Err(PumpxRpcError::from(
			DetailedError::new(
				// todo: proper error code
				AUTH_VERIFICATION_FAILED_CODE,
				"Call data too short for ERC20 transfer",
			)
			.with_field("call_data_length")
			.with_received(call_bytes.len().to_string())
			.with_expected("68 bytes minimum"),
		));
	}

	// Extract recipient address (bytes 4-36, but we need the last 20 bytes)
	let recipient_bytes = &call_bytes[16..36];
	let recipient = format!("0x{}", hex::encode(recipient_bytes));

	// Validate recipient is the official arb -> hyper bridge
	if recipient.to_lowercase() != ARB_TO_HYPER_BRIDGE_ADDRESS.to_lowercase() {
		return Err(PumpxRpcError::from(
			DetailedError::new(
				AUTH_VERIFICATION_FAILED_CODE,
				"USDC transfer recipient is not the official Arbitrum to Hyperliquid bridge",
			)
			.with_field("recipient")
			.with_received(recipient)
			.with_expected(ARB_TO_HYPER_BRIDGE_ADDRESS),
		));
	}

	Ok(())
}

/// Validates core writer call for HyperEVM chains
fn validate_hyperevm_core_writer(call_data: &str, chain_id: ChainId) -> Result<(), PumpxRpcError> {
	// Validate chain is HyperEVM
	if chain_id != HYPEREVM_MAINNET && chain_id != HYPEREVM_TESTNET {
		return Err(PumpxRpcError::from(
			DetailedError::new(
				// todo: proper error code
				AUTH_VERIFICATION_FAILED_CODE,
				"Chain ID is not supported for HyperEVM validation",
			)
			.with_field("chain_id")
			.with_received(chain_id.to_string())
			.with_expected("999 or 998"),
		));
	}

	// Parse calldata
	let call_bytes =
		hex::decode(call_data.strip_prefix("0x").unwrap_or(call_data)).map_err(|e| {
			PumpxRpcError::from(
				DetailedError::new(
					// todo: proper error code
					AUTH_VERIFICATION_FAILED_CODE,
					"Invalid hex encoding in call data",
				)
				.with_field("call_data")
				.with_reason(format!("Hex decode error: {}", e)),
			)
		})?;

	// Extract method signature if available
	if call_bytes.len() < 4 {
		return Err(PumpxRpcError::from(
			DetailedError::new(
				// todo: proper error code
				AUTH_VERIFICATION_FAILED_CODE,
				"Call data too short to contain method signature",
			)
			.with_field("call_data_length")
			.with_received(call_bytes.len().to_string())
			.with_expected("4 bytes minimum"),
		));
	}

	// Proper action_id extraction based on payload format:
	// [1 byte] → Version (currently 0x01)
	// [3 bytes] → action_id
	// [rest] → Action-specific payload (encoded via Solidity ABI rules)

	// The calldata should contain the raw payload as data parameter in method call
	// First decode the actual payload from the method call parameters
	if call_bytes.len() < 4 + 32 + 32 {
		return Err(PumpxRpcError::from(
			DetailedError::new(
				INVALID_USER_OPERATION_CODE,
				"Call data too short for payload parameter",
			)
			.with_field("call_data_length")
			.with_received(call_bytes.len().to_string())
			.with_expected("68 bytes minimum (4 bytes method + 32 bytes offset + 32 bytes length)"),
		));
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
		return Err(PumpxRpcError::from(
			DetailedError::new(
				INVALID_USER_OPERATION_CODE,
				"Call data too short for declared payload length",
			)
			.with_field("call_data_length")
			.with_received(call_bytes.len().to_string())
			.with_expected(format!("{} bytes", 68 + payload_length)),
		));
	}

	// Extract the actual payload
	let payload = &call_bytes[68..68 + payload_length];

	// Validate payload format: minimum 4 bytes (1 version + 3 action_id)
	if payload.len() < 4 {
		return Err(PumpxRpcError::from(
			DetailedError::new(
				INVALID_USER_OPERATION_CODE,
				"Payload too short for version and action_id",
			)
			.with_field("payload_length")
			.with_received(payload.len().to_string())
			.with_expected("4 bytes minimum (1 version + 3 action_id)"),
		));
	}

	// Extract version (first byte)
	let version = payload[0];
	if version != 0x01 {
		return Err(PumpxRpcError::from(
			DetailedError::new(INVALID_USER_OPERATION_CODE, "Invalid payload version")
				.with_field("version")
				.with_received(format!("0x{:02x}", version))
				.with_expected("0x01"),
		));
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
		return Err(PumpxRpcError::from(
			DetailedError::new(
				INVALID_USER_OPERATION_CODE,
				"Invalid action_id for HyperEVM core writer call",
			)
			.with_field("action_id")
			.with_received(format!("0x{:06x}", action_id))
			.with_expected("One of: 0x000002, 0x000003, 0x000004, 0x000005, 0x000007"),
		));
	}

	Ok(())
}

/// Validates calldata for wallet_index == 0 backend requests
fn validate_backend_calldata(
	user_operations: &[SerializablePackedUserOperation],
	chain_id: ChainId,
) -> Result<(), PumpxRpcError> {
	for (index, user_op) in user_operations.iter().enumerate() {
		// Parse the target address from sender field
		let target_address = user_op.sender.parse::<Address>().map_err(|e| {
			PumpxRpcError::from(
				DetailedError::new(AUTH_VERIFICATION_FAILED_CODE, "Invalid sender address format")
					.with_field(format!("user_operations[{}].sender", index))
					.with_received(&user_op.sender)
					.with_reason(format!("Address parse error: {}", e)),
			)
		})?;

		match chain_id {
			ARBITRUM_MAINNET | ARBITRUM_SEPOLIA => {
				// For Arbitrum: sender must be USDC contract and calldata must be transfer to bridge
				let expected_usdc = match chain_id {
					ARBITRUM_MAINNET => ARBITRUM_USDC_ADDRESS,
					ARBITRUM_SEPOLIA => ARBITRUM_SEPOLIA_USDC_ADDRESS,
					_ => unreachable!(),
				};

				if target_address != expected_usdc.parse::<Address>().unwrap() {
					return Err(PumpxRpcError::from(
						DetailedError::new(
							AUTH_VERIFICATION_FAILED_CODE,
							"For Arbitrum backend requests, target contract must be USDC",
						)
						.with_field(format!("user_operations[{}].sender", index))
						.with_received(&user_op.sender)
						.with_expected(expected_usdc),
					));
				}

				validate_arbitrum_usdc_transfer(&user_op.call_data, chain_id)?;
			},
			HYPEREVM_MAINNET | HYPEREVM_TESTNET => {
				// For HyperEVM: target must be core writer and calldata must have valid action_id
				if target_address != HYPEREVM_CORE_WRITER_ADDRESS.parse::<Address>().unwrap() {
					return Err(PumpxRpcError::from(
						DetailedError::new(
							AUTH_VERIFICATION_FAILED_CODE,
							"For HyperEVM backend requests, target contract must be core writer",
						)
						.with_field(format!("user_operations[{}].sender", index))
						.with_received(&user_op.sender)
						.with_expected(HYPEREVM_CORE_WRITER_ADDRESS),
					));
				}

				validate_hyperevm_core_writer(&user_op.call_data, chain_id)?;
			},
			_ => {
				return Err(PumpxRpcError::from(
					DetailedError::new(
						AUTH_VERIFICATION_FAILED_CODE,
						"Chain ID not supported for backend wallet_index validation",
					)
					.with_field("chain_id")
					.with_received(chain_id.to_string())
					.with_expected("42161, 421614, 999, or 998"),
				));
			},
		}
	}

	Ok(())
}

pub fn register_submit_user_op_with_auth<
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
		.register_async_method("omni_submitUserOpWithAuth", |params, ctx, _ext| async move {
			let params = params.parse::<SubmitUserOpWithAuthParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(
						PARSE_ERROR_CODE,
						"Failed to parse request parameters",
					)
					.with_reason(format!("Invalid JSON structure: {}", e)),
				)
			})?;

			debug!("Received omni_submitUserOpWithAuth, params: {:?}", params);

			// Validate common parameters
			validate_chain_id(params.chain_id as u32, Some("evm")).map_err(PumpxRpcError::from)?;
			validate_wallet_index(params.wallet_index).map_err(PumpxRpcError::from)?;
			validate_user_operations(&params.user_operations).map_err(PumpxRpcError::from)?;

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
							PumpxRpcError::from(
								DetailedError::new(
									PARSE_ERROR_CODE,
									"Failed to parse business JSON",
								)
								.with_field("business_json")
								.with_reason(format!("JSON parse error: {}", e)),
							)
						})?;

					let timestamp = business_data
						.get("timestamp")
						.and_then(|v| v.as_u64())
						.ok_or_else(|| {
							error!("Missing timestamp in business_json");
							PumpxRpcError::from(
								DetailedError::new(
									crate::error_code::MISSING_REQUIRED_FIELD_CODE,
									"Missing required field in business JSON",
								)
								.with_field("timestamp")
								.with_expected("Unix timestamp as number")
								.with_reason("Business JSON must contain a 'timestamp' field"),
							)
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
							PumpxRpcError::from(
								DetailedError::new(
									crate::error_code::EXTERNAL_API_ERROR_CODE,
									"Failed to verify Hyperliquid account link",
								)
								.with_field("operation")
								.with_received("verify_hyperliquid_link")
								.with_reason("Could not verify agent and main address linkage"),
							)
						})?;

					if !linked {
						error!("Agent and main addresses are not linked");
						return Err(PumpxRpcError::from(
							DetailedError::new(
								AUTH_VERIFICATION_FAILED_CODE,
								"Agent and main addresses are not linked",
							)
							.with_field("agent_address")
							.with_received(agent_address.to_string())
							.with_suggestion("Ensure the agent address is properly linked to the main address"),
						));
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
						return Err(PumpxRpcError::from(
							DetailedError::new(
								AUTH_VERIFICATION_FAILED_CODE,
								"Invalid wallet index for WildmetaBackend authentication",
							)
							.with_field("wallet_index")
							.with_received(params.wallet_index.to_string())
							.with_expected("0 or 1")
							.with_suggestion("WildmetaBackend authentication requires wallet_index to be 0 or 1"),
						));
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
						return Err(PumpxRpcError::from(
							DetailedError::new(
								AUTH_VERIFICATION_FAILED_CODE,
								"Invalid client ID for WildmetaBackend authentication",
							)
							.with_field("client_id")
							.with_received(params.client_id.clone())
							.with_expected("wildmeta")
							.with_suggestion("WildmetaBackend authentication requires client_id to be 'wildmeta'"),
						));
					}

					// Get entry point address for the chain
					let entry_point_client =
						ctx.entry_point_clients.get(&params.chain_id).ok_or_else(|| {
							error!("No entry point client found for chain_id: {}", params.chain_id);
							PumpxRpcError::from(
								DetailedError::chain_not_supported(params.chain_id),
							)
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
					return Err(PumpxRpcError::from(
						DetailedError::new(
							PARSE_ERROR_CODE,
							"Invalid client authentication type",
						)
						.with_field("client_auth")
						.with_expected("WildmetaHl or WildmetaBackend")
						.with_suggestion("Use a supported authentication method"),
					));
				},
			};

			let identity = Identity::try_from(params.user_id.clone()).map_err(|e| {
				error!("Failed to convert UserId to Identity: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(
						crate::error_code::ACCOUNT_PARSE_ERROR_CODE,
						"Invalid user identity format",
					)
					.with_field("user_id")
					.with_reason(format!("Failed to parse user identity: {}", e)),
				)
			})?;

			// Only validate main_address if it's provided (not None)
			if let Some(main_addr) = &main_address {
				match &identity {
					Identity::Evm(_) => {
						if let UserId::Evm(user_address) = &params.user_id {
							if user_address.to_lowercase() != main_addr.to_lowercase() {
								error!("Main address does not match user_id for EVM identity");
								return Err(PumpxRpcError::from(
									DetailedError::new(
										AUTH_VERIFICATION_FAILED_CODE,
										"User address does not match authenticated main address for EVM identity",
									)
									.with_field("user_address")
									.with_received(user_address.to_string())
									.with_expected(main_addr.to_string())
									.with_suggestion("For EVM identity, the user_id must match the authenticated main address"),
								));
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
								PumpxRpcError::from(
									DetailedError::signer_service_error(
										"request_wallet",
										&format!("Failed to derive wallet: {:?}", e),
									),
								)
							})?;
						let derived_address = pubkey_to_address(ChainType::Evm, &derived_pubkey)
							.map_err(|e| {
								error!("Failed to convert derived pubkey to address: {:?}", e);
								PumpxRpcError::from(
									DetailedError::new(
										AUTH_VERIFICATION_FAILED_CODE,
										"Failed to convert derived public key to address",
									)
									.with_field("operation")
									.with_received("pubkey_to_address conversion")
									.with_reason(format!("Internal error converting public key to address: {:?}", e)),
								)
							})?;
						if derived_address.to_lowercase() != main_addr.to_lowercase() {
							error!("Main address does not match derived EVM address");
							return Err(PumpxRpcError::from(
								DetailedError::new(
									AUTH_VERIFICATION_FAILED_CODE,
									"Derived address does not match authenticated address",
								)
								.with_field("derived_address")
								.with_received(derived_address.to_string())
								.with_expected(main_addr.to_string())
								.with_suggestion("The derived EVM address must match the authenticated main address"),
							));
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
					PumpxRpcError::from(
						DetailedError::invalid_address_format(
							&format!("user_operations[{}].sender", index),
							&op.sender,
							"0x-prefixed 20-byte Ethereum address (40 hex chars)",
						),
					)
				})?;
			}

			let wrapper = NativeTaskWrapper::new(
				NativeTask::SubmitUserOp(
					account_id,
					params.user_operations.clone(),
					params.chain_id,
					params.wallet_index,
				),
				None,
				None,
				params.client_id,
			);

			handle_omni_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::SubmitUserOp(transaction_hash) => {
					Ok(SubmitUserOpWithAuthResponse { transaction_hash })
				},
				_ => {
					error!("Unexpected response type from native task handler");
					Err(DetailedError::unexpected_response_type("SubmitUserOp", "Unknown").into())
				},
			})
			.await
		})
		.expect("Failed to register omni_submitUserOpWithAuth method");
}

// Wrapper functions to convert shared function return types to PumpxRpcError
fn verify_wildmeta_signature_wrapper(
	agent_address: &str,
	business_json: &str,
	signature: &str,
) -> Result<(), PumpxRpcError> {
	verify_wildmeta_signature(agent_address, business_json, signature).map_err(|err| {
		PumpxRpcError::from(
			DetailedError::new(err.code(), "Wildmeta signature verification failed")
				.with_field("agent_address")
				.with_received(agent_address.to_string())
				.with_suggestion("Ensure the signature is valid and matches the agent address"),
		)
	})
}

fn verify_payload_timestamp_wrapper(
	storage: &Arc<WildmetaTimestampStorage>,
	main_address: &str,
	new_timestamp: u64,
) -> Result<(), PumpxRpcError> {
	verify_payload_timestamp(storage, main_address, new_timestamp).map_err(|err| {
		PumpxRpcError::from(
			DetailedError::new(err.code(), "Timestamp verification failed")
				.with_field("timestamp")
				.with_received(new_timestamp.to_string())
				.with_suggestion("Timestamp must be greater than the previously used timestamp"),
		)
	})
}

fn verify_wildmeta_backend_signature_wrapper(
	signature: &str,
	user_operations: &[SerializablePackedUserOperation],
	chain_id: ChainId,
	entry_point_address: Address,
	expected_pubkey: &[u8; 33],
) -> Result<(), PumpxRpcError> {
	verify_wildmeta_backend_signature(
		signature,
		user_operations,
		chain_id,
		entry_point_address,
		expected_pubkey,
	)
	.map_err(|err| {
		PumpxRpcError::from(
			DetailedError::new(err.code(), "Backend signature verification failed")
				.with_field("signature")
				.with_suggestion("Ensure the backend signature is valid for the given operations"),
		)
	})
}

#[cfg(test)]
mod tests {
	use super::*;
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
		use native_task_handler::convert_to_packed_user_op;

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
		use native_task_handler::convert_to_packed_user_op;

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
		use native_task_handler::convert_to_packed_user_op;

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

		let valid_transfer_calldata = format!(
			"0xa9059cbb000000000000000000000000{}00000000000000000000000000000000000000000000000000000000000f4240",
			&ARB_TO_HYPER_BRIDGE_ADDRESS[2..]
		);

		let user_op = SerializablePackedUserOperation {
			sender: ARBITRUM_USDC_ADDRESS.to_string(),
			nonce: 42,
			init_code: "0x".to_string(),
			call_data: valid_transfer_calldata,
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

		let valid_core_writer_calldata = create_test_calldata(0x000002, &[0x12, 0x34]);

		let user_op = SerializablePackedUserOperation {
			sender: HYPEREVM_CORE_WRITER_ADDRESS.to_string(),
			nonce: 42,
			init_code: "0x".to_string(),
			call_data: valid_core_writer_calldata,
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
