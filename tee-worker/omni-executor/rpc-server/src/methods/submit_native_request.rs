use crate::{
	crypto::{
		aes256::{aes_decrypt, Aes256Key, AesOutput},
		traits::ShieldingCryptoDecrypt,
	},
	server::RpcContext,
	utils::hex::FromHexPrefixed,
};
use jsonrpsee::{types::Params, RpcModule};
use parity_scale_codec::{Decode, Encode};
use std::{fmt::Debug, sync::Arc, vec::Vec};

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
		shielding_key: Arc<impl ShieldingCryptoDecrypt<Error = T>>,
	) -> Result<Vec<u8>, ()> {
		let aes_key: Aes256Key =
			shielding_key.decrypt(&self.key).map_err(|_| ())?.try_into().map_err(|_| ())?;

		aes_decrypt(&aes_key, &mut self.payload).ok_or(())
	}
}

pub fn register_submit_native_request(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("submitNativeRequest", |params, ctx, _| async move {
			let payload = get_request_payload(params).unwrap(); // TODO: handle error
			let mut request: AesRequest = AesRequest::from_hex(&payload).unwrap();

			// TODO: check mrenclave

			let encoded_task = request.decrypt(ctx.shielding_key.clone()).unwrap();

			// TODO:
			// check authentication if needed

			// send task or proccess it ???
		})
		.expect("Failed to register submitNativeRequest method");
}

fn get_request_payload(params: Params) -> Result<String, String> {
	let s_vec = params.parse::<Vec<String>>().map_err(|e| format!("{}", e))?;
	let s = s_vec.first().ok_or_else(|| "Empty params".to_string())?;
	log::debug!("Request payload: {}", s);
	Ok(s.to_owned())
}
