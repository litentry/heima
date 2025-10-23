use crate::server::RpcContext;
use crate::ErrorCode;
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::utils::hex::hex_encode;
use executor_storage::{Storage, VerificationCodeStorage};
use heima_authentication::web3::HeimaMessagePayload;
use heima_identity_verification::helpers::generate_otp;
use heima_primitives::{AccountId, Hashable};
use jsonrpsee::{types::ErrorObject, RpcModule};
use serde::Deserialize;
use std::str::FromStr;
use tracing::error;

#[derive(Deserialize)]
pub struct GetWeb3SignInMessageParams {
	pub client_id: String,
	pub omni_account: String,
}

pub fn register_get_web3_sign_in_message<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_getWeb3SignInMessage", |params, ctx, _| async move {
			let params = params.parse::<GetWeb3SignInMessageParams>()?;
			let omni_account = AccountId::from_str(&params.omni_account).map_err(|_| {
				error!("Could not parse AccountId: {:?}", params.omni_account);
				ErrorCode::InvalidParams
			})?;
			let verification_code_storage = VerificationCodeStorage::new(ctx.storage_db.clone());
			let storage_key = omni_account.hash();
			let message_code = match verification_code_storage.get(&storage_key) {
				Ok(Some(message_code)) => message_code,
				Ok(None) => {
					let message_code = generate_otp(8);
					verification_code_storage
						.insert(&storage_key, message_code.clone())
						.map_err(|_| ErrorCode::InternalError)?;
					message_code
				},
				Err(_) => return Err(ErrorCode::InternalError.into()),
			};

			Ok::<HeimaMessagePayload, ErrorObject>(HeimaMessagePayload {
				message_code,
				omni_account: hex_encode(omni_account.as_ref()),
				client_id: params.client_id,
			})
		})
		.expect("Failed to register omni_getWeb3SignInMessage method");
}
