use crate::{
	error_code::*,
	native_call_authenticated::{verify_native_call_authenticated, NativeCallAuthenticated},
	request::PlainRequest,
	server::RpcContext,
};
use executor_primitives::utils::hex::{FromHexPrefixed, ToHexPrefixed};
use jsonrpsee::{
	types::{ErrorCode, ErrorObject},
	RpcModule,
};
use native_task_handler::NativeTask;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use parity_scale_codec::Decode;
use tokio::{runtime::Handle, sync::oneshot, task};

pub fn register_submit_plain_request<
	AccountId: Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<AccountId, Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<AccountId, Header, RpcClient, RpcClientFactory>>,
) {
	module
		.register_async_method("native_submitPlainRequest", |params, ctx, _| async move {
			let Ok(hex_request) = params.one::<String>() else {
				return Err(ErrorCode::ParseError.into());
			};
			let Ok(request) = PlainRequest::from_hex(&hex_request) else {
				return Err(ErrorCode::ServerError(INVALID_PLAIN_REQUEST_CODE).into());
			};
			let nca = NativeCallAuthenticated::decode(&mut request.payload.as_slice())
				.map_err(|_| ErrorCode::ServerError(INVALID_NATIVE_CALL_AUTHENTICATED_CODE))?;

			task::spawn_blocking({
				let ctx = ctx.clone();
				let nca = nca.clone();
				move || {
					verify_native_call_authenticated(ctx, &request.shard, Handle::current(), &nca)
				}
			})
			.await
			.map_err(|e| {
				log::error!("Failed to verify native call authenticated: {:?}", e);
				ErrorCode::InternalError
			})?
			.map_err(|e| {
				log::error!("Failed to verify native call authenticated: {:?}", e);
				ErrorCode::ServerError(AUTHENTICATION_FAILED_CODE)
			})?;

			let (response_sender, response_receiver) = oneshot::channel();

			let native_task = NativeTask {
				call: nca.call,
				auth_type: nca.authentication.into(),
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
		.expect("Failed to register native_submitPlainRequest method");
}
