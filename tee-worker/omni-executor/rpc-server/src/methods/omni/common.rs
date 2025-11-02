use http::Extensions;
use jsonrpsee::types::{ErrorCode, ErrorObject, ErrorObjectOwned};
use parity_scale_codec::Codec;
use pumpx::methods::common::ApiResponse;
use serde::Serialize;

use crate::{
	detailed_error::DetailedError, error_code::INTERNAL_ERROR_CODE, middlewares::RpcExtensions,
};
use tracing::error;

#[derive(Serialize, Debug)]
pub struct PumpxRpcError {
	pub code: i32,
	pub message: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub data: Option<PumpxRpcErrorData>,
}

#[derive(Serialize, Debug)]
pub struct PumpxRpcErrorData {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub backend_response: Option<PumpxRpcErrorBackendResponse>,
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

#[derive(Serialize, Debug)]
pub struct PumpxRpcErrorBackendResponse {
	pub code: i32,
	pub message: String,
}

impl PumpxRpcError {
	pub fn from_code_and_message(code: i32, message: String) -> Self {
		Self { code, message, data: None }
	}

	pub fn from_error_code(error_code: ErrorCode) -> Self {
		Self { code: error_code.code(), message: error_code.message().to_string(), data: None }
	}

	pub fn from_api_response<T>(api_response: ApiResponse<T>) -> Self
	where
		T: Codec,
	{
		Self {
			code: INTERNAL_ERROR_CODE,
			message: ErrorCode::InternalError.message().to_string(),
			data: Some(PumpxRpcErrorData {
				backend_response: Some(PumpxRpcErrorBackendResponse {
					code: api_response.code as i32,
					message: api_response.message,
				}),
				field: None,
				expected: None,
				received: None,
				reason: None,
				suggestion: None,
			}),
		}
	}
}

impl From<PumpxRpcError> for ErrorObjectOwned {
	fn from(error: PumpxRpcError) -> Self {
		ErrorObjectOwned::owned(error.code, error.message, error.data)
	}
}

impl From<DetailedError> for PumpxRpcError {
	fn from(error: DetailedError) -> Self {
		Self {
			code: error.code,
			message: error.message,
			data: Some(PumpxRpcErrorData {
				backend_response: None,
				field: error.details.field,
				expected: error.details.expected,
				received: error.details.received,
				reason: error.details.reason,
				suggestion: error.details.suggestion,
			}),
		}
	}
}

impl From<Box<DetailedError>> for PumpxRpcError {
	fn from(error: Box<DetailedError>) -> Self {
		Self::from(*error)
	}
}

// Removed: handle_omni_native_task - all RPC methods now call handlers directly

pub fn check_omni_api_response<T>(response: ApiResponse<T>, name: String) -> Result<(), ErrorObject>
where
	T: Codec,
{
	if response.code != 10000 {
		error!("{} failed: code={}, message={}", name, response.code, response.message);
		return Err(DetailedError::from_api_response(response).into());
	}
	Ok(())
}

/// This is used to verify that the request is authenticated.
/// If the RpcExtensions is not found, it indicates that the request is not authenticated.
/// If the RpcExtensions is found, it contains the sender's omni account extracted from the JWT.
/// Check rpc_middleware.rs
pub fn check_auth(ext: &Extensions) -> Result<String, ()> {
	if let Some(rpc_extensions) = ext.get::<RpcExtensions>() {
		return Ok(rpc_extensions.sender.clone());
	}
	Err(())
}
