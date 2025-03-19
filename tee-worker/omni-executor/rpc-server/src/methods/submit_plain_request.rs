use crate::{
	error_code::*,
	native_operation_authenticated::{
		verify_native_operation_authenticated, AuthenticatedOperation,
	},
	request::PlainRequest,
	server::RpcContext,
};
use executor_core::native_operation::NativeOperation;
use executor_primitives::{
	utils::hex::{hex_encode, FromHexPrefixed},
	OmniAccountAuthType,
};
use jsonrpsee::{
	types::{ErrorCode, ErrorObject, Params},
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
			let (native_call, auth_type) =
				handle_plain_request(params, ctx.clone()).await.map_err(|e| {
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
				Ok(response) => Ok::<String, ErrorObject>(hex_encode(response.as_slice())),
				Err(e) => {
					log::error!("Failed to receive response from native call handler: {:?}", e);
					Err(ErrorCode::InternalError.into())
				},
			}
		})
		.expect("Failed to register native_submitCallPlainRequest method");

	module
		.register_async_method("native_submitQueryPlainRequest", |params, ctx, _| async move {
			let (native_query, auth_type) =
				handle_plain_request(params, ctx.clone()).await.map_err(|e| {
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
				log::error!("Failed to send request to native query executor");
				return Err(ErrorCode::InternalError.into());
			}
			match response_receiver.await {
				Ok(response) => Ok::<String, ErrorObject>(hex_encode(response.as_slice())),
				Err(e) => {
					log::error!("Failed to receive response from native query handler: {:?}", e);
					Err(ErrorCode::InternalError.into())
				},
			}
		})
		.expect("Failed to register native_submitQueryPlainRequest method");
}

fn handle_plain_request<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
	OP: NativeOperation + Send + Sync + 'static,
>(
	params: Params<'static>,
	ctx: Arc<RpcContext<Header, RpcClient, RpcClientFactory>>,
) -> task::JoinHandle<Result<(OP, OmniAccountAuthType), ErrorObject<'_>>> {
	task::spawn_blocking(move || {
		let Ok(hex_request) = params.one::<String>() else {
			return Err(ErrorCode::ParseError.into());
		};
		let Ok(request) = PlainRequest::from_hex(&hex_request) else {
			return Err(ErrorCode::ServerError(INVALID_PLAIN_REQUEST_CODE).into());
		};
		if request.mrenclave != ctx.mrenclave {
			return Err(ErrorCode::ServerError(INVALID_MRENCLAVE_CODE).into());
		}
		let authenticated_op = AuthenticatedOperation::decode(&mut request.payload.as_slice())
			.map_err(|_| ErrorCode::ServerError(INVALID_NATIVE_CALL_AUTHENTICATED_CODE))?;

		if verify_native_operation_authenticated(ctx, Handle::current(), &authenticated_op).is_err()
		{
			return Err(ErrorCode::ServerError(AUTHENTICATION_FAILED_CODE).into());
		}

		Ok((authenticated_op.operation, authenticated_op.authentication.into()))
	})
}
