use crate::{
	error_code::*,
	server::RpcContext,
	utils::hex::{FromHexPrefixed, ToHexPrefixed},
};
use crypto::{
	aes256::{aes_decrypt, Aes256Key, AesOutput},
	traits::Decrypt,
};
use jsonrpsee::{
	types::{ErrorCode, ErrorObject},
	RpcModule,
};
use native_call_executor::NativeCall;
use parity_scale_codec::{Decode, Encode};
use std::{fmt::Debug, sync::Arc, vec::Vec};
use tokio::sync::oneshot;

type MrEnclave = [u8; 32];

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct AesRequest {
	pub mrenclave: MrEnclave,
	pub key: Vec<u8>,
	pub payload: AesOutput,
}

impl AesRequest {
	fn decrypt<T: Debug>(
		&mut self,
		shielding_key: Arc<impl Decrypt<Error = T>>,
	) -> Result<Vec<u8>, ()> {
		let aes_key: Aes256Key =
			shielding_key.decrypt(&self.key).map_err(|_| ())?.try_into().map_err(|_| ())?;

		aes_decrypt(&aes_key, &mut self.payload).ok_or(())
	}
}

pub type VerificationCode = String;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum Authentication {
	Email(VerificationCode),
	AuthToken(String),
	// OAuth2(OAuth2Data),
	// Web3(LitentryMultiSignature),
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct AuthenticatedCall {
	pub call: NativeCall,
	pub nonce: u32,
	pub authentication: Authentication,
}

pub fn register_native_submit_aes_request(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("native_submitAesRequest", |params, ctx, _| async move {
			let Ok(hex_request) = params.one::<String>() else {
				return Err(ErrorCode::InvalidParams.into());
			};
			let Ok(mut request) = AesRequest::from_hex(&hex_request) else {
				return Err(ErrorCode::ServerError(INVALID_AES_REQUEST_CODE).into());
			};
			let native_call = get_native_call_from_aes_request(&mut request, ctx.clone())?;

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
		.expect("Failed to register submitNativeRequest method");
}

fn get_native_call_from_aes_request<'a>(
	request: &mut AesRequest,
	ctx: Arc<RpcContext>,
) -> Result<NativeCall, ErrorObject<'a>> {
	if request.mrenclave != ctx.mrenclave {
		log::error!("Invalid mrenclave");
		return Err(ErrorCode::ServerError(INVALID_MRENCLAVE_CODE).into());
	}

	let Ok(encoded_auth_call) = request.decrypt(ctx.shielding_key.clone()) else {
		return Err(ErrorCode::ServerError(REQUEST_DECRYPTION_FAILED_CODE).into());
	};

	let Ok(auth_call) = AuthenticatedCall::decode(&mut encoded_auth_call.as_slice()) else {
		return Err(ErrorCode::ServerError(INVALID_AUTHENTICATED_CALL_CODE).into());
	};

	let authentication_result: Result<(), &str> = match auth_call.authentication {
		// TODO:
		Authentication::Email(ref _verification_code) => {
			// Verify code
			Ok(())
		},
		Authentication::AuthToken(ref _token) => {
			// Verify token
			Ok(())
		},
	};

	if authentication_result.is_err() {
		return Err(ErrorCode::ServerError(AUTHENTICATION_FAILED_CODE).into());
	}

	Ok(auth_call.call)
}
