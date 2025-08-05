use http::Extensions;
use jsonrpsee::types::{ErrorCode, ErrorObjectOwned};
use parity_scale_codec::Codec;
use pumpx::methods::common::ApiResponse;
use serde::Serialize;

use crate::{error_code::*, middlewares::RpcExtensions, server::RpcContext};
use executor_core::intent_executor::IntentExecutor;
use executor_core::native_task::*;
use native_task_handler::{handle_native_task, NativeTaskError, NativeTaskOk};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
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
pub async fn handle_omni_native_task<
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
	// We handle the task right here
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

pub fn check_omni_api_response<T>(
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

pub struct User {
	pub omni_account: String,
	pub client_id: String,
}

/// This is used to verify that the request is authenticated.
/// If the RpcExtensions is not found, it indicates that the request is not authenticated.
/// If the RpcExtensions is found, it contains the sender's omni account extracted from the JWT.
/// Check rpc_middleware.rs
pub fn check_auth(ext: &Extensions) -> Result<User, ()> {
	if let Some(rpc_extensions) = ext.get::<RpcExtensions>() {
		return Ok(User {
			omni_account: rpc_extensions.sender.clone(),
			client_id: rpc_extensions.client_id.clone(),
		});
	}
	Err(())
}
