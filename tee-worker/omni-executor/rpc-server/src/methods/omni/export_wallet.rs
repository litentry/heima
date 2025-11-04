use crate::{
	detailed_error::DetailedError, server::RpcContext, utils::omni::extract_omni_account,
	utils::validation::parse_rpc_params, Deserialize,
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
			debug!("Received omni_exportWallet, params: {:?}", params);

			let params = parse_rpc_params::<ExportWalletParams>(params)?;
			let omni_account = extract_omni_account(&ext)?;

			let aes_key = ctx
				.shielding_key
				.private_key()
				.decrypt(Oaep::new::<Sha256>(), &params.key)
				.map_err(|e| {
					let msg = format!("Failed to decrypt shielded value: {:?}", e);
					error!(msg);
					DetailedError::internal_error(&msg).to_rpc_error()
				})?;

			let aes_key: Aes256Key = aes_key.try_into().map_err(|_| {
				let msg = "Failed to convert AesKey".to_string();
				error!(msg);
				DetailedError::internal_error(&msg).to_rpc_error()
			})?;

			// Inlined handler logic from handle_pumpx_export_wallet
			let storage = HeimaJwtStorage::new(ctx.storage_db.clone());
			let Ok(Some(access_token)) =
				storage.get(&(omni_account.clone(), AUTH_TOKEN_ACCESS_TYPE))
			else {
				error!("Failed to get {}_jwt_token", AUTH_TOKEN_ACCESS_TYPE);
				return Err(DetailedError::storage_service_error("get access token").to_rpc_error());
			};

			// Inline verify_google_code logic
			debug!("Calling pumpx verify_google_code, code: {}", params.google_code);
			let verify_result =
				ctx.pumpx_api.verify_google_code(&access_token, params.google_code, None).await;
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
				let msg = "Failed to verify google code within ExportWallet";
				error!(msg);
				return Err(DetailedError::internal_error(msg).to_rpc_error());
			}

			let Some(chain) = ChainType::from_pumpx_chain_id(params.chain_id) else {
				error!("Failed to map pumpx chain_id {}", params.chain_id);
				return Err(DetailedError::invalid_chain_id(params.chain_id.into()).to_rpc_error());
			};

			let mut wallet = ctx
				.signer_client
				.export_wallet(
					chain,
					params.wallet_index,
					omni_account.clone().into(),
					ctx.aes256_key.to_vec(),
					params.wallet_address,
				)
				.await
				.map_err(|_| DetailedError::signer_service_error().to_rpc_error())?;

			let Some(decrypted_wallet) = aes_decrypt(&ctx.aes256_key, &mut wallet) else {
				let msg = "Failed to decrypt wallet";
				error!(msg);
				return Err(DetailedError::internal_error(msg).to_rpc_error());
			};

			let omni_account_profile_storage = PumpxProfileStorage::new(ctx.storage_db.clone());
			if let Ok(maybe_profile) = omni_account_profile_storage.get(&omni_account) {
				let profile = maybe_profile
					.map(|mut p| {
						p.wallet_exported = true;
						p
					})
					.unwrap_or_else(|| PumpxAccountProfile { wallet_exported: true });
				omni_account_profile_storage.insert(&omni_account, profile).map_err(|e| {
					error!("Failed to update pumpx account profile: {:?}", e);
					DetailedError::storage_service_error("insert account profile").to_rpc_error()
				})?;
			} else {
				error!("Failed to get pumpx account profile");
				return Err(
					DetailedError::storage_service_error("get account profile").to_rpc_error()
				);
			}

			let encrypted_wallet: SerdeAesOutput =
				aes_encrypt_default(&aes_key, &decrypted_wallet).into();
			Ok(encrypted_wallet)
		})
		.expect("Failed to register omni_exportWallet method");
}
