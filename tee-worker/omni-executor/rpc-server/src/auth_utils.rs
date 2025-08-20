use crate::{error_code::AUTH_VERIFICATION_FAILED_CODE, ErrorCode};
use aa_contracts_client::calculate_user_operation_hash;
use alloy::primitives::Address;
use executor_core::types::SerializablePackedUserOperation;
use executor_crypto::ecdsa;
use executor_primitives::{
	signature::{EthereumSignature, HeimaMultiSignature},
	utils::hex::{decode_hex, FromHexPrefixed},
	ChainId,
};
use executor_storage::{Storage, WildmetaTimestampStorage};
use heima_primitives::{Address20, Identity};
use jsonrpsee::types::ErrorObject;
use native_task_handler::convert_to_packed_user_op;
use std::sync::Arc;
use tracing::error;

/// Verify WildMeta signature
pub fn verify_wildmeta_signature(
	agent_address: &str,
	business_json: &str,
	signature: &str,
) -> Result<(), ErrorObject<'static>> {
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

	let agent_address = Address20::from_hex(agent_address).map_err(|e| {
		error!("Failed to parse agent address: {:?}", e);
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
