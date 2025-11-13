use crate::{
	detailed_error::DetailedError, error_code::EXTERNAL_API_ERROR_CODE, server::RpcContext,
};
use heima_identity_verification::web2::oauth2_common;
use jsonrpsee::{
	types::{ErrorCode, ErrorObject},
	RpcModule,
};
use oe_core::intent_executor::IntentExecutor;
use oe_crypto::hashing::blake2_256;
use oe_primitives::{Hash, OAuth2Provider, OAuth2VerificationData};
use oe_storage::{OAuth2StateVerifierStorage, Storage};
use parity_scale_codec::Encode;
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Debug, Deserialize)]
struct GetOAuth2AuthorizationDataParams {
	pub provider: String, // "google" or "apple"
	pub uid: String,      // A unique identifier for the user/session requesting the OAuth2 URL
	pub redirect_uri: Option<String>,
	pub client_id: String,
}

#[derive(Debug, Serialize, Clone)]
struct OAuth2AuthorizationData {
	pub authorize_url: String,
	pub client_id: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub redirect_uri: Option<String>,
	pub state: String,
	pub nonce: String,
	pub scope: String,
	pub response_type: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub response_mode: Option<String>,
}

pub fn register_get_oauth2_authorization_data<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_getOAuth2AuthorizationData", |params, ctx, _| async move {
			let params = params
				.parse::<GetOAuth2AuthorizationDataParams>()
				.map_err(|_| ErrorCode::ParseError)?;

			let provider = match params.provider.to_lowercase().as_str() {
				"google" => OAuth2Provider::Google,
				"apple" => OAuth2Provider::Apple,
				_ => {
					error!("Unsupported OAuth2 provider: {}", params.provider);
					return Err(DetailedError::new(
						ErrorCode::InvalidParams.code(),
						"Invalid OAuth2 provider",
					)
					.with_field("provider")
					.with_received(&params.provider)
					.with_expected("google, apple")
					.to_rpc_error());
				},
			};

			let oauth2_config =
				ctx.oauth2_factory.get_config(&params.client_id, provider).map_err(|e| {
					error!(
						"Failed to get {} OAuth2 config for client '{}': {}",
						params.provider, params.client_id, e
					);
					DetailedError::new(
						EXTERNAL_API_ERROR_CODE,
						"Failed to get OAuth2 configuration",
					)
					.with_field("client_id")
					.with_received(&params.client_id)
					.with_reason(format!("Error: {}", e))
					.to_rpc_error()
				})?;

			let provider_config = oauth2_common::OAuth2ProviderConfig::from_provider(provider);

			let authorize_data = oauth2_common::get_authorize_data(
				provider_config.base_url,
				&oauth2_config.client_id,
				params.redirect_uri.as_deref(),
				provider_config.scopes,
				provider_config.use_response_mode,
			);

			let storage = OAuth2StateVerifierStorage::new(ctx.storage_db.clone());
			let key: Hash =
				blake2_256((params.client_id.clone(), params.uid.clone()).encode().as_slice())
					.into();

			let verification_data = OAuth2VerificationData {
				state: authorize_data.state.clone(),
				nonce: authorize_data.nonce.clone(),
			};

			storage.insert(&key, verification_data).map_err(|e| {
				error!("Failed to store OAuth2 verification data: {:?}", e);
				DetailedError::storage_service_error("insert OAuth2 verification data")
					.to_rpc_error()
			})?;

			let response_mode = provider_config.use_response_mode.then(|| "form_post".to_string());

			Ok::<OAuth2AuthorizationData, ErrorObject>(OAuth2AuthorizationData {
				authorize_url: authorize_data.authorize_url,
				client_id: params.client_id,
				redirect_uri: params.redirect_uri,
				state: authorize_data.state,
				nonce: authorize_data.nonce,
				scope: provider_config.scopes.to_string(),
				response_type: "code".to_string(),
				response_mode,
			})
		})
		.expect("Failed to register omni_getOAuth2AuthorizationData method");
}
