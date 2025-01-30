use crate::{
	error_code::*,
	native_call_authenticated::{verify_native_call_authenticated, NativeCallAuthenticated},
	request::{AesRequest, DecryptableRequest},
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
use parity_scale_codec::{Decode, Encode};
use std::sync::Arc;
use tokio::{runtime::Handle, sync::oneshot, task};

pub fn register_submit_aes_request<
	AccountId: Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<AccountId, Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<AccountId, Header, RpcClient, RpcClientFactory>>,
) {
	module
		.register_async_method("native_submitAesRequest", |params, ctx, _| async move {
			let Ok(hex_request) = params.one::<String>() else {
				return Err(ErrorCode::ParseError.into());
			};
			let Ok(request) = AesRequest::from_hex(&hex_request) else {
				return Err(ErrorCode::ServerError(INVALID_AES_REQUEST_CODE).into());
			};
			let join_handle = task::spawn_blocking({
				let ctx = ctx.clone();
				let aes_request = request.clone();
				|| handle_aes_request(aes_request, ctx, Handle::current())
			});
			let (native_call, auth_type) = join_handle.await.map_err(|e| {
				log::error!("Failed to handle AES request: {:?}", e);
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
		.expect("Failed to register native_submitAesRequest method");
}

fn handle_aes_request<
	'a,
	AccountId,
	Header,
	RpcClient: SubstrateRpcClient<AccountId, Header>,
	RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, RpcClient>,
>(
	mut request: AesRequest,
	ctx: Arc<RpcContext<AccountId, Header, RpcClient, RpcClientFactory>>,
	handle: Handle,
) -> Result<(NativeCall, OmniAccountAuthType), ErrorObject<'a>> {
	if request.shard().encode() != ctx.mrenclave.encode() {
		return Err(ErrorCode::ServerError(INVALID_SHARD_CODE).into());
	}
	let Ok(encoded_nca) = request.decrypt(Box::new(ctx.shielding_key.clone())) else {
		return Err(ErrorCode::ServerError(REQUEST_DECRYPTION_FAILED_CODE).into());
	};
	let nca = match NativeCallAuthenticated::decode(&mut encoded_nca.as_slice()) {
		Ok(nca) => nca,
		Err(e) => {
			log::error!("Failed to decode authenticated call: {:?}", e);
			return Err(ErrorCode::ServerError(INVALID_AUTHENTICATED_CALL_CODE).into());
		},
	};
	if verify_native_call_authenticated(ctx, &request.shard(), handle, &nca).is_err() {
		return Err(ErrorCode::ServerError(AUTHENTICATION_FAILED_CODE).into());
	}

	Ok((nca.call, nca.authentication.into()))
}
