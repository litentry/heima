use crate::{
	error_code::*, methods::omni::PumpxRpcError, server::RpcContext, verify_auth::verify_auth,
	Deserialize, ErrorCode,
};
use ethers::types::Bytes;
use executor_core::native_task::*;
use executor_crypto::aes256::{aes_encrypt_default, Aes256Key, SerdeAesOutput};
use executor_primitives::OmniAuth;
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use rsa::Oaep;
use sha2::Sha256;
use tracing::{debug, error};

use super::common::handle_omni_native_task;

#[derive(Debug, Deserialize)]
pub struct ExportWalletParams {
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
				Identity::from_web2_account(p.user_email.as_str(), Web2IdentityType::Pumpx),
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
		.register_async_method("omni_exportWallet", |params, ctx, _| async move {
			let params = params.parse::<ExportWalletParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!("Received omni_exportWallet, user_email: {}, chain_id: {}, wallet_index: {}, expected_wallet_address: {}", params.user_email, params.chain_id, params.wallet_index, params.wallet_address);

			let aes_key = ctx
				.shielding_key
				.private_key()
				.decrypt(Oaep::new::<Sha256>(), &params.key)
				.map_err(|e| {
					error!("Failed to decrypt shielded value: {:?}", e);
					PumpxRpcError::from_code_and_message(
						DECRYPT_REQUEST_FAILED_CODE,
						"Shielded value decryption failed".into(),
					)
				})?;
			let aes_key: Aes256Key = aes_key.try_into().map_err(|_| {
				error!("Failed to convert AesKey");
				PumpxRpcError::from_code_and_message(
					AES_KEY_CONVERT_FAILED_CODE,
					"AesKey convert failed".into(),
				)
			})?;

			let wrapper: NativeTaskWrapper<NativeTask> = params.into();

			if wrapper.task.require_auth() && verify_auth(ctx.clone(), &wrapper).await.is_err() {
				return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
					AUTH_VERIFICATION_FAILED_CODE,
				)));
			}

			handle_omni_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::PumpxExportWallet(wallet) => {
					let encrypted_wallet: SerdeAesOutput =
						aes_encrypt_default(&aes_key, &wallet).into();
					Ok(encrypted_wallet)
				},
				_ => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register omni_exportWallet method");
}
