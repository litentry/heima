use jsonrpsee::types::{ErrorCode, ErrorObjectOwned};
use parity_scale_codec::Codec;
use pumpx::methods::common::ApiResponse;
use serde::Serialize;

use crate::{error_code::*, oneshot, server::RpcContext, Decode};
use executor_core::native_task::*;
use native_task_handler::{NativeTaskError, NativeTaskOk, NativeTaskResponse};

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
			code: ErrorCode::InternalError.code(),
			message: ErrorCode::InternalError.message().to_string(),
			data: Some(PumpxRpcErrorData {
				backend_response: Some(PumpxRpcErrorBackendResponse {
					code: api_response.code as i32,
					message: api_response.message,
				}),
			}),
		}
	}
}

impl From<PumpxRpcError> for ErrorObjectOwned {
	fn from(error: PumpxRpcError) -> Self {
		ErrorObjectOwned::owned(error.code, error.message, error.data)
	}
}

/// Process native task and handle response
pub async fn handle_omni_native_task<F, R>(
	ctx: &RpcContext,
	wrapper: NativeTaskWrapper<NativeTask>,
	task_ok_handler: F,
) -> Result<R, PumpxRpcError>
where
	F: FnOnce(NativeTaskOk) -> Result<R, PumpxRpcError>,
{
	// Create channel for response
	let (response_sender, response_receiver) = oneshot::channel();

	// Send task to executor
	ctx.native_task_sender.send((wrapper, response_sender)).await.map_err(|_| {
		tracing::log::error!("Failed to send request to native call executor");
		PumpxRpcError::from_error_code(ErrorCode::InternalError)
	})?;

	// Receive response
	let response = response_receiver.await.map_err(|e| {
		tracing::log::error!("Failed to receive response from native call handler: {:?}", e);
		PumpxRpcError::from_error_code(ErrorCode::InternalError)
	})?;

	// Decode response
	let native_task_response: NativeTaskResponse = Decode::decode(&mut response.as_slice())
		.map_err(|_| {
			tracing::log::error!("Failed to decode native task response");
			PumpxRpcError::from_error_code(ErrorCode::InternalError)
		})?;

	// Process response
	match native_task_response {
		Ok(task_ok) => task_ok_handler(task_ok),
		Err(NativeTaskError::InternalError) => {
			tracing::log::error!("Internal error in native task");
			Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
		},
		Err(native_task_error) => {
			tracing::log::error!("Failed to execute native task: {:?}", native_task_error);
			Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(get_native_task_error_code(
				&native_task_error,
			))))
		},
	}
}

pub fn check_omni_api_response<T>(
	response: ApiResponse<T>,
	name: String,
) -> Result<(), PumpxRpcError>
where
	T: Codec,
{
	if response.code != 10000 {
		tracing::log::error!(
			"{} failed: code={}, message={}",
			name,
			response.code,
			response.message
		);
		return Err(PumpxRpcError::from_api_response(response));
	}
	Ok(())
}
