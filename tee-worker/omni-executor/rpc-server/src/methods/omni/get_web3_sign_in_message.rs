use crate::server::RpcContext;
use crate::ErrorCode;
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::utils::hex::ToHexPrefixed;
use executor_storage::{Storage, VerificationCodeStorage};
use heima_authentication::web3::HeimaMessagePayload;
use heima_identity_verification::helpers::generate_otp;
use heima_primitives::{AccountId, Hashable};
use jsonrpsee::{types::ErrorObjectOwned, RpcModule};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, error};

#[derive(Deserialize)]
pub struct GetWeb3SignInMessageParams {
	pub client_id: String,
	pub omni_account: String,
}

#[tracing::instrument(skip(params, ctx), fields(
	client_id = %params.client_id,
	omni_account = %params.omni_account
))]
async fn handle_get_web3_sign_in_message_request<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	params: GetWeb3SignInMessageParams,
	ctx: Arc<RpcContext<
		Header,
		RpcClient,
		RpcClientFactory,
		EthereumIntentExecutor,
		SolanaIntentExecutor,
		CrossChainIntentExecutor,
	>>,
) -> Result<HeimaMessagePayload, ErrorObjectOwned> {
	debug!("Processing omni_getWeb3SignInMessage request");

	let omni_account = AccountId::from_str(&params.omni_account).map_err(|_| {
		error!("Could not parse AccountId: {:?}", params.omni_account);
		<ErrorCode as Into<ErrorObjectOwned>>::into(ErrorCode::InvalidParams)
	})?;
	
	let verification_code_storage = VerificationCodeStorage::new(ctx.storage_db.clone());
	let storage_key = omni_account.hash();
	let message_code = match verification_code_storage.get(&storage_key) {
		Ok(Some(message_code)) => message_code,
		Ok(None) => {
			let message_code = generate_otp(8);
			verification_code_storage
				.insert(&storage_key, message_code.clone())
				.map_err(|_| <ErrorCode as Into<ErrorObjectOwned>>::into(ErrorCode::InternalError))?;
			message_code
		},
		Err(_) => return Err(<ErrorCode as Into<ErrorObjectOwned>>::into(ErrorCode::InternalError)),
	};

	Ok(HeimaMessagePayload {
		message_code,
		omni_account: omni_account.to_hex(),
		client_id: params.client_id,
	})
}

pub fn register_get_web3_sign_in_message<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<
			Header,
			RpcClient,
			RpcClientFactory,
			EthereumIntentExecutor,
			SolanaIntentExecutor,
			CrossChainIntentExecutor,
		>,
	>,
) {
	module
		.register_async_method("omni_getWeb3SignInMessage", |params, ctx, _| async move {
			let params = params.parse::<GetWeb3SignInMessageParams>()?;
			handle_get_web3_sign_in_message_request(params, ctx).await
		})
		.expect("Failed to register omni_getWeb3SignInMessage method");
}
