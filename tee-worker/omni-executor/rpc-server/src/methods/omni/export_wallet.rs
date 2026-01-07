use crate::{
	detailed_error::DetailedError, server::RpcContext, utils::omni::extract_omni_account,
	utils::types::RpcResultExt, utils::validation::parse_rpc_params, Deserialize,
};
use ::oe_client_pumpx::signer_client::PumpxChainId as _;
use ethers::types::Bytes;
use jsonrpsee::RpcModule;
use oe_client_signer::ChainType;
use oe_core::auth::constants::AUTH_TOKEN_ACCESS_TYPE;
use oe_core::intent::executor::IntentExecutor;
use oe_core::native_task::*;
use oe_crypto::aes256::{aes_decrypt, aes_encrypt_default, Aes256Key, SerdeAesOutput};
use oe_primitives::PumpxAccountProfile;
use oe_storage::{HeimaJwtStorage, PumpxProfileStorage, Storage};
use rsa::Oaep;
use sha2::Sha256;
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
				.map_err_internal("Failed to decrypt shielded value")?;

			let aes_key: Aes256Key =
				aes_key.try_into().map_err_internal("Failed to convert AesKey")?;

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

			let decrypted_wallet = aes_decrypt(&ctx.aes256_key, &mut wallet)
				.ok_or(DetailedError::internal_error("Failed to decrypt wallet").to_rpc_error())?;

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
