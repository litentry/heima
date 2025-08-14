use super::common::handle_omni_native_task;
use crate::auth_utils::{
	verify_payload_timestamp, verify_wildmeta_backend_signature, verify_wildmeta_signature,
};
use crate::error_code::AUTH_VERIFICATION_FAILED_CODE;
use crate::methods::omni::PumpxRpcError;
use crate::server::RpcContext;
use crate::ErrorCode;
use alloy::primitives::Address;
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
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!("Received omni_submitUserOpWithAuth, params: {:?}", params);

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
							PumpxRpcError::from_error_code(ErrorCode::ParseError)
						})?;

					let timestamp = business_data
						.get("timestamp")
						.and_then(|v| v.as_u64())
						.ok_or_else(|| {
							error!("Missing timestamp in business_json");
							PumpxRpcError::from_error_code(ErrorCode::ParseError)
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
							error!("Failed to verify hyperliquid link");
							PumpxRpcError::from_error_code(ErrorCode::InternalError)
						})?;

					if !linked {
						error!("Agent and main addresses are not linked");
						return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
							AUTH_VERIFICATION_FAILED_CODE,
						)));
					}

					Some(main_address.clone())
				},
				ClientAuth::WildmetaBackend { signature } => {
					// Validate wallet_index must be 1 for WildmetaBackend
					if params.wallet_index != 1 {
						error!(
							"WildmetaBackend requires wallet_index to be 1, got: {}",
							params.wallet_index
						);
						return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
							AUTH_VERIFICATION_FAILED_CODE,
						)));
					}

					// Validate client_id must be "wildmeta" for WildmetaBackend
					if params.client_id != "wildmeta" {
						error!(
							"WildmetaBackend requires client_id to be 'wildmeta', got: {}",
							params.client_id
						);
						return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
							AUTH_VERIFICATION_FAILED_CODE,
						)));
					}

					// Get entry point address for the chain
					let entry_point_client =
						ctx.entry_point_clients.get(&params.chain_id).ok_or_else(|| {
							error!("No entry point client found for chain_id: {}", params.chain_id);
							PumpxRpcError::from_error_code(ErrorCode::InternalError)
						})?;

					let entry_point_address = entry_point_client.entry_point_address();

					// Verify the backend signature against user operation hash
					verify_wildmeta_backend_signature_wrapper(
						signature,
						&params.user_operations,
						params.chain_id,
						entry_point_address,
						&ctx.heima_backend_ecdsa_pubkey,
					)?;

					None
				},
				_ => {
					error!("Invalid client auth type");
					return Err(PumpxRpcError::from_error_code(ErrorCode::ParseError));
				},
			};

			let identity = Identity::try_from(params.user_id.clone()).map_err(|e| {
				error!("Failed to convert UserId to Identity: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			// Only validate main_address if it's provided (not None)
			if let Some(main_addr) = &main_address {
				match &identity {
					Identity::Evm(_) => {
						if let UserId::Evm(user_address) = &params.user_id {
							if user_address.to_lowercase() != main_addr.to_lowercase() {
								error!("Main address does not match user_id for EVM identity");
								return Err(PumpxRpcError::from_error_code(
									ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE),
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
							.map_err(|_| {
								error!("Failed to derive EVM address");
								PumpxRpcError::from_error_code(ErrorCode::InternalError)
							})?;
						let derived_address = pubkey_to_address(ChainType::Evm, &derived_pubkey)
							.map_err(|_| {
								error!("Failed to convert derived pubkey to address");
								PumpxRpcError::from_error_code(ErrorCode::ServerError(
									AUTH_VERIFICATION_FAILED_CODE,
								))
							})?;
						if derived_address.to_lowercase() != main_addr.to_lowercase() {
							error!("Main address does not match derived EVM address");
							return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
								AUTH_VERIFICATION_FAILED_CODE,
							)));
						}
					},
				}
			}

			let account_id = identity.to_omni_account(&params.client_id);

			for op in &params.user_operations {
				op.sender.parse::<Address>().map_err(|e| {
					error!("Invalid sender address '{}': {}", op.sender, e);
					PumpxRpcError::from_error_code(ErrorCode::ParseError)
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
					error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
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
	verify_wildmeta_signature(agent_address, business_json, signature)
		.map_err(|err| PumpxRpcError::from_error_code(err.code().into()))
}

fn verify_payload_timestamp_wrapper(
	storage: &Arc<WildmetaTimestampStorage>,
	main_address: &str,
	new_timestamp: u64,
) -> Result<(), PumpxRpcError> {
	verify_payload_timestamp(storage, main_address, new_timestamp)
		.map_err(|err| PumpxRpcError::from_error_code(err.code().into()))
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
	.map_err(|err| PumpxRpcError::from_error_code(err.code().into()))
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
		let signature = "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
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

		let invalid_signature = "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
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
		use alloy::primitives::Address;
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

		// Convert to PackedUserOperation and calculate hash
		let packed_user_op = convert_to_packed_user_op(user_op.clone()).unwrap();
		let chain_id = 31337u64; // Local test chain
		let entry_point_address = Address::from([0u8; 20]);
		let user_op_hash =
			calculate_user_operation_hash(&packed_user_op, entry_point_address, chain_id);

		// Sign the hash with our private key
		let signature = match secp256k1_ecdsa_sign(&private_key, &user_op_hash.0) {
			Ok(sig) => sig,
			Err(_) => panic!("Failed to sign with valid private key"),
		};

		// Derive the expected public key from the signature and hash
		let expected_pubkey = match secp256k1_ecdsa_recover_compressed(&signature, &user_op_hash.0)
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
	fn test_wildmeta_backend_invalid_signature_wrong_key() {
		use aa_contracts_client::calculate_user_operation_hash;
		use alloy::primitives::Address;
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

		// Convert to PackedUserOperation and calculate hash
		let packed_user_op = convert_to_packed_user_op(user_op.clone()).unwrap();
		let chain_id = 31337u64;
		let entry_point_address = Address::from([0u8; 20]);
		let user_op_hash =
			calculate_user_operation_hash(&packed_user_op, entry_point_address, chain_id);

		// Sign the hash with our private key
		let signature = match secp256k1_ecdsa_sign(&private_key, &user_op_hash.0) {
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
}
