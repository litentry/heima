use crate::{
	detailed_error::DetailedError, error_code::PARSE_ERROR_CODE, server::RpcContext,
	validation_helpers::validate_email, Deserialize,
};
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::{Hashable, Identity, Web2IdentityType};
use executor_storage::{Storage, VerificationCodeStorage};
use heima_identity_verification::web2::email::{
	generate_verification_code, send_verification_email,
};
use jsonrpsee::{types::ErrorObjectOwned, RpcModule};
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use std::sync::Arc;
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct RequestEmailVerificationCodeParams {
	pub client_id: String,
	pub user_email: String,
}

#[tracing::instrument(skip(params, ctx), fields(
	client_id = %params.client_id,
	user_email = %params.user_email
))]
async fn handle_request_email_verification_code_request<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	params: RequestEmailVerificationCodeParams,
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
) -> Result<(), ErrorObjectOwned> {
	debug!("Processing omni_requestEmailVerificationCode request");

	validate_email(&params.user_email).map_err(|e| e.to_error_object())?;

	let email_identity = Identity::from_web2_account(&params.user_email, Web2IdentityType::Email);
	let omni_account = email_identity.to_omni_account(&params.client_id);
	let verification_code_storage = VerificationCodeStorage::new(ctx.storage_db.clone());
	let verification_code = generate_verification_code();

	verification_code_storage
		.insert(&omni_account.hash(), verification_code.clone())
		.map_err(|e| {
			error!("Failed to store verification code: {:?}", e);
			DetailedError::storage_error("insert verification code").to_error_object()
		})?;

	// Get the appropriate mailer for this client
	let mailer = ctx.mailer_factory.get_mailer_for_client(&params.client_id).map_err(|e| {
		error!("Failed to get mailer for client '{}': {}", params.client_id, e);
		DetailedError::new(
			crate::error_code::EXTERNAL_API_ERROR_CODE,
			"Failed to initialize email service",
		)
		.with_field("client_id")
		.with_received(&params.client_id)
		.with_reason(format!("Error: {}", e))
		.to_error_object()
	})?;

	send_verification_email(&*mailer, params.user_email.clone(), verification_code)
		.await
		.map_err(|e| {
			error!("Failed to send verification email for client '{}': {:?}", params.client_id, e);
			DetailedError::email_service_error(&params.user_email).to_error_object()
		})?;

	Ok(())
}

pub fn register_request_email_verification_code<
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
		.register_async_method("omni_requestEmailVerificationCode", |params, ctx, _| async move {
			let params = params.parse::<RequestEmailVerificationCodeParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				DetailedError::new(PARSE_ERROR_CODE, "Failed to parse request parameters")
					.with_reason(format!("Invalid JSON structure: {}", e))
					.to_error_object()
			})?;

			handle_request_email_verification_code_request(params, ctx).await
		})
		.expect("Failed to register omni_requestEmailVerificationCode method");
}
