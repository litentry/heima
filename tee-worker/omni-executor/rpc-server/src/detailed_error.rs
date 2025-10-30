use crate::error_code::{
	ACCOUNT_PARSE_ERROR_CODE, EMAIL_SERVICE_ERROR_CODE, GAS_ESTIMATION_FAILED_CODE,
	INVALID_ADDRESS_FORMAT_CODE, INVALID_AMOUNT_CODE, INVALID_CHAIN_ID_CODE,
	INVALID_HEX_FORMAT_CODE, INVALID_USER_OPERATION_CODE, INVALID_WALLET_INDEX_CODE,
	PASSKEY_ALREADY_EXISTS_CODE, PASSKEY_ATTESTATION_PARSE_ERROR_CODE,
	PASSKEY_CHALLENGE_EXPIRED_CODE, PASSKEY_CHALLENGE_NOT_FOUND_CODE,
	PASSKEY_CLIENT_DATA_PARSE_ERROR_CODE, PASSKEY_COUNTER_VALIDATION_FAILED_CODE,
	PASSKEY_INVALID_CHALLENGE_CODE, PASSKEY_NOT_FOUND_CODE,
	PASSKEY_ORIGIN_VERIFICATION_FAILED_CODE, PASSKEY_PARSE_ERROR_CODE, PASSKEY_REPLAY_ATTACK_CODE,
	PASSKEY_RP_ID_MISMATCH_CODE, PASSKEY_SIGNATURE_INVALID_CODE,
	PASSKEY_USER_VERIFICATION_FAILED_CODE, SIGNATURE_SERVICE_UNAVAILABLE_CODE,
	SIGNER_SERVICE_ERROR_CODE, STORAGE_SERVICE_ERROR_CODE, UNEXPECTED_RESPONSE_TYPE_CODE,
};
use jsonrpsee::types::ErrorObject;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedError {
	pub code: i32,
	pub message: String,
	pub details: ErrorDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub field: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub expected: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub received: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub reason: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub suggestion: Option<String>,
}

impl DetailedError {
	pub fn new(code: i32, message: impl Into<String>) -> Self {
		Self {
			code,
			message: message.into(),
			details: ErrorDetails {
				field: None,
				expected: None,
				received: None,
				reason: None,
				suggestion: None,
			},
		}
	}

	pub fn with_field(mut self, field: impl Into<String>) -> Self {
		self.details.field = Some(field.into());
		self
	}

	pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
		self.details.expected = Some(expected.into());
		self
	}

	pub fn with_received(mut self, received: impl Into<String>) -> Self {
		self.details.received = Some(received.into());
		self
	}

	pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
		self.details.reason = Some(reason.into());
		self
	}

	pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
		self.details.suggestion = Some(suggestion.into());
		self
	}

	pub fn to_error_object(&self) -> ErrorObject<'static> {
		ErrorObject::owned(self.code, self.message.clone(), Some(self.details.clone()))
	}
}

impl DetailedError {
	pub fn invalid_chain_id(chain_id: u64, supported_chains: &[u64]) -> Self {
		Self::new(INVALID_CHAIN_ID_CODE, "Invalid or unsupported chain ID")
			.with_field("chain_id")
			.with_received(chain_id.to_string())
			.with_expected(format!("One of: {:?}", supported_chains))
			.with_suggestion("Use a supported chain ID from the list")
	}

	pub fn invalid_wallet_index(index: u32, max_index: u32) -> Self {
		Self::new(INVALID_WALLET_INDEX_CODE, "Wallet index out of bounds")
			.with_field("wallet_index")
			.with_received(index.to_string())
			.with_expected(format!("0 to {}", max_index))
			.with_suggestion(format!("Use a wallet index between 0 and {}", max_index))
	}

	pub fn invalid_address_format(field: &str, address: &str, expected_format: &str) -> Self {
		Self::new(INVALID_ADDRESS_FORMAT_CODE, "Invalid address format")
			.with_field(field)
			.with_received(address.to_string())
			.with_expected(expected_format)
			.with_suggestion("Ensure the address follows the correct format")
	}

	pub fn invalid_amount(field: &str, amount: &str, reason: &str) -> Self {
		Self::new(INVALID_AMOUNT_CODE, "Invalid amount value")
			.with_field(field)
			.with_received(amount.to_string())
			.with_expected("Positive number within valid range")
			.with_suggestion(reason)
	}

	pub fn invalid_hex_format(field: &str, value: &str, expected_length: Option<usize>) -> Self {
		let expected = if let Some(len) = expected_length {
			format!("0x-prefixed hex string of {} bytes", len)
		} else {
			"Valid hexadecimal string".to_string()
		};

		Self::new(INVALID_HEX_FORMAT_CODE, "Invalid hexadecimal format")
			.with_field(field)
			.with_received(value.to_string())
			.with_expected(expected)
			.with_suggestion("Check that the value is properly hex-encoded")
	}

	pub fn account_parse_error(account: &str, error: &str) -> Self {
		Self::new(ACCOUNT_PARSE_ERROR_CODE, "Failed to parse account identifier")
			.with_field("omni_account")
			.with_received(account.to_string())
			.with_expected("Valid 32-byte account identifier")
			.with_suggestion(format!("Error: {}", error))
	}

	pub fn unexpected_response_type(expected: &str, received: &str) -> Self {
		Self::new(UNEXPECTED_RESPONSE_TYPE_CODE, "Unexpected response type from service")
			.with_expected(expected)
			.with_received(received)
			.with_suggestion("This is likely an internal error. Please contact support.")
	}

	pub fn signer_service_error(operation: &str, error: &str) -> Self {
		Self::new(SIGNER_SERVICE_ERROR_CODE, format!("Signer service error during {}", operation))
			.with_suggestion(format!("Signer error: {}", error))
	}

	pub fn email_service_error(email: &str) -> Self {
		Self::new(EMAIL_SERVICE_ERROR_CODE, "Email service error")
			.with_field("email")
			.with_received(email.to_string())
			.with_suggestion("Failed to send verification email. Please try again later.")
	}

	pub fn storage_error(operation: &str) -> Self {
		Self::new(STORAGE_SERVICE_ERROR_CODE, format!("Storage service error during {}", operation))
			.with_suggestion("Storage operation failed. Please try again later.")
	}

	// Native task error factory methods
	pub fn chain_not_supported(chain_id: u64) -> Self {
		// Get supported chains from config
		use crate::config::SUPPORTED_EVM_CHAINS;
		let supported: Vec<u64> = SUPPORTED_EVM_CHAINS.iter().map(|&c| c as u64).collect();

		Self::new(INVALID_CHAIN_ID_CODE, "Chain not supported")
			.with_field("chain_id")
			.with_received(chain_id.to_string())
			.with_expected(format!("One of: {:?}", supported))
			.with_suggestion("Please use a supported chain ID")
	}

	pub fn invalid_user_operation_error(description: &str) -> Self {
		Self::new(INVALID_USER_OPERATION_CODE, description)
	}

	pub fn gas_estimation_failed() -> Self {
		Self::new(GAS_ESTIMATION_FAILED_CODE, "Unable to estimate gas for operation")
			.with_suggestion("Please check the user operation parameters and try again")
	}

	pub fn signature_service_unavailable() -> Self {
		Self::new(SIGNATURE_SERVICE_UNAVAILABLE_CODE, "Signature service temporarily unavailable")
			.with_suggestion("Please try again in a few moments")
	}

	// Passkey/WebAuthn error factory methods
	pub fn passkey_challenge_not_found() -> Self {
		Self::new(PASSKEY_CHALLENGE_NOT_FOUND_CODE, "Passkey challenge not found")
			.with_reason("Challenge may have expired or was never created")
			.with_suggestion("Request a new challenge using omni_requestPasskeyChallenge")
	}

	pub fn passkey_challenge_expired() -> Self {
		Self::new(PASSKEY_CHALLENGE_EXPIRED_CODE, "Passkey challenge has expired")
			.with_reason("Challenge exceeded its 5-minute validity period")
			.with_suggestion("Request a new challenge using omni_requestPasskeyChallenge")
	}

	pub fn passkey_invalid_challenge(reason: &str) -> Self {
		Self::new(PASSKEY_INVALID_CHALLENGE_CODE, "Invalid passkey challenge")
			.with_reason(reason)
			.with_suggestion(
				"Ensure the challenge matches the one received from omni_requestPasskeyChallenge",
			)
	}

	pub fn passkey_not_found(credential_id: &str) -> Self {
		Self::new(PASSKEY_NOT_FOUND_CODE, "Passkey not found")
			.with_field("credential_id")
			.with_received(credential_id)
			.with_reason("No passkey registered for this account and credential ID")
			.with_suggestion("Register a new passkey using omni_attachPasskey")
	}

	pub fn passkey_signature_invalid(reason: &str) -> Self {
		Self::new(PASSKEY_SIGNATURE_INVALID_CODE, "Passkey signature verification failed")
			.with_reason(reason)
			.with_suggestion("Ensure the passkey signature is correctly generated and matches the registered public key")
	}

	pub fn passkey_parse_error(field: &str, error: &str) -> Self {
		Self::new(PASSKEY_PARSE_ERROR_CODE, "Failed to parse passkey data")
			.with_field(field)
			.with_reason(error)
			.with_suggestion("Check the passkey data format and encoding")
	}

	pub fn passkey_rp_id_mismatch(expected_rp_id: &str, client_id: &str) -> Self {
		Self::new(PASSKEY_RP_ID_MISMATCH_CODE, "Passkey RP ID validation failed")
			.with_field("rp_id")
			.with_expected(expected_rp_id)
			.with_reason(format!(
				"RP ID hash in authenticator data does not match expected RP ID for client '{}'",
				client_id
			))
			.with_suggestion("Ensure the passkey was created for the correct relying party domain")
	}

	pub fn passkey_replay_attack(counter: u32) -> Self {
		Self::new(PASSKEY_REPLAY_ATTACK_CODE, "Replay attack detected")
			.with_field("signature_counter")
			.with_received(counter.to_string())
			.with_reason(
				"Signature counter is not greater than stored value - possible replay attack",
			)
			.with_suggestion("This authentication request may have been captured and replayed by an attacker. Contact support if you believe this is an error.")
	}

	pub fn passkey_counter_validation_failed() -> Self {
		Self::new(
			PASSKEY_COUNTER_VALIDATION_FAILED_CODE,
			"Passkey counter validation failed",
		)
		.with_reason("Authenticator may have been cloned or downgraded")
		.with_suggestion("The authenticator returned a zero counter when a non-zero counter was expected. This may indicate device cloning or tampering.")
	}

	pub fn passkey_user_verification_failed(flag_type: &str) -> Self {
		Self::new(PASSKEY_USER_VERIFICATION_FAILED_CODE, "Passkey user verification failed")
			.with_field(flag_type)
			.with_reason(format!("{} flag not set in authenticator data", flag_type))
			.with_suggestion(
				"Ensure user presence and verification are enabled on the authenticator",
			)
	}

	pub fn passkey_attestation_parse_error(error: &str) -> Self {
		Self::new(
			PASSKEY_ATTESTATION_PARSE_ERROR_CODE,
			"Failed to parse passkey attestation object",
		)
		.with_reason(error)
		.with_suggestion(
			"Ensure the attestation object is properly CBOR-encoded and base64url-encoded",
		)
	}

	pub fn passkey_already_exists(credential_id: &str) -> Self {
		Self::new(PASSKEY_ALREADY_EXISTS_CODE, "Passkey already registered")
			.with_field("credential_id")
			.with_received(credential_id)
			.with_reason("A passkey with this credential ID already exists for this account")
			.with_suggestion("Use the existing passkey or remove it before registering a new one")
	}

	pub fn passkey_origin_verification_failed(expected_origin: &str) -> Self {
		Self::new(PASSKEY_ORIGIN_VERIFICATION_FAILED_CODE, "Origin verification failed")
			.with_field("client_data_json")
			.with_expected(format!("Origin: {}", expected_origin))
			.with_reason("Origin in client data does not match expected value")
			.with_suggestion("Ensure the WebAuthn ceremony is initiated from the correct origin")
	}

	pub fn passkey_client_data_parse_error(error: &str) -> Self {
		Self::new(PASSKEY_CLIENT_DATA_PARSE_ERROR_CODE, "Failed to parse client data JSON")
			.with_field("client_data_json")
			.with_reason(error)
			.with_suggestion("Ensure the client data is properly base64url-encoded JSON")
	}
}
