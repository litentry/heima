use crate::{
	error_code::*,
	hex_encode, oneshot,
	server::RpcContext,
	task::{DecryptableTask, RawTask},
	verify_auth::*,
	FromHexPrefixed,
};
use executor_core::native_task::{NativeTask, NativeTaskTrait, NativeTaskWrapper};
use executor_crypto::aes256::{aes_encrypt_default, Aes256Key};
use jsonrpsee::{
	types::{ErrorCode, ErrorObject, Params},
	RpcModule,
};
use parity_scale_codec::{Decode, Encode};
use std::sync::Arc;
use tracing::error;

pub fn register_submit_native_task(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_submitNativeTask", |params, ctx, _| async move {
			let (wrapper, maybe_aes_key) = parse(params, ctx.clone()).await.map_err(|e| {
				error!("Failed to parse: {:?}", e);
				ErrorCode::InternalError
			})?;
			let (response_sender, response_receiver) = oneshot::channel();

			if ctx.native_task_sender.send((wrapper, response_sender)).await.is_err() {
				error!("Failed to send request to native call executor");
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
					error!("Failed to receive response from native call handler: {:?}", e);
					Err(ErrorCode::InternalError.into())
				},
			}
		})
		.expect("Failed to register omni_submitNativeTask method");
}

type ParseResult<'a> = Result<(NativeTaskWrapper<NativeTask>, Option<Aes256Key>), ErrorObject<'a>>;

async fn parse(params: Params<'static>, ctx: Arc<RpcContext>) -> ParseResult<'static> {
	let Ok(hex_request) = params.one::<String>() else {
		error!("Failed to parse params: {:?}", params);
		return Err(ErrorCode::ParseError.into());
	};
	let Ok(request) = RawTask::<NativeTask>::from_hex(&hex_request) else {
		error!("Failed to parse request: {:?}", hex_request);
		return Err(ErrorCode::ServerError(INVALID_RAW_REQUEST_CODE).into());
	};

	let request_is_encrypted = request.is_encrypted();

	let (wrapper, maybe_aes_key) = match request {
		RawTask::Plain(w) => (w, None),
		RawTask::Aes(mut r) => {
			let key = r.decrypt_aes_key(Box::new(ctx.shielding_key.clone())).map_err(|_| {
				error!("Failed to decrypt AES key");
				ErrorCode::ServerError(DECRYPT_REQUEST_FAILED_CODE)
			})?;
			let r = r.decrypt(Box::new(ctx.shielding_key.clone())).map_err(|_| {
				error!("Failed to decrypt request");
				ErrorCode::ServerError(DECRYPT_REQUEST_FAILED_CODE)
			})?;
			(
				NativeTaskWrapper::<NativeTask>::decode(&mut r.as_slice()).map_err(|_| {
					error!("Failed to decode request");
					ErrorCode::ServerError(DECODE_REQUEST_FAILED_CODE)
				})?,
				Some(key),
			)
		},
	};

	if wrapper.task.require_encrypt() && !request_is_encrypted {
		error!("Request is not encrypted, but it is required");
		return Err(ErrorCode::ServerError(REQUIRE_ENCRYPTED_REQUEST_CODE).into());
	}

	if wrapper.task.require_auth() {
		let Some(ref auth) = wrapper.auth else {
			error!("Request requires authentication, but no auth provided");
			return Err(ErrorCode::ServerError(REQUIRE_AUTHENTICATION_CODE).into());
		};
		verify_auth(ctx, auth).await.map_err(|_| {
			error!("Failed to verify auth: {:?}", wrapper.auth);
			ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE)
		})?;
	}

	Ok((wrapper, maybe_aes_key))
}
