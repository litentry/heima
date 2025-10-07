use crate::{
	detailed_error::DetailedError, error_code::PARSE_ERROR_CODE, server::RpcContext,
	validation_helpers::validate_email, Deserialize,
};
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::{Hashable, Identity, Web2IdentityType};
use executor_storage::{Storage, VerificationCodeStorage};
use heima_identity_verification::web2::email::{
	generate_verification_code, send_verification_email, send_wildmeta_verification_email,
};
use jsonrpsee::{types::ErrorObject, RpcModule};
use tracing::{error, info};

#[derive(Debug, Deserialize)]
pub struct RequestEmailVerificationCodeParams {
	pub client_id: String,
	pub user_email: String,
}

pub fn register_request_email_verification_code<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>,
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

			info!("[EMAIL_LIFECYCLE] Received omni_requestEmailVerificationCode, client_id: {}, user_email: {}", params.client_id, params.user_email);

			validate_email(&params.user_email).map_err(|e| {
				error!("[EMAIL_LIFECYCLE] Email validation failed for {}: {:?}", params.user_email, e);
				e.to_error_object()
			})?;

			let email_identity =
				Identity::from_web2_account(&params.user_email, Web2IdentityType::Email);
			let omni_account = email_identity.to_omni_account(&params.client_id);

			let verification_code_storage = VerificationCodeStorage::new(ctx.storage_db.clone());
			let verification_code = generate_verification_code();

			verification_code_storage
				.insert(&omni_account.hash(), verification_code.clone())
				.map_err(|e| {
					error!("[EMAIL_LIFECYCLE] Failed to store verification code for {}: {:?}", params.user_email, e);
					DetailedError::storage_error("insert verification code").to_error_object()
				})?;

			// Get the appropriate mailer for this client
			let mailer =
				ctx.mailer_factory.get_mailer_for_client(&params.client_id).map_err(|e| {
					error!("[EMAIL_LIFECYCLE] Failed to get mailer for client '{}': {}", params.client_id, e);
					DetailedError::new(
						crate::error_code::EXTERNAL_API_ERROR_CODE,
						"Failed to initialize email service",
					)
					.with_field("client_id")
					.with_received(&params.client_id)
					.with_reason(format!("Error: {}", e))
					.to_error_object()
				})?;

			// Use Wildmeta template for wildmeta client
			if params.client_id.to_lowercase() == "wildmeta" {
				send_wildmeta_verification_email(
					&*mailer,
					params.user_email.clone(),
					verification_code.clone(),
				)
				.await
				.map_err(|e| {
					error!("[EMAIL_LIFECYCLE] Failed to send Wildmeta verification email to {} (client: {}): {:?}", params.user_email, params.client_id, e);
					DetailedError::email_service_error(&params.user_email).to_error_object()
				})?;
			} else {
				send_verification_email(&*mailer, params.user_email.clone(), verification_code.clone())
					.await
					.map_err(|e| {
						error!("[EMAIL_LIFECYCLE] Failed to send verification email to {} (client: {}): {:?}", params.user_email, params.client_id, e);
						DetailedError::email_service_error(&params.user_email).to_error_object()
					})?;
			}

			Ok::<(), ErrorObject>(())
		})
		.expect("Failed to register omni_requestEmailVerificationCode method");
}
