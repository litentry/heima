use crate::utils::user_op::convert_to_packed_user_op;
use crate::RpcResult;
use crate::{error_code::AUTH_VERIFICATION_FAILED_CODE, ErrorCode};
use alloy::primitives::Address;
use executor_core::types::SerializablePackedUserOperation;
use executor_crypto::ecdsa;
use executor_primitives::{
	signature::{EthereumSignature, HeimaMultiSignature},
	utils::hex::decode_hex,
	ChainId,
};
use executor_storage::{Storage, WildmetaTimestampStorage};
use heima_primitives::{Address20, Identity};
use jsonrpsee::types::ErrorObject;
use oe_client_aa::calculate_user_operation_hash;
use std::sync::Arc;
use tracing::error;

/// Verify WildMeta signature
pub fn verify_wildmeta_signature(
	agent_address: &str,
	business_json: &str,
	signature: &str,
) -> RpcResult<()> {
	let message = business_json.as_bytes();

	let signature_bytes = decode_hex(signature).map_err(|e| {
		error!("Failed to decode signature: {:?}", e);
		ErrorObject::from(ErrorCode::ParseError)
	})?;
	let ethereum_signature =
		EthereumSignature::try_from(signature_bytes.as_slice()).map_err(|e| {
			error!("Failed to convert signature to EthereumSignature: {:?}", e);
			ErrorObject::from(ErrorCode::ParseError)
		})?;
	let heima_sig = HeimaMultiSignature::Ethereum(ethereum_signature);

	let agent_address_bytes = decode_hex(agent_address).map_err(|e| {
		error!("Failed to decode signature: {:?}", e);
		ErrorObject::from(ErrorCode::ParseError)
	})?;
	let agent_address = Address20::try_from(agent_address_bytes.as_slice()).map_err(|_| {
		error!("Failed to parse agent address");
		ErrorObject::from(ErrorCode::ParseError)
	})?;

	let agent_identity = Identity::Evm(agent_address);

	if !heima_sig.verify(message, &agent_identity) {
		error!("Signature verification failed");
		return Err(ErrorObject::from(ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE)));
	}

	Ok(())
}

/// Verify WildmetaBackend signature against user operation hash
pub fn verify_wildmeta_backend_signature(
	signature: &str,
	user_operations: &[SerializablePackedUserOperation],
	chain_id: ChainId,
	entry_point_address: Address,
	expected_pubkey: &[u8; 33],
) -> Result<(), ErrorObject<'static>> {
	if user_operations.is_empty() {
		error!("No user operations provided for signature verification");
		return Err(ErrorObject::from(ErrorCode::ParseError));
	}

	// Decode signature from hex
	let signature_bytes = decode_hex(signature).map_err(|e| {
		error!("Failed to decode signature: {:?}", e);
		ErrorObject::from(ErrorCode::ParseError)
	})?;

	if signature_bytes.len() != 65 {
		error!("Invalid signature length: expected 65 bytes, got {}", signature_bytes.len());
		return Err(ErrorObject::from(ErrorCode::ParseError));
	}

	let signature_array: [u8; 65] = signature_bytes
		.try_into()
		.map_err(|_| ErrorObject::from(ErrorCode::ParseError))?;

	// Convert all user operations to PackedUserOperations and calculate their combined hash
	let mut combined_hash_data = Vec::new();

	for user_op in user_operations {
		let packed_user_op = convert_to_packed_user_op(user_op.clone()).map_err(|e| {
			error!("Failed to convert user operation: {}", e);
			ErrorObject::from(ErrorCode::ParseError)
		})?;

		let user_op_hash =
			calculate_user_operation_hash(&packed_user_op, entry_point_address, chain_id);
		combined_hash_data.extend_from_slice(&user_op_hash.0);
	}

	// Hash the combined data using keccak256 to create a single 32-byte hash
	use alloy::primitives::keccak256;
	let user_op_hash = keccak256(&combined_hash_data);

	// Convert user op hash to 32-byte array
	let user_op_hash_array: [u8; 32] = user_op_hash.0;

	// Convert expected public key to ecdsa::Public
	let public_key = ecdsa::Public::from_raw(*expected_pubkey);

	let signature = ecdsa::Signature::from_raw(signature_array);

	// Verify signature directly using verify_prehashed
	if !ecdsa::Pair::verify_prehashed(&signature, &user_op_hash_array, &public_key) {
		error!("Signature verification failed");
		return Err(ErrorObject::from(ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE)));
	}

	Ok(())
}

/// Verify and update payload timestamp to prevent replay attacks
pub fn verify_payload_timestamp(
	storage: &Arc<WildmetaTimestampStorage>,
	main_address: &str,
	new_timestamp: u64,
) -> Result<(), ErrorObject<'static>> {
	let last_timestamp = storage
		.get(&main_address.to_string())
		.map_err(|_| {
			error!("Failed to get last timestamp");
			ErrorObject::from(ErrorCode::InternalError)
		})?
		.unwrap_or(0);

	if new_timestamp <= last_timestamp {
		error!("Invalid payload timestamp: {} <= {}", new_timestamp, last_timestamp);
		return Err(ErrorObject::from(ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE)));
	}

	storage.insert(&main_address.to_string(), new_timestamp).map_err(|_| {
		error!("Failed to store timestamp");
		ErrorObject::from(ErrorCode::InternalError)
	})?;

	Ok(())
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

		let result = verify_wildmeta_signature(agent_address, business_json, signature);
		assert!(result.is_ok(), "Signature verification should succeed");
	}

	#[test]
	fn test_verify_wildmeta_signature_invalid_signature() {
		let business_json = r#"{"action":"trade","timestamp":1752573555}"#;
		let signature = "0x020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
		let agent_address = "0xf8b16F021438B710fDE9d59dD17dDE1Eb2691BFd";

		let result = verify_wildmeta_signature(agent_address, business_json, signature);
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

		let result = verify_wildmeta_signature(agent_address, business_json, signature);
		assert!(result.is_err(), "Should fail with invalid hex signature");
	}

	#[test]
	fn test_verify_payload_timestamp_success() {
		let tmp_dir = tempdir().unwrap();
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());
		let storage = Arc::new(WildmetaTimestampStorage::new(db));

		let main_address = "0xA9d439F4DED81152DB00CB7CD94A8d908FEF903e";

		// First timestamp should succeed
		let result = verify_payload_timestamp(&storage, main_address, 1000);
		assert!(result.is_ok(), "First timestamp should succeed");

		// Higher timestamp should succeed
		let result = verify_payload_timestamp(&storage, main_address, 2000);
		assert!(result.is_ok(), "Higher timestamp should succeed");
	}

	#[test]
	fn test_verify_payload_timestamp_fails_with_old_timestamp() {
		let tmp_dir = tempdir().unwrap();
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());
		let storage = Arc::new(WildmetaTimestampStorage::new(db));

		let main_address = "0xA9d439F4DED81152DB00CB7CD94A8d908FEF903e";

		// Store initial timestamp
		let result = verify_payload_timestamp(&storage, main_address, 1000);
		assert!(result.is_ok());

		// Same timestamp should fail
		let result = verify_payload_timestamp(&storage, main_address, 1000);
		assert!(result.is_err(), "Same timestamp should fail");

		// Lower timestamp should fail
		let result = verify_payload_timestamp(&storage, main_address, 500);
		assert!(result.is_err(), "Lower timestamp should fail");
	}

	#[test]
	fn test_verify_payload_timestamp_first_time() {
		let tmp_dir = tempdir().unwrap();
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());
		let storage = Arc::new(WildmetaTimestampStorage::new(db));

		let main_address = "0xA9d439F4DED81152DB00CB7CD94A8d908FEF903e";

		// Any timestamp should succeed for first time
		let result = verify_payload_timestamp(&storage, main_address, 1);
		assert!(result.is_ok(), "First timestamp should succeed even if it's 1");
	}

	#[test]
	fn test_verify_payload_timestamp_persistence() {
		let tmp_dir = tempdir().unwrap();
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());
		let storage = Arc::new(WildmetaTimestampStorage::new(db));

		let main_address = "0xA9d439F4DED81152DB00CB7CD94A8d908FEF903e";

		// Store timestamp
		verify_payload_timestamp(&storage, main_address, 1000).unwrap();

		// Verify it's persisted by checking that lower timestamp fails
		let result = verify_payload_timestamp(&storage, main_address, 999);
		assert!(result.is_err(), "Timestamp should be persisted");

		// Verify exact stored value fails
		let result = verify_payload_timestamp(&storage, main_address, 1000);
		assert!(result.is_err(), "Exact stored timestamp should fail");

		// Higher should succeed
		let result = verify_payload_timestamp(&storage, main_address, 1001);
		assert!(result.is_ok(), "Higher timestamp should succeed");
	}

	#[test]
	fn test_verify_wildmeta_backend_signature_invalid_signature() {
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

		let result = verify_wildmeta_backend_signature(
			invalid_signature,
			&[user_op],
			chain_id,
			entry_point_address,
			&expected_pubkey,
		);

		assert!(result.is_err(), "Should fail with invalid signature");
	}

	#[test]
	fn test_wildmeta_backend_valid_signature_verification() {
		use alloy::primitives::{keccak256, Address};
		use executor_core::types::SerializablePackedUserOperation;
		use executor_crypto::secp256k1::{
			secp256k1_ecdsa_recover_compressed, secp256k1_ecdsa_sign,
		};
		use oe_client_aa::calculate_user_operation_hash;

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
		let result = verify_wildmeta_backend_signature(
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
		use alloy::primitives::{keccak256, Address};
		use executor_core::types::SerializablePackedUserOperation;
		use executor_crypto::secp256k1::{
			secp256k1_ecdsa_recover_compressed, secp256k1_ecdsa_sign,
		};
		use oe_client_aa::calculate_user_operation_hash;

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

		// Calculate combined hash for all operations
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
		let result = verify_wildmeta_backend_signature(
			&signature_hex,
			&user_operations,
			chain_id,
			entry_point_address,
			&expected_pubkey,
		);

		assert!(result.is_ok(), "Multiple operations signature verification should succeed");

		// Test that single operation would fail with same signature (different hash)
		let single_op_result = verify_wildmeta_backend_signature(
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
		use alloy::primitives::{keccak256, Address};
		use executor_core::types::SerializablePackedUserOperation;
		use executor_crypto::secp256k1::secp256k1_ecdsa_sign;
		use oe_client_aa::calculate_user_operation_hash;

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

		// Calculate combined hash
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
		let result = verify_wildmeta_backend_signature(
			&signature_hex,
			&[user_op],
			chain_id,
			entry_point_address,
			&wrong_expected_pubkey,
		);

		assert!(result.is_err(), "Signature verification with wrong public key should fail");
	}
}
