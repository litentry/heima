use crate::server::RpcContext;
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::{utils::hex::ToHexPrefixed, Identity, Web2IdentityType};
use executor_storage::{OAuth2StateVerifierStorage, Storage};
use heima_identity_verification::web2::google;
use jsonrpsee::{
	types::{ErrorCode, ErrorObjectOwned},
	RpcModule,
};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use std::sync::Arc;
use tracing::{debug, error};

#[tracing::instrument(skip(ctx), fields(
	google_account = %google_account,
	redirect_uri = %redirect_uri
))]
async fn handle_get_oauth2_google_authorization_url_request<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	google_account: String,
	redirect_uri: String,
	ctx: Arc<
		RpcContext<
			Header,
			RpcClient,
			RpcClientFactory,
			EthereumIntentExecutor,
			SolanaIntentExecutor,
			CrossChainIntentExecutor,
		>,
	>,
) -> Result<String, ErrorObjectOwned> {
	debug!("Processing omni_getOAuth2GoogleAuthorizationUrl request");

	let google_identity = Identity::from_web2_account(&google_account, Web2IdentityType::Google);
	let authorization_data = google::get_authorize_data(&ctx.google_client_id, &redirect_uri);
	let storage = OAuth2StateVerifierStorage::new(ctx.storage_db.clone());

	storage
		.insert(&google_identity.hash(), authorization_data.state.clone())
		.map_err(|e| {
			error!("Failed to store OAuth2 state verifier: {:?}", e);
			<ErrorCode as Into<ErrorObjectOwned>>::into(ErrorCode::InternalError)
		})?;

	Ok(authorization_data.authorize_url.to_hex())
}

pub fn register_get_oauth2_google_authorization_url<
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
		.register_async_method(
			"omni_getOAuth2GoogleAuthorizationUrl",
			|params, ctx, _| async move {
				match params.parse::<(String, String)>() {
					Ok((google_account, redirect_uri)) => {
						handle_get_oauth2_google_authorization_url_request(
							google_account,
							redirect_uri,
							ctx,
						)
						.await
					},
					Err(e) => {
						error!("Failed to parse params: {:?}", e);
						Err(<ErrorCode as Into<ErrorObjectOwned>>::into(ErrorCode::ParseError))
					},
				}
			},
		)
		.expect("Failed to register omni_getOAuth2GoogleAuthorizationUrl method");
}
