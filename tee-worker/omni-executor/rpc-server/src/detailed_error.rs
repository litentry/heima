use crate::error_code::{
	EMAIL_SERVICE_ERROR_CODE, GAS_ESTIMATION_FAILED_CODE, INVALID_BACKEND_RESPONSE_CODE,
	INVALID_USEROP_CODE, PUMPX_SERVICE_ERROR_CODE, SIGNER_SERVICE_ERROR_CODE,
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
}

impl From<DetailedError> for ErrorObjectOwned {
	fn from(error: DetailedError) -> Self {
		error.to_rpc_error()
	}
}
