use crate::{
	error_code::*,
	native_operation_authenticated::{
		verify_native_operation_authenticated, AuthenticatedOperation,
	},
	request::PlainRequest,
	server::RpcContext,
};
use executor_core::native_operation::{NativeOperation, NativeQuery};
use executor_primitives::{
	utils::hex::{FromHexPrefixed, ToHexPrefixed},
	OmniAccountAuthType,
};
use jsonrpsee::{
	types::{ErrorCode, ErrorObject},
	RpcModule,
};
use native_task_handler::{NativeTask, NativeTaskOperation};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use parity_scale_codec::Decode;
use std::sync::Arc;
use tokio::{runtime::Handle, sync::oneshot, task};

pub fn register_submit_plain_requests<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<Header, RpcClient, RpcClientFactory>>,
) {
	module
		.register_async_method("native_submitCallPlainRequest", |params, ctx, _| async move {
			let Ok(hex_request) = params.one::<String>() else {
				return Err(ErrorCode::ParseError.into());
			};
			let Ok(request) = PlainRequest::from_hex(&hex_request) else {
				return Err(ErrorCode::ServerError(INVALID_PLAIN_REQUEST_CODE).into());
			};
			let join_handle = task::spawn_blocking({
				let ctx = ctx.clone();
				let plain_request = request.clone();
				|| handle_plain_request(plain_request, ctx, Handle::current())
			});
			let (native_call, auth_type) = join_handle.await.map_err(|e| {
				log::error!("Failed to handle Plain request: {:?}", e);
				ErrorCode::InternalError
			})??;
			let (response_sender, response_receiver) = oneshot::channel();
			let native_task = NativeTask {
				operation: NativeTaskOperation::Call(native_call),
				auth_type,
				response_sender,
			};
			if ctx.native_task_sender.send(native_task).await.is_err() {
				log::error!("Failed to send request to native call executor");
				return Err(ErrorCode::InternalError.into());
			}
			match response_receiver.await {
				Ok(response) => Ok::<String, ErrorObject>(response.to_hex()),
				Err(e) => {
					log::error!("Failed to receive response from native call handler: {:?}", e);
					Err(ErrorCode::InternalError.into())
				},
			}
		})
		.expect("Failed to register native_submitCallPlainRequest method");

	module
		.register_async_method("native_submitQueryPlainRequest", |params, ctx, _| async move {
			let Ok(hex_request) = params.one::<String>() else {
				return Err(ErrorCode::ParseError.into());
			};
			let Ok(request) = PlainRequest::from_hex(&hex_request) else {
				return Err(ErrorCode::ServerError(INVALID_PLAIN_REQUEST_CODE).into());
			};
			let join_handle = task::spawn_blocking({
				let ctx = ctx.clone();
				let plain_request = request.clone();
				|| handle_plain_request(plain_request, ctx, Handle::current())
			});
			let (native_query, auth_type) = join_handle.await.map_err(|e| {
				log::error!("Failed to handle Plain request: {:?}", e);
				ErrorCode::InternalError
			})??;
			let (response_sender, response_receiver) = oneshot::channel();
			let native_task = NativeTask {
				operation: NativeTaskOperation::Query(native_query),
				auth_type,
				response_sender,
			};
			if ctx.native_task_sender.send(native_task).await.is_err() {
				log::error!("Failed to send request to native call executor");
				return Err(ErrorCode::InternalError.into());
			}
			match response_receiver.await {
				Ok(response) => Ok::<String, ErrorObject>(response.to_hex()),
				Err(e) => {
					log::error!("Failed to receive response from native call handler: {:?}", e);
					Err(ErrorCode::InternalError.into())
				},
			}
		})
		.expect("Failed to register native_submitQueryPlainRequest method");
}

pub fn handle_plain_request<
	'a,
	Header,
	RpcClient: SubstrateRpcClient<Header>,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient>,
	OP: NativeOperation,
>(
	request: PlainRequest,
	ctx: Arc<RpcContext<Header, RpcClient, RpcClientFactory>>,
	handle: Handle,
) -> Result<(OP, OmniAccountAuthType), ErrorObject<'a>> {
	if request.mrenclave != ctx.mrenclave {
		return Err(ErrorCode::ServerError(INVALID_MRENCLAVE_CODE).into());
	}
	let authenticated_op = AuthenticatedOperation::decode(&mut request.payload.as_slice())
		.map_err(|_| ErrorCode::ServerError(INVALID_NATIVE_CALL_AUTHENTICATED_CODE))?;

	if verify_native_operation_authenticated(ctx, handle, &authenticated_op).is_err() {
		return Err(ErrorCode::ServerError(AUTHENTICATION_FAILED_CODE).into());
	}

	Ok((authenticated_op.operation, authenticated_op.authentication.into()))
}
