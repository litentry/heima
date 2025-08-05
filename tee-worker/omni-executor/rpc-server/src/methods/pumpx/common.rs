use crate::{error_code::*, oneshot, server::RpcContext, Decode};
use executor_core::intent_executor::IntentExecutor;
use executor_core::native_task::*;
use jsonrpsee::types::{ErrorCode, ErrorObjectOwned};
use native_task_handler::{handle_native_task, NativeTaskError, NativeTaskOk, NativeTaskResponse};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use parity_scale_codec::Codec;
use pumpx::methods::common::ApiResponse;
use serde::Serialize;
use tracing::error;

#[derive(Serialize, Debug)]
pub struct PumpxRpcError {
	pub code: i32,
	pub message: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub data: Option<PumpxRpcErrorData>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
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
pub async fn handle_pumpx_native_task<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
	F,
	R,
>(
	ctx: &RpcContext<
		Header,
		RpcClient,
		RpcClientFactory,
		EthereumIntentExecutor,
		SolanaIntentExecutor,
		CrossChainIntentExecutor,
	>,
	wrapper: NativeTaskWrapper<NativeTask>,
	task_ok_handler: F,
) -> Result<R, PumpxRpcError>
where
	F: FnOnce(NativeTaskOk) -> Result<R, PumpxRpcError>,
{
	let native_task_response = handle_native_task(ctx.to_task_handler_context(), wrapper).await;

	// Process response
	match native_task_response {
		Ok(task_ok) => task_ok_handler(task_ok),
		Err(NativeTaskError::InternalError) => {
			error!("Internal error in native task");
			Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
		},
		Err(native_task_error) => {
			error!("Failed to execute native task: {:?}", native_task_error);
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
		error!("{} failed: code={}, message={}", name, response.code, response.message);
		return Err(PumpxRpcError::from_api_response(response));
	}
	Ok(())
}

pub fn check_and_get_option_response_data<T>(
	data: Option<T>,
	code: i32,
	message: &str,
) -> Result<T, PumpxRpcError> {
	data.ok_or_else(|| {
		error!("{}", message);
		PumpxRpcError::from_error_code(ErrorCode::ServerError(code))
	})
}
