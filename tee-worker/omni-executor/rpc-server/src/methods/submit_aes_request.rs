use crate::{
	authentication::{
		verify_auth_token_authentication, verify_email_authentication,
		verify_oauth2_authentication, verify_web3_authentication, Authentication,
	},
	error_code::*,
	request::{AesRequest, DecryptableRequest},
	server::RpcContext,
	utils::hex::{FromHexPrefixed, ToHexPrefixed},
};
use executor_core::native_call::NativeCall;
use jsonrpsee::{
	types::{ErrorCode, ErrorObject},
	RpcModule,
};
use native_task_handler::NativeTask;
use parentchain_primitives::{Nonce, OmniAccountAuthType};
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
			let (native_call, auth_type) = handle_aes_request(&mut request, ctx.clone()).await?;
			let (response_sender, response_receiver) = oneshot::channel();
			let native_task = NativeTask { call: native_call, auth_type, response_sender };

			if ctx.native_task_sender.send(native_task).await.is_err() {
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

async fn handle_aes_request<'a>(
	request: &mut AesRequest,
	ctx: Arc<RpcContext>,
) -> Result<(NativeCall, OmniAccountAuthType), ErrorObject<'a>> {
	if request.shard().encode() != ctx.mrenclave.encode() {
		return Err(ErrorCode::ServerError(INVALID_SHARD_CODE).into());
	}
	let Ok(encoded_auth_call) = request.decrypt(Box::new(ctx.shielding_key.clone())) else {
		return Err(ErrorCode::ServerError(REQUEST_DECRYPTION_FAILED_CODE).into());
	};
	let authenticated_call: AuthenticatedCall =
		match AuthenticatedCall::decode(&mut encoded_auth_call.as_slice()) {
			Ok(auth_call) => auth_call,
			Err(e) => {
				log::error!("Failed to decode authenticated call: {:?}", e);
				return Err(ErrorCode::ServerError(INVALID_AUTHENTICATED_CALL_CODE).into());
			},
		};
	let authentication_result = match authenticated_call.authentication {
		Authentication::Web3(ref signature) => verify_web3_authentication(
			signature,
			&authenticated_call.call,
			authenticated_call.nonce,
			&ctx.mrenclave,
			&request.shard,
		),
		Authentication::Email(ref verification_code) => {
			verify_email_authentication(
				ctx,
				authenticated_call.call.sender_identity(),
				verification_code,
			)
			.await
		},
		Authentication::OAuth2(ref oauth2_data) => {
			verify_oauth2_authentication(
				ctx,
				authenticated_call.call.sender_identity().hash(),
				oauth2_data,
			)
			.await
		},
		Authentication::AuthToken(ref auth_token) => {
			verify_auth_token_authentication(
				ctx,
				authenticated_call.call.sender_identity(),
				auth_token,
			)
			.await
		},
	};

	if authentication_result.is_err() {
		return Err(ErrorCode::ServerError(AUTHENTICATION_FAILED_CODE).into());
	}

	Ok((authenticated_call.call, authenticated_call.authentication.into()))
}
