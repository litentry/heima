use crate::server::RpcContext;
use crate::ErrorCode;
use executor_storage::{Storage, VerificationCodeStorage};
use heima_authentication::web3::HeimaMessagePayload;
use heima_identity_verification::helpers::generate_otp;
use heima_primitives::{AccountId, Hashable};
use jsonrpsee::{types::ErrorObject, RpcModule};
use serde::Deserialize;
use std::str::FromStr;

#[derive(Deserialize)]
pub struct GetMessageCodeParams {
	pub omni_account: String,
}

pub fn register_get_message_code(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_getMessageCode", |params, ctx, _| async move {
			let params = params.parse::<GetMessageCodeParams>()?;
			let omni_account = AccountId::from_str(&params.omni_account).map_err(|_| {
				log::error!("Could not parse AccountId: {:?}", params.omni_account);
				ErrorCode::InvalidParams
			})?;
			let verification_code_storage = VerificationCodeStorage::new(ctx.storage_db.clone());
			let message_code = generate_otp(8);

			verification_code_storage
				.insert(&omni_account.hash(), message_code.clone())
				.map_err(|_| ErrorCode::InternalError)?;

			Ok::<HeimaMessagePayload, ErrorObject>(HeimaMessagePayload { message_code })
		})
		.expect("Failed to register omni_getMessageCode method");
}
