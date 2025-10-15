use crate::{
	detailed_error::DetailedError, error_code::EXTERNAL_API_ERROR_CODE, server::RpcContext,
};
use executor_core::intent_executor::IntentExecutor;
use executor_crypto::hashing::blake2_256;
use executor_primitives::Hash;
use executor_storage::{OAuth2StateVerifierStorage, Storage};
use heima_identity_verification::web2::{apple, google};
use jsonrpsee::{
	types::{ErrorCode, ErrorObject},
	RpcModule,
};
use parity_scale_codec::Encode;
use serde::Deserialize;
use tracing::error;

#[derive(Debug, Deserialize)]
struct GetOAuth2AuthorizationUrlParams {
	pub provider: String, // "google" or "apple"
	pub uid: String,      // A unique identifier for the user/session requesting the OAuth2 URL
	pub redirect_uri: String,
	pub client_id: String,
}

pub fn register_get_oauth2_authorization_url<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>,
	>,
) {
	module
		.register_async_method("omni_getOAuth2AuthorizationUrl", |params, ctx, _| async move {
			let params = params
				.parse::<GetOAuth2AuthorizationUrlParams>()
				.map_err(|_| ErrorCode::ParseError)?;

			let (authorize_url, state) = match params.provider.to_lowercase().as_str() {
				"google" => {
					let google_config = ctx
						.google_oauth2_factory
						.get_google_config_for_client(&params.client_id)
						.map_err(|e| {
							error!(
								"Failed to get Google OAuth2 config for client '{}': {}",
								params.client_id, e
							);
							DetailedError::new(
								EXTERNAL_API_ERROR_CODE,
								"Failed to get Google OAuth2 configuration",
							)
							.with_field("client_id")
							.with_received(&params.client_id)
							.with_reason(format!("Error: {}", e))
							.to_error_object()
						})?;

					let data =
						google::get_authorize_data(&google_config.client_id, &params.redirect_uri);
					(data.authorize_url, data.state)
				},
				"apple" => {
					let apple_config = ctx
						.apple_oauth2_factory
						.get_apple_config_for_client(&params.client_id)
						.map_err(|e| {
							error!(
								"Failed to get Apple OAuth2 config for client '{}': {}",
								params.client_id, e
							);
							DetailedError::new(
								EXTERNAL_API_ERROR_CODE,
								"Failed to get Apple OAuth2 configuration",
							)
							.with_field("client_id")
							.with_received(&params.client_id)
							.with_reason(format!("Error: {}", e))
							.to_error_object()
						})?;

					let data =
						apple::get_authorize_data(&apple_config.client_id, &params.redirect_uri);
					(data.authorize_url, data.state)
				},
				_ => {
					error!("Unsupported OAuth2 provider: {}", params.provider);
					return Err(DetailedError::new(
						ErrorCode::InvalidParams.code(),
						"Invalid OAuth2 provider",
					)
					.with_field("provider")
					.with_received(&params.provider)
					.with_expected("google, apple")
					.to_error_object());
				},
			};

			let storage = OAuth2StateVerifierStorage::new(ctx.storage_db.clone());
			let key: Hash =
				blake2_256((params.client_id.clone(), params.uid.clone()).encode().as_slice())
					.into();

			storage.insert(&key, state).map_err(|_| ErrorCode::InternalError)?;
			Ok::<String, ErrorObject>(authorize_url)
		})
		.expect("Failed to register omni_getOAuth2AuthorizationUrl method");
}
