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
	pub suggestion: Option<String>,
}

impl DetailedError {
	pub fn new(code: i32, message: impl Into<String>) -> Self {
		Self {
			code,
			message: message.into(),
			details: ErrorDetails { field: None, expected: None, received: None, suggestion: None },
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

	pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
		self.details.suggestion = Some(suggestion.into());
		self
	}

	pub fn to_error_object(&self) -> ErrorObject<'static> {
		ErrorObject::owned(self.code, self.message.clone(), Some(self.details.clone()))
	}
}

// Input Validation Error Codes (-32100 to -32119)
pub const INVALID_CHAIN_ID_CODE: i32 = -32100;
pub const INVALID_WALLET_INDEX_CODE: i32 = -32101;
pub const INVALID_ADDRESS_FORMAT_CODE: i32 = -32102;
pub const INVALID_AMOUNT_CODE: i32 = -32103;
pub const INVALID_TOKEN_ADDRESS_CODE: i32 = -32104;
pub const MISSING_REQUIRED_FIELD_CODE: i32 = -32105;
pub const INVALID_HEX_FORMAT_CODE: i32 = -32106;
pub const INVALID_EMAIL_FORMAT_CODE: i32 = -32107;
// Account/Identity Error Codes (-32120 to -32139)
pub const ACCOUNT_PARSE_ERROR_CODE: i32 = -32121;
pub const INVALID_ACCOUNT_LENGTH_CODE: i32 = -32124;

// Operation Error Codes (-32140 to -32159)
// Note: Currently no operation error codes are in use

// External Service Error Codes (-32160 to -32179)
pub const SIGNER_SERVICE_ERROR_CODE: i32 = -32160;
pub const EMAIL_SERVICE_ERROR_CODE: i32 = -32162;
pub const STORAGE_SERVICE_ERROR_CODE: i32 = -32163;
pub const EXTERNAL_API_ERROR_CODE: i32 = -32164;

// Response Processing Error Codes (-32180 to -32199)
pub const UNEXPECTED_RESPONSE_TYPE_CODE: i32 = -32180;

// Native Task Error Codes (-32200 to -32219)
pub const UNSUPPORTED_CHAIN_CODE: i32 = -32200;
pub const INVALID_USER_OPERATION_CODE: i32 = -32201;
pub const GAS_ESTIMATION_FAILED_CODE: i32 = -32202;
pub const SIGNATURE_SERVICE_UNAVAILABLE_CODE: i32 = -32203;

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
		Self::new(SIGNER_SERVICE_ERROR_CODE, "Signer service error")
			.with_field("operation")
			.with_received(operation.to_string())
			.with_suggestion(format!("Signer error: {}", error))
	}

	pub fn email_service_error(_email: &str, _client_id: &str) -> Self {
		Self::new(EMAIL_SERVICE_ERROR_CODE, "Email service error")
			.with_field("email")
			.with_suggestion("Failed to send verification email. Please try again later.")
	}

	pub fn storage_error(operation: &str, _key: &str) -> Self {
		Self::new(STORAGE_SERVICE_ERROR_CODE, "Storage service error")
			.with_field("operation")
			.with_received(operation.to_string())
			.with_suggestion("Storage operation failed. Please try again later.")
	}

	// Native task error factory methods
	pub fn chain_not_supported(chain_id: u64) -> Self {
		// Get supported chains from config
		use crate::config::SUPPORTED_EVM_CHAINS;
		let supported: Vec<u64> = SUPPORTED_EVM_CHAINS.iter().map(|&c| c as u64).collect();

		Self::new(UNSUPPORTED_CHAIN_CODE, "Chain not supported")
			.with_field("chain_id")
			.with_received(chain_id.to_string())
			.with_expected(format!("One of: {:?}", supported))
			.with_suggestion("Please use a supported chain ID")
	}

	pub fn invalid_user_operation_error(description: &str) -> Self {
		Self::new(INVALID_USER_OPERATION_CODE, "Invalid user operation")
			.with_suggestion(description)
	}

	pub fn gas_estimation_failed() -> Self {
		Self::new(GAS_ESTIMATION_FAILED_CODE, "Unable to estimate gas for operation")
			.with_suggestion("Please check the user operation parameters and try again")
	}

	pub fn signature_service_unavailable() -> Self {
		Self::new(SIGNATURE_SERVICE_UNAVAILABLE_CODE, "Signature service temporarily unavailable")
			.with_suggestion("Please try again in a few moments")
	}
}
