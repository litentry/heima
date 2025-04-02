use crate::{
	error_code::*,
	server::RpcContext,
	task::{DecryptableTask, RawTask},
	verify_auth::*,
};
use executor_core::native_task::{NativeTask, NativeTaskTrait, NativeTaskWrapper};
use executor_crypto::aes256::{aes_encrypt_default, Aes256Key};
use executor_primitives::{
	utils::hex::{hex_encode, FromHexPrefixed},
	OmniAuth,
};
use jsonrpsee::{
	types::{ErrorCode, ErrorObject, Params},
	RpcModule,
};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use parity_scale_codec::{Decode, Encode};
use std::sync::Arc;
use tokio::{runtime::Handle, sync::oneshot, task};

pub fn register_submit_native_task<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<Header, RpcClient, RpcClientFactory>>,
) {
	module
		.register_async_method("omni_submitNativeTask", |params, ctx, _| async move {
			let (wrapper, maybe_aes_key) = parse(params, ctx.clone()).await.map_err(|e| {
				log::error!("Failed to parse: {:?}", e);
				ErrorCode::InternalError
			})?;
			let (response_sender, response_receiver) = oneshot::channel();

			if ctx.native_task_sender.send((wrapper, response_sender)).await.is_err() {
				log::error!("Failed to send request to native call executor");
				return Err(ErrorCode::InternalError.into());
			}
			match response_receiver.await {
				Ok(response) => {
					let response = if let Some(aes_key) = maybe_aes_key {
						aes_encrypt_default(&aes_key, &response).encode()
					} else {
						response
					};
					Ok::<String, ErrorObject>(hex_encode(response.as_slice()))
				},
				Err(e) => {
					log::error!("Failed to receive response from native call handler: {:?}", e);
					Err(ErrorCode::InternalError.into())
				},
			}
		})
		.expect("Failed to register omni_submitNativeTask method");
}

type ParseResult<'a> = Result<(NativeTaskWrapper<NativeTask>, Option<Aes256Key>), ErrorObject<'a>>;

async fn parse<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	params: Params<'static>,
	ctx: Arc<RpcContext<Header, RpcClient, RpcClientFactory>>,
) -> ParseResult {
	let Ok(hex_request) = params.one::<String>() else {
		return Err(ErrorCode::ParseError.into());
	};
	let Ok(request) = RawTask::<NativeTask>::from_hex(&hex_request) else {
		return Err(ErrorCode::ServerError(INVALID_RAW_REQUEST_CODE).into());
	};

	let request_is_encrypted = request.is_encrypted();

	let (wrapper, maybe_aes_key) = match request {
		RawTask::Plain(w) => (w, None),
		RawTask::Aes(mut r) => {
			let key = r
				.decrypt_aes_key(Box::new(ctx.shielding_key.clone()))
				.map_err(|_| ErrorCode::ServerError(DECRYPT_REQUEST_FAILED_CODE))?;
			let r = r
				.decrypt(Box::new(ctx.shielding_key.clone()))
				.map_err(|_| ErrorCode::ServerError(DECRYPT_REQUEST_FAILED_CODE))?;
			(
				NativeTaskWrapper::<NativeTask>::decode(&mut r.as_slice())
					.map_err(|_| ErrorCode::ServerError(DECODE_REQUEST_FAILED_CODE))?,
				Some(key),
			)
		},
	};

	if wrapper.task.require_encrypt() && !request_is_encrypted {
		return Err(ErrorCode::ServerError(REQUIRE_ENCRYPTED_REQUEST_CODE).into());
	}

	if wrapper.task.require_auth() && verify_auth(ctx, &wrapper).await.is_err() {
		return Err(ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE).into());
	}

	Ok((wrapper, maybe_aes_key))
}

pub async fn verify_auth<
	Header,
	RpcClient: SubstrateRpcClient<Header>,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient>,
>(
	ctx: Arc<RpcContext<Header, RpcClient, RpcClientFactory>>,
	wrapper: &NativeTaskWrapper<NativeTask>,
) -> Result<(), AuthenticationError> {
	match wrapper.auth {
		None => Err(AuthenticationError::AuthNotExist),
		Some(OmniAuth::Web3(ref signature)) => {
			verify_web3_authentication(signature, &wrapper.task, wrapper.nonce, ctx.mrenclave)
		},
		Some(OmniAuth::Email(ref verification_code)) => {
			verify_email_authentication(ctx, wrapper.task.sender(), verification_code)
		},
		Some(OmniAuth::OAuth2(ref oauth2_data)) => {
			verify_oauth2_authentication(ctx, wrapper.task.sender(), oauth2_data).await
		},
		Some(OmniAuth::AuthToken(ref auth_token)) => {
			verify_auth_token_authentication(ctx, wrapper.task.sender(), auth_token).await
		},
	}
}
