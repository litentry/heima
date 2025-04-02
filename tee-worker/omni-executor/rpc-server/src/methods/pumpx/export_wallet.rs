use crate::{
	error_code::*, hex_encode, oneshot, serde_as, server::RpcContext, verify_auth::verify_auth,
	Deserialize, Encode, ErrorCode, Serialize,
};
use executor_core::native_task::*;
use executor_crypto::aes256::{aes_encrypt_default, Aes256Key};
use executor_primitives::OmniAuth;
use heima_primitives::{Identity, Web2IdentityType};
use jsonrpsee::{types::ErrorObject, RpcModule};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use rsa::Oaep;
use sha2::Sha256;

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportWalletParams {
	pub user_email: String,
	#[serde_as(as = "serde_with::hex::Hex")]
	pub key: Vec<u8>, // RSA-encrypted AES key to encrypt the wallet private key
	pub google_code: MaybeGoogleCode,
	pub wallet_chain: PumpxWalletChain,
	pub wallet_index: PumxWalletIndex,
	pub wallet_address: ExpectedWalletAddress,
	pub email_code: String,
}

impl From<ExportWalletParams> for NativeTaskWrapper<NativeTask> {
	fn from(p: ExportWalletParams) -> Self {
		Self {
			task: NativeTask::PumpxExportWallet(
				Identity::from_web2_account(p.user_email.as_str(), Web2IdentityType::Email),
				p.google_code,
				p.wallet_chain,
				p.wallet_index,
				p.wallet_address,
			),
			nonce: None,
			auth: Some(OmniAuth::Email(p.email_code)),
		}
	}
}

pub fn register_export_wallet<
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<Header, RpcClient, RpcClientFactory>>,
) {
	module
		.register_async_method("pumpx_exportWallet", |params, ctx, _| async move {
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
				ErrorObject::owned::<()>(
					DECRYPT_REQUEST_FAILED_CODE,
					"Shielded value decryption failed",
					None,
				)
			})?;

			let wrapper: NativeTaskWrapper<NativeTask> = params.into();

			if wrapper.task.require_auth() && verify_auth(ctx.clone(), &wrapper).await.is_err() {
				return Err(ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE).into());
			}

			let (response_sender, response_receiver) = oneshot::channel();

			if ctx.native_task_sender.send((wrapper, response_sender)).await.is_err() {
				log::error!("Failed to send request to native call executor");
				return Err(ErrorCode::InternalError.into());
			}
			match response_receiver.await {
				// TODO: we should use json field for better readability, either a defined structure, or serde_json::Value
				Ok(response) => {
					// TODO: maybe not encode? we want to dismiss SCALE
					let response = aes_encrypt_default(&aes_key, &response).encode();
					Ok::<String, ErrorObject>(hex_encode(response.as_slice()))
				},
				Err(e) => {
					log::error!("Failed to receive response from native call handler: {:?}", e);
					Err(ErrorCode::InternalError.into())
				},
			}
		})
		.expect("Failed to register pumpx_requestJwt method");
}
