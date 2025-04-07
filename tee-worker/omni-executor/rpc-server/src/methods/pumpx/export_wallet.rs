use crate::{
	error_code::*, oneshot, serde_as, server::RpcContext, verify_auth::verify_auth, Decode,
	Deserialize, ErrorCode, Serialize,
};
use executor_core::native_task::*;
use executor_crypto::aes256::{aes_encrypt_default, Aes256Key, AesOutput};
use executor_primitives::OmniAuth;
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::{types::ErrorObject, RpcModule};
use native_task_handler::{NativeTaskError, NativeTaskOk, NativeTaskResponse};
use rsa::Oaep;
use sha2::Sha256;

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct ExportWalletParams {
	pub user_email: String,
	#[serde_as(as = "serde_with::hex::Hex")]
	pub key: Vec<u8>, // RSA-encrypted AES key to encrypt the wallet private key
	pub google_code: MaybeGoogleCode,
	pub chain_id: PumpxChainId,
	pub wallet_index: PumxWalletIndex,
	pub wallet_address: String,
	pub email_code: String,
}

#[derive(Serialize, Clone)]
pub struct ExportWalletResponse {
	pub encrypted_wallet: AesOutput,
}

impl From<ExportWalletParams> for NativeTaskWrapper<NativeTask> {
	fn from(p: ExportWalletParams) -> Self {
		Self {
			task: NativeTask::PumpxExportWallet(
				Identity::from_web2_account(p.user_email.as_str(), Web2IdentityType::Email),
				p.google_code,
				p.chain_id,
				p.wallet_index,
				p.wallet_address,
			),
			nonce: None,
			auth: Some(OmniAuth::Email(p.email_code)),
		}
	}
}

pub fn register_export_wallet(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("pumpx_exportWallet", |params, ctx, _| async move {
			let internal_error: ErrorObject = ErrorCode::InternalError.into();
			let params = params.parse::<ExportWalletParams>()?;
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
							let encrypted_wallet = aes_encrypt_default(&aes_key, &wallet);
							Ok(ExportWalletResponse { encrypted_wallet })
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
