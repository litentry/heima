use crate::error_code::{
	EMAIL_SERVICE_ERROR_CODE, GAS_ESTIMATION_FAILED_CODE, INVALID_BACKEND_RESPONSE_CODE,
	INVALID_USEROP_CODE, PASSKEY_ALREADY_EXISTS_CODE, PASSKEY_ATTESTATION_PARSE_ERROR_CODE,
	PASSKEY_CHALLENGE_EXPIRED_CODE, PASSKEY_CHALLENGE_NOT_FOUND_CODE,
	PASSKEY_CLIENT_DATA_PARSE_ERROR_CODE, PASSKEY_COUNTER_VALIDATION_FAILED_CODE,
	PASSKEY_INVALID_CHALLENGE_CODE, PASSKEY_NOT_FOUND_CODE,
	PASSKEY_ORIGIN_VERIFICATION_FAILED_CODE, PASSKEY_PARSE_ERROR_CODE, PASSKEY_REPLAY_ATTACK_CODE,
	PASSKEY_RP_ID_MISMATCH_CODE, PASSKEY_SIGNATURE_INVALID_CODE,
	PASSKEY_USER_VERIFICATION_FAILED_CODE, PUMPX_SERVICE_ERROR_CODE, SIGNER_SERVICE_ERROR_CODE,
	STORAGE_SERVICE_ERROR_CODE, WILDMETA_SERVICE_ERROR_CODE,
};
use jsonrpsee::types::{ErrorCode, ErrorObject, ErrorObjectOwned};
use parity_scale_codec::Codec;
use pumpx::methods::common::ApiResponse;
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
	pub backend_response: Option<BackendResponse>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendResponse {
	pub code: i32,
	pub message: String,
}

impl DetailedError {
	pub fn new(code: i32, message: impl Into<String>) -> Self {
		Self {
			code,
			message: message.into(),
			details: ErrorDetails {
				backend_response: None,
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

	pub fn with_backend_response(mut self, code: i32, message: impl Into<String>) -> Self {
		self.details.backend_response = Some(BackendResponse { code, message: message.into() });
		self
	}

	pub fn to_rpc_error(&self) -> ErrorObjectOwned {
		ErrorObject::owned(self.code, self.message.clone(), Some(self.details.clone()))
	}
}

impl DetailedError {
	pub fn internal_error(reason: &str) -> Self {
		Self::new(ErrorCode::InternalError.code(), ErrorCode::InternalError.message())
			.with_reason(reason)
	}

	pub fn parse_error(reason: &str) -> Self {
		Self::new(ErrorCode::ParseError.code(), ErrorCode::ParseError.message()).with_reason(reason)
	}

	pub fn invalid_params(field: &str, reason: &str) -> Self {
		Self::new(ErrorCode::InvalidParams.code(), ErrorCode::InvalidParams.message())
			.with_field(field)
			.with_reason(reason)
	}

	pub fn invalid_backend_response<T: Codec>(response: &ApiResponse<T>, op: &str) -> Self {
		Self::new(INVALID_BACKEND_RESPONSE_CODE, "Invalid backend response")
			.with_field(op)
			.with_backend_response(response.code as i32, response.message.to_owned())
	}

	pub fn invalid_chain_id(chain_id: u64) -> Self {
		let supported: Vec<u64> =
			crate::config::SUPPORTED_EVM_CHAINS.iter().map(|&c| c as u64).collect();
		Self::invalid_params(
			"chain_id",
			&format!("invalid chain_id: {}, expected one of {:?}", chain_id, supported),
		)
	}

	pub fn signer_service_error() -> Self {
		Self::new(SIGNER_SERVICE_ERROR_CODE, "Signer service error")
	}

	pub fn email_service_error(email: &str) -> Self {
		Self::new(EMAIL_SERVICE_ERROR_CODE, "Email service error")
			.with_field("email")
			.with_received(email.to_string())
			.with_suggestion("Failed to send verification email. Please try again later.")
	}

	pub fn storage_service_error(op: &str) -> Self {
		Self::new(STORAGE_SERVICE_ERROR_CODE, format!("Storage service error in {}", op))
	}

	pub fn pumpx_service_error(op: &str, reason: impl Into<String>) -> Self {
		Self::new(PUMPX_SERVICE_ERROR_CODE, format!("Pumpx service error in {}", op))
			.with_reason(reason)
	}

	pub fn wildmeta_service_error(op: &str) -> Self {
		Self::new(WILDMETA_SERVICE_ERROR_CODE, format!("Wildmeta service error in {}", op))
	}

	pub fn invalid_user_op(reason: &str) -> Self {
		Self::new(INVALID_USEROP_CODE, "Invalid UserOp").with_received(reason)
	}

	pub fn gas_estimation_failed() -> Self {
		Self::new(GAS_ESTIMATION_FAILED_CODE, "Gas estimaton failed")
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

impl From<DetailedError> for ErrorObjectOwned {
	fn from(error: DetailedError) -> Self {
		error.to_rpc_error()
	}
}
