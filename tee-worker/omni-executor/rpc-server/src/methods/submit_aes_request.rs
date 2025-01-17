use crate::{
	authentication::{
		verify_auth_token_authentication, verify_email_authentication, verify_web3_authentication,
		Authentication,
	},
	error_code::*,
	request::{AesRequest, DecryptableRequest},
	server::RpcContext,
	utils::hex::{FromHexPrefixed, ToHexPrefixed},
};
use jsonrpsee::{
	types::{ErrorCode, ErrorObject},
	RpcModule,
};
use native_call_executor::NativeCall;
use parentchain_primitives::Nonce;
use parity_scale_codec::{Decode, Encode};
use std::{fmt::Debug, sync::Arc};
use tokio::sync::oneshot;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct AuthenticatedCall {
	pub call: NativeCall,
	pub nonce: Nonce,
	pub authentication: Authentication,
}

pub fn register_submit_aes_request(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("native_submitAesRequest", |params, ctx, _| async move {
			let Ok(hex_request) = params.one::<String>() else {
				return Err(ErrorCode::InvalidParams.into());
			};
			let Ok(mut request) = AesRequest::from_hex(&hex_request) else {
				return Err(ErrorCode::ServerError(INVALID_AES_REQUEST_CODE).into());
			};
			let native_call = get_native_call_from_aes_request(&mut request, ctx.clone()).await?;

			let (response_sender, response_receiver) = oneshot::channel();

			if ctx.native_call_sender.send((native_call, response_sender)).await.is_err() {
				log::error!("Failed to send request to native call executor");
				return Err(ErrorCode::InternalError.into());
			}
			match response_receiver.await {
				Ok(response) => Ok::<String, ErrorObject>(response.to_hex()),
				Err(e) => {
					log::error!("Failed to receive response from native call executor: {:?}", e);
					Err(ErrorCode::InternalError.into())
				},
			}
		})
		.expect("Failed to register native_submitAesRequest method");
}

async fn get_native_call_from_aes_request<'a>(
	request: &mut AesRequest,
	ctx: Arc<RpcContext>,
) -> Result<NativeCall, ErrorObject<'a>> {
	if request.shard().encode() != ctx.mrenclave.encode() {
		return Err(ErrorCode::ServerError(INVALID_SHARD_CODE).into());
	}

	let Ok(encoded_auth_call) = request.decrypt(Box::new(ctx.shielding_key.clone())) else {
		return Err(ErrorCode::ServerError(REQUEST_DECRYPTION_FAILED_CODE).into());
	};

	let auth_call: AuthenticatedCall =
		match AuthenticatedCall::decode(&mut encoded_auth_call.as_slice()) {
			Ok(auth_call) => auth_call,
			Err(e) => {
				log::error!("Failed to decode authenticated call: {:?}", e);
				return Err(ErrorCode::ServerError(INVALID_AUTHENTICATED_CALL_CODE).into());
			},
		};

	let authentication_result = match auth_call.authentication {
		Authentication::Web3(ref signature) => verify_web3_authentication(
			signature,
			&auth_call.call,
			auth_call.nonce,
			&ctx.mrenclave,
			&request.shard,
		),
		Authentication::Email(ref verification_code) => {
			verify_email_authentication(ctx, auth_call.call.sender_identity(), verification_code)
				.await
		},
		Authentication::AuthToken(ref auth_token) => {
			verify_auth_token_authentication(ctx, auth_call.call.sender_identity(), auth_token)
				.await
		},
	};

	if authentication_result.is_err() {
		return Err(ErrorCode::ServerError(AUTHENTICATION_FAILED_CODE).into());
	}

	Ok(auth_call.call)
}
