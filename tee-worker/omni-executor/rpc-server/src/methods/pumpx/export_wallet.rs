use crate::{
	error_code::*, oneshot, server::RpcContext, verify_auth::verify_auth, Decode, Deserialize,
	ErrorCode,
};
use ethers::types::Bytes;
use executor_core::native_task::*;
use executor_crypto::aes256::{aes_encrypt_default, Aes256Key, SerdeAesOutput};
use executor_primitives::OmniAuth;
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::{types::ErrorObject, RpcModule};
use native_task_handler::{NativeTaskError, NativeTaskOk, NativeTaskResponse};
use rsa::Oaep;
use sha2::Sha256;

#[derive(Debug, Deserialize)]
pub struct ExportWalletParams {
	pub user_id: String,
	pub user_email: String,
	pub key: Bytes, // RSA-encrypted AES key to encrypt the wallet private key, in 0x-hex-string
	pub google_code: String,
	pub chain_id: PumpxChainId,
	pub wallet_index: PumxWalletIndex,
	pub wallet_address: String,
	pub email_code: String,
}

impl From<ExportWalletParams> for NativeTaskWrapper<NativeTask> {
	fn from(p: ExportWalletParams) -> Self {
		Self {
			task: NativeTask::PumpxExportWallet(
				Identity::from_web2_account(p.user_id.as_str(), Web2IdentityType::Pumpx),
				p.google_code,
				p.chain_id,
				p.wallet_index,
				p.wallet_address,
			),
			nonce: None,
			auth: Some(OmniAuth::Email(p.user_email, p.email_code)),
		}
	}
}

pub fn register_export_wallet(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("pumpx_exportWallet", |params, ctx, _| async move {
			let internal_error: ErrorObject = ErrorCode::InternalError.into();
			let params = params.parse::<ExportWalletParams>()?;

			log::debug!("Received pumpx_exportWallet, user_id: {}, chain_id: {}, wallet_index: {}, expected_wallet_address: {}", params.user_id, params.chain_id, params.wallet_index, params.wallet_address);

			let aes_key = ctx
				.shielding_key
				.private_key()
				.decrypt(Oaep::new::<Sha256>(), &params.key)
				.map_err(|_| {
					ErrorObject::owned::<()>(
						DECRYPT_REQUEST_FAILED_CODE,
						"Shielded value decryption failed",
						None,
					)
				})?;
			let aes_key: Aes256Key = aes_key.try_into().map_err(|_| {
				ErrorObject::owned::<()>(AES_KEY_CONVERT_FAILED_CODE, "AesKey convert failed", None)
			})?;

			// verify user_id and user_email matches
			log::debug!("Calling pumpx get_account_user_id, email: {}", params.user_email);
			let Ok(res) = ctx.pumpx_api.get_account_user_id(params.user_email.clone()).await else {
				log::error!("Failed to call get_account_user_id");
				return Err(
					ErrorCode::ServerError(PUMPX_API_GET_ACCOUNT_USER_ID_FAILED_CODE).into()
				);
			};
			log::debug!("Response pumpx get_account_user_id: {:?}", res);

			if res.data.user_id != params.user_id {
				log::error!(
					"Parameter mismatch: user_id {} and user_email {}, expected user_id {}",
					params.user_id,
					params.user_email,
					res.data.user_id
				);
				return Err(ErrorCode::ServerError(USER_EMAIL_ID_MISMATCH_CODE).into());
			}

			let wrapper: NativeTaskWrapper<NativeTask> = params.into();

			if wrapper.task.require_auth() && verify_auth(ctx.clone(), &wrapper).await.is_err() {
				return Err(ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE).into());
			}

			let (response_sender, response_receiver) = oneshot::channel();

			if ctx.native_task_sender.send((wrapper, response_sender)).await.is_err() {
				log::error!("Failed to send request to native call executor");
				return Err(internal_error);
			}

			match response_receiver.await {
				Ok(response) => {
					let native_task_response: NativeTaskResponse =
						Decode::decode(&mut response.as_slice())
							.map_err(|_| internal_error.clone())?;
					match native_task_response {
						Ok(NativeTaskOk::PumpxExportWallet(wallet)) => {
							let encrypted_wallet: SerdeAesOutput =
								aes_encrypt_default(&aes_key, &wallet).into();
							Ok(encrypted_wallet)
						},
						Err(NativeTaskError::InternalError) => {
							log::error!("Internal error in native task");
							Err(internal_error)
						},
						Err(native_task_error) => {
							log::error!("Native task error: {:?}", native_task_error);
							Err(ErrorCode::ServerError(get_native_task_error_code(
								&native_task_error,
							))
							.into())
						},
						_ => {
							log::error!("Unexpected response type");
							Err(internal_error)
						},
					}
				},
				Err(e) => {
					log::error!("Failed to receive response from native call handler: {:?}", e);
					Err(internal_error)
				},
			}
		})
		.expect("Failed to register pumpx_exportWallet method");
}
