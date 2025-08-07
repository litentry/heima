use crate::{error_code::AUTH_VERIFICATION_FAILED_CODE, ErrorCode};
use executor_primitives::{
	signature::{EthereumSignature, HeimaMultiSignature},
	utils::hex::{decode_hex, FromHexPrefixed},
};
use executor_storage::{Storage, WildmetaTimestampStorage};
use heima_primitives::{Address20, Identity};
use jsonrpsee::types::ErrorObject;
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
