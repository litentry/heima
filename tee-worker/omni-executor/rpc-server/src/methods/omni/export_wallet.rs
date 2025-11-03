use crate::{
	detailed_error::DetailedError,
	error_code::{INTERNAL_ERROR_CODE, PARSE_ERROR_CODE, *},
	methods::omni::check_auth,
	server::RpcContext,
	utils::omni::to_omni_account,
	Deserialize,
};
use ::pumpx::signer_client::PumpxChainId as _;
use ethers::types::Bytes;
use executor_core::intent_executor::IntentExecutor;
use executor_core::native_task::*;
use executor_crypto::aes256::{aes_decrypt, aes_encrypt_default, Aes256Key, SerdeAesOutput};
use executor_primitives::PumpxAccountProfile;
use executor_storage::{HeimaJwtStorage, PumpxProfileStorage, Storage};
use heima_authentication::constants::AUTH_TOKEN_ACCESS_TYPE;
use jsonrpsee::RpcModule;
use rsa::Oaep;
use sha2::Sha256;
use signer_client::ChainType;
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct ExportWalletParams {
	pub key: Bytes, // RSA-encrypted AES key to encrypt the wallet private key, in 0x-hex-string
	pub google_code: String,
	pub chain_id: PumpxChainId,
	pub wallet_index: PumxWalletIndex,
	pub wallet_address: String,
}

pub fn register_export_wallet<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_exportWallet", |params, ctx, ext| async move {
			let oa_str = check_auth(&ext).map_err(|e| {
				error!("Authentication check failed: {:?}", e);
				DetailedError::new(
					AUTH_VERIFICATION_FAILED_CODE,
					"Authentication verification failed"
				).with_suggestion("Please check your authentication credentials").to_rpc_error()
			})?;

			let params = params.parse::<ExportWalletParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				DetailedError::new(
					PARSE_ERROR_CODE,
					"Parse error"
				).with_reason("Invalid JSON format or missing required fields").to_rpc_error()
			})?;

			debug!("Received omni_exportWallet, chain_id: {}, wallet_index: {}, expected_wallet_address: {}", params.chain_id, params.wallet_index, params.wallet_address);

			let omni_account = to_omni_account(&oa_str).map_err(|_| {
				DetailedError::new(
					PARSE_ERROR_CODE,
					"Failed to parse omni account",
				).to_rpc_error()
			})?;

			let aes_key = ctx
				.shielding_key
				.private_key()
				.decrypt(Oaep::new::<Sha256>(), &params.key)
				.map_err(|e| {
					error!("Failed to decrypt shielded value: {:?}", e);
					DetailedError::new(
						DECRYPT_REQUEST_FAILED_CODE,
						"Shielded value decryption failed"
					).with_field("key").with_reason("The provided RSA-encrypted AES key could not be decrypted").with_suggestion("Ensure the RSA public key matches the encryption key").to_rpc_error()
				})?;
			let aes_key: Aes256Key = aes_key.try_into().map_err(|_| {
				error!("Failed to convert AesKey");
				DetailedError::new(
					AES_KEY_CONVERT_FAILED_CODE,
					"AesKey convert failed"
				).with_field("key").with_reason("The decrypted key is not a valid 256-bit AES key").with_suggestion("Ensure the AES key is exactly 32 bytes (256 bits)").to_rpc_error()
			})?;

			// Inlined handler logic from handle_pumpx_export_wallet
			let storage = HeimaJwtStorage::new(ctx.storage_db.clone());
			let Ok(Some(access_token)) = storage.get(&(omni_account.clone(), AUTH_TOKEN_ACCESS_TYPE))
			else {
				error!("Failed to get pumpx_{}_jwt_token", AUTH_TOKEN_ACCESS_TYPE);
				return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason("Failed to get access token").to_rpc_error());
			};

			// Inline verify_google_code logic
			debug!("Calling pumpx verify_google_code, code: {}", params.google_code);
			let verify_result = ctx.pumpx_api.verify_google_code(&access_token, params.google_code, None).await;
			let verify_success = verify_result.map_or_else(
				|e| {
					error!("Google code verification request failed: {:?}", e);
					false
				},
				|res| {
					res.data.result.map_or_else(
						|| {
							error!("Google code verification response result is none");
							false
						},
						|success| success,
					)
				},
			);
			if !verify_success {
				error!("Failed to verify google code within NativeTask::PumpxExportWallet");
				return Err(DetailedError::new(
						PUMPX_API_GOOGLE_CODE_VERIFICATION_FAILED_CODE,
						"Google code verification failed",
					)
					.with_suggestion("Please check your Google verification code and try again").to_rpc_error());
			}

			let Some(chain) = ChainType::from_pumpx_chain_id(params.chain_id) else {
				error!("Failed to map pumpx chain_id {}", params.chain_id);
				return Err(DetailedError::new(INVALID_CHAIN_ID_CODE, "Chain not supported")
						.with_reason(format!("Chain ID {} is not supported", params.chain_id)).to_rpc_error()
				);
			};

			let Ok(mut wallet) = ctx
				.signer_client
				.export_wallet(
					chain,
					params.wallet_index,
					omni_account.clone().into(),
					ctx.aes256_key.to_vec(),
					params.wallet_address,
				)
				.await
			else {
				error!("Failed to export wallet from pumpx-signer");
				return Err(DetailedError::new(
						PUMPX_SIGNER_REQUEST_WALLET_FAILED_CODE,
						"Failed to export wallet from pumpx-signer",
					)
					.with_suggestion("Please try again").to_rpc_error());
			};
			let Some(decrypted_wallet) = aes_decrypt(&ctx.aes256_key, &mut wallet) else {
				error!("Failed to decrypt wallet");
				return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason("Failed to decrypt wallet").to_rpc_error());
			};

			let omni_account_profile_storage = PumpxProfileStorage::new(ctx.storage_db.clone());
			if let Ok(maybe_profile) = omni_account_profile_storage.get(&omni_account) {
				let profile = maybe_profile
					.map(|mut p| {
						p.wallet_exported = true;
						p
					})
					.unwrap_or_else(|| PumpxAccountProfile { wallet_exported: true });
				if let Err(e) = omni_account_profile_storage.insert(&omni_account, profile) {
					error!("Failed to update pumpx account profile: {:?}", e);
					return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
							.with_reason("Failed to update pumpx account profile").to_rpc_error());
				};
			} else {
				error!("Failed to get pumpx account profile");
				return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason("Failed to get pumpx account profile").to_rpc_error());
			}

			let encrypted_wallet: SerdeAesOutput =
				aes_encrypt_default(&aes_key, &decrypted_wallet).into();
			Ok(encrypted_wallet)
		})
		.expect("Failed to register omni_exportWallet method");
}
