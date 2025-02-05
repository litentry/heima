use crate::{
	error_code::*,
	native_call_authenticated::{verify_native_call_authenticated, NativeCallAuthenticated},
	request::PlainRequest,
	server::RpcContext,
};
use executor_core::native_call::NativeCall;
use executor_primitives::{
	utils::hex::{FromHexPrefixed, ToHexPrefixed},
	OmniAccountAuthType,
};
use jsonrpsee::{
	types::{ErrorCode, ErrorObject},
	RpcModule,
};
use native_task_handler::NativeTask;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use parity_scale_codec::Decode;
use std::sync::Arc;
use tokio::{runtime::Handle, sync::oneshot, task};

pub fn register_submit_plain_request<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<Header, RpcClient, RpcClientFactory>>,
) {
	module
		.register_async_method("native_submitPlainRequest", |params, ctx, _| async move {
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
			let native_task = NativeTask { call: native_call, auth_type, response_sender };

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
		.expect("Failed to register native_submitPlainRequest method");
}

fn handle_plain_request<
	'a,
	Header,
	RpcClient: SubstrateRpcClient<Header>,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient>,
>(
	request: PlainRequest,
	ctx: Arc<RpcContext<Header, RpcClient, RpcClientFactory>>,
	handle: Handle,
) -> Result<(NativeCall, OmniAccountAuthType), ErrorObject<'a>> {
	if request.mrenclave != ctx.mrenclave {
		return Err(ErrorCode::ServerError(INVALID_MRENCLAVE_CODE).into());
	}
	let nca = NativeCallAuthenticated::decode(&mut request.payload.as_slice())
		.map_err(|_| ErrorCode::ServerError(INVALID_NATIVE_CALL_AUTHENTICATED_CODE))?;

	if verify_native_call_authenticated(ctx, handle, &nca).is_err() {
		return Err(ErrorCode::ServerError(AUTHENTICATION_FAILED_CODE).into());
	}

	Ok((nca.call, nca.authentication.into()))
}
