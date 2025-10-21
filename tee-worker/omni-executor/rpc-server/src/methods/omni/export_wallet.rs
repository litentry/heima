use crate::{
	detailed_error::DetailedError,
	error_code::{INTERNAL_ERROR_CODE, PARSE_ERROR_CODE, *},
	methods::omni::{common::check_auth, PumpxRpcError},
	server::RpcContext,
	Deserialize,
};
use ethers::types::Bytes;
use executor_core::intent_executor::IntentExecutor;
use executor_core::native_task::*;
use executor_crypto::aes256::{aes_encrypt_default, Aes256Key, SerdeAesOutput};
use executor_primitives::{utils::hex::FromHexPrefixed, AccountId};
use heima_primitives::Address32;
use jsonrpsee::RpcModule;
use native_task_handler::{handle_pumpx_export_wallet, NativeTaskError, NativeTaskOk};
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

pub fn register_export_wallet<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>,
	>,
) {
	module
		.register_async_method("omni_exportWallet", |params, ctx, ext| async move {
			let user = check_auth(&ext).map_err(|e| {
				error!("Authentication check failed: {:?}", e);
				PumpxRpcError::from(DetailedError::new(
					AUTH_VERIFICATION_FAILED_CODE,
					"Authentication verification failed"
				).with_suggestion("Please check your authentication credentials"))
			})?;

			let params = params.parse::<ExportWalletParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(DetailedError::new(
					PARSE_ERROR_CODE,
					"Parse error"
				).with_reason("Invalid JSON format or missing required fields"))
			})?;

			debug!("Received omni_exportWallet, chain_id: {}, wallet_index: {}, expected_wallet_address: {}", params.chain_id, params.wallet_index, params.wallet_address);

			let Ok(address) = Address32::from_hex(&user.omni_account) else {
				error!("Failed to parse from omni account token");
				return Err(PumpxRpcError::from(DetailedError::new(
					INTERNAL_ERROR_CODE,
					"Internal error"
				).with_reason("Failed to parse omni account from authentication token")));
			};
			let omni_account = AccountId::from(address);

			let aes_key = ctx
				.shielding_key
				.private_key()
				.decrypt(Oaep::new::<Sha256>(), &params.key)
				.map_err(|e| {
					error!("Failed to decrypt shielded value: {:?}", e);
					PumpxRpcError::from(DetailedError::new(
						DECRYPT_REQUEST_FAILED_CODE,
						"Shielded value decryption failed"
					).with_field("key").with_reason("The provided RSA-encrypted AES key could not be decrypted").with_suggestion("Ensure the RSA public key matches the encryption key"))
				})?;
			let aes_key: Aes256Key = aes_key.try_into().map_err(|_| {
				error!("Failed to convert AesKey");
				PumpxRpcError::from(DetailedError::new(
					AES_KEY_CONVERT_FAILED_CODE,
					"AesKey convert failed"
				).with_field("key").with_reason("The decrypted key is not a valid 256-bit AES key").with_suggestion("Ensure the AES key is exactly 32 bytes (256 bits)"))
			})?;

			let result = handle_pumpx_export_wallet(
				ctx.to_task_handler_context(),
				omni_account,
				params.google_code,
				params.chain_id,
				params.wallet_index,
				params.wallet_address,
				user.client_id,
			)
			.await;

			match result {
				Ok(NativeTaskOk::PumpxExportWallet(wallet)) => {
					let encrypted_wallet: SerdeAesOutput =
						aes_encrypt_default(&aes_key, &wallet).into();
					Ok(encrypted_wallet)
				},
				Ok(_) => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from(DetailedError::new(
						INTERNAL_ERROR_CODE,
						"Internal error"
					).with_reason("Unexpected response type from native task handler")))
				},
				Err(NativeTaskError::PumpxApiError(pumpx_error)) => {
					error!("Pumpx API error: {:?}", pumpx_error);
					match pumpx_error {
						native_task_handler::PumpxApiError::GoogleCodeVerificationFailed => {
							Err(PumpxRpcError::from(DetailedError::new(
								PUMPX_API_GOOGLE_CODE_VERIFICATION_FAILED_CODE,
								"Google code verification failed"
							).with_suggestion("Please check your Google verification code and try again")))
						},
						_ => {
							Err(PumpxRpcError::from(DetailedError::new(
								INTERNAL_ERROR_CODE,
								"Pumpx API error"
							).with_reason(format!("{:?}", pumpx_error))))
						}
					}
				},
				Err(NativeTaskError::ChainNotSupported(chain_id)) => {
					error!("Chain not supported: {}", chain_id);
					Err(PumpxRpcError::from(DetailedError::new(
						INVALID_CHAIN_ID_CODE,
						"Chain not supported"
					).with_reason(format!("Chain ID {} is not supported", chain_id))))
				},
				Err(NativeTaskError::PumpxSignerError(signer_error)) => {
					error!("Pumpx signer error: {:?}", signer_error);
					Err(PumpxRpcError::from(DetailedError::new(
						PUMPX_SIGNER_REQUEST_WALLET_FAILED_CODE,
						"Wallet export failed"
					).with_reason(format!("{:?}", signer_error))))
				},
				Err(e) => {
					error!("Failed to export wallet: {:?}", e);
					Err(PumpxRpcError::from(DetailedError::new(
						INTERNAL_ERROR_CODE,
						"Failed to export wallet"
					).with_reason(format!("{:?}", e))))
				},
			}
		})
		.expect("Failed to register omni_exportWallet method");
}
