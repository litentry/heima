use crate::{
	error_code::*,
	methods::pumpx::{common::check_and_get_option_response_data, PumpxRpcError},
	server::RpcContext,
	verify_auth::verify_auth,
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

use super::common::handle_pumpx_native_task;

#[derive(Debug, Deserialize)]
pub struct ExportWalletParams {
	pub user_id: String,
	pub user_email: String,
	pub client_id: String,
	pub key: Bytes, // RSA-encrypted AES key to encrypt the wallet private key, in 0x-hex-string
	pub google_code: String,
	pub chain_id: PumpxChainId,
	pub wallet_index: PumxWalletIndex,
	pub wallet_address: String,
	pub email_code: String,
}

impl From<ExportWalletParams> for NativeTaskWrapper<NativeTask> {
	fn from(p: ExportWalletParams) -> Self {
		NativeTaskWrapper::new(
			NativeTask::PumpxExportWallet(
				Identity::from_web2_account(p.user_id.as_str(), Web2IdentityType::Pumpx),
				p.google_code,
				p.chain_id,
				p.wallet_index,
				p.wallet_address,
			),
			None,
			Some(OmniAuth::Email(p.client_id.clone(), p.user_email, p.email_code)),
		)
	}
}

pub fn register_export_wallet(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("pumpx_exportWallet", |params, ctx, _| async move {
			let params = params.parse::<ExportWalletParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!("Received pumpx_exportWallet, user_id: {}, chain_id: {}, wallet_index: {}, expected_wallet_address: {}", params.user_id, params.chain_id, params.wallet_index, params.wallet_address);

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

			// verify user_id and user_email matches
			debug!("Calling pumpx get_account_user_id, email: {}", params.user_email);
			let Ok(res) = ctx.pumpx_api.get_account_user_id(params.user_email.clone()).await else {
				error!("Failed to call get_account_user_id");
				return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
					PUMPX_API_GET_ACCOUNT_USER_ID_FAILED_CODE,
				)));
			};
			debug!("Response pumpx get_account_user_id: {:?}", res);

			let user_id = check_and_get_option_response_data(res.data.user_id, PUMPX_API_GET_ACCOUNT_USER_ID_FAILED_CODE, "Response data.user_id of call get_account_user_id is none")?;

			if user_id != params.user_id {
				error!(
					"Parameter mismatch: user_id {} and user_email {}, expected user_id {}",
					params.user_id,
					params.user_email,
					user_id
				);
				return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
					USER_EMAIL_ID_MISMATCH_CODE,
				)));
			}

			let wrapper: NativeTaskWrapper<NativeTask> = params.into();

           	if wrapper.task.require_auth() {
				let Some(ref auth) = wrapper.auth else {
					error!("Missing auth");
					return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
						REQUIRE_AUTHENTICATION_CODE,
					)));
				};
				verify_auth(ctx.clone(), auth).await.map_err(|_| {
					error!("Failed to verify auth: {:?}", wrapper.auth);
					PumpxRpcError::from_error_code(ErrorCode::ServerError(
						AUTH_VERIFICATION_FAILED_CODE,
					))
				})?;
			}

			handle_pumpx_native_task(&ctx, wrapper, |task_ok| match task_ok {
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
		.expect("Failed to register pumpx_exportWallet method");
}
