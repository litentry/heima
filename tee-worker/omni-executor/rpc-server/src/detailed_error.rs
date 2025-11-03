use crate::error_code::{
	EMAIL_SERVICE_ERROR_CODE, GAS_ESTIMATION_FAILED_CODE, INVALID_ADDRESS_FORMAT_CODE,
	INVALID_AMOUNT_CODE, INVALID_CHAIN_ID_CODE, INVALID_HEX_FORMAT_CODE, INVALID_USEROP_CODE,
	INVALID_WALLET_INDEX_CODE, SIGNATURE_SERVICE_UNAVAILABLE_CODE, SIGNER_SERVICE_ERROR_CODE,
	STORAGE_SERVICE_ERROR_CODE, UNEXPECTED_RESPONSE_TYPE_CODE,
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

	/// Create an error from a backend API response
	pub fn from_api_response<T>(api_response: ApiResponse<T>) -> Self
	where
		T: Codec,
	{
		Self::new(ErrorCode::InternalError.code(), ErrorCode::InternalError.message())
			.with_backend_response(api_response.code as i32, api_response.message)
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

	pub fn invalid_chain_id(chain_id: u64) -> Self {
		let supported: Vec<u64> =
			crate::config::SUPPORTED_EVM_CHAINS.iter().map(|&c| c as u64).collect();

		Self::new(INVALID_CHAIN_ID_CODE, "Chain not supported")
			.with_field("chain_id")
			.with_received(chain_id.to_string())
			.with_expected(format!("One of: {:?}", supported))
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

	pub fn invalid_user_op(reason: &str) -> Self {
		Self::new(INVALID_USEROP_CODE, "Invalid UserOp").with_received(reason)
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

impl From<DetailedError> for ErrorObjectOwned {
	fn from(error: DetailedError) -> Self {
		error.to_rpc_error()
	}
}
