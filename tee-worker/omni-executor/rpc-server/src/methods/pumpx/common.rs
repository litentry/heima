use jsonrpsee::types::{ErrorCode, ErrorObjectOwned};
use parity_scale_codec::Codec;
use pumpx::types::ApiResponse;
use serde::Serialize;

use crate::{error_code::*, oneshot, server::RpcContext, Decode};
use executor_core::native_task::*;
use native_task_handler::{NativeTaskError, NativeTaskOk, NativeTaskResponse};

#[derive(Serialize, Debug)]
pub struct PumpxRpcError {
	pub code: i32,
	pub message: String,
}

impl PumpxRpcError {
	pub fn from_code_and_message(code: i32, message: String) -> Self {
		Self { code, message }
	}

	pub fn from_error_code(error_code: ErrorCode) -> Self {
		let (code, message) = match error_code {
			ErrorCode::ParseError => (-32700, "Parse error"),
			ErrorCode::InvalidParams => (-32602, "Invalid params"),
			ErrorCode::InternalError => (-32603, "Internal error"),
			ErrorCode::ServerError(n) => (n, "Server error"),
			_ => todo!(),
		};
		Self { code, message: message.to_string() }
	}
}

impl From<PumpxRpcError> for ErrorObjectOwned {
	fn from(error: PumpxRpcError) -> Self {
		ErrorObjectOwned::owned(error.code, error.message, None::<()>)
	}
}

/// Process native task and handle response
pub async fn handle_pumpx_native_task<F, R>(
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
		log::error!("Failed to send request to native call executor");
		PumpxRpcError::from_error_code(ErrorCode::InternalError)
	})?;

	// Receive response
	let response = response_receiver.await.map_err(|e| {
		log::error!("Failed to receive response from native call handler: {:?}", e);
		PumpxRpcError::from_error_code(ErrorCode::InternalError)
	})?;

	// Decode response
	let native_task_response: NativeTaskResponse = Decode::decode(&mut response.as_slice())
		.map_err(|_| {
			log::error!("Failed to decode native task response");
			PumpxRpcError::from_error_code(ErrorCode::InternalError)
		})?;

	// Process response
	match native_task_response {
		Ok(task_ok) => task_ok_handler(task_ok),
		Err(NativeTaskError::InternalError) => {
			log::error!("Internal error in native task");
			Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
		},
		Err(native_task_error) => {
			log::error!("Failed to execute native task: {:?}", native_task_error);
			Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(get_native_task_error_code(
				&native_task_error,
			))))
		},
	}
}

pub fn check_pumpx_api_response<T>(
	response: ApiResponse<T>,
	name: String,
) -> Result<(), PumpxRpcError>
where
	T: Codec,
{
	if response.code != 10000 {
		log::error!("{} failed: code={}, message={}", name, response.code, response.message);
		return Err(PumpxRpcError::from_code_and_message(response.code as i32, response.message));
	}
	Ok(())
}
