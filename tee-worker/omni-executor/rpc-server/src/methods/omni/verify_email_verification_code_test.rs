use crate::detailed_error::DetailedError;
use crate::error_code::{AUTH_VERIFICATION_FAILED_CODE, INVALID_PARAMS_CODE, PARSE_ERROR_CODE};
use crate::server::RpcContext;
use crate::verify_auth::verify_email_authentication;
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::{UserAuth, UserId};
use jsonrpsee::{types::ErrorObject, RpcModule};
use tracing::error;

#[derive(Debug, serde::Deserialize)]
pub struct VerifyEmailVerificationCodeTestParams {
	pub auth: UserAuth,
	pub user_id: UserId,
	pub client_id: String,
}

pub fn register_verify_email_verification_code_test<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>,
	>,
) {
	module
		.register_async_method(
			"omni_verifyEmailVerificationCodeTest",
			|params, ctx, _ext| async move {
				let params =
					params.parse::<VerifyEmailVerificationCodeTestParams>().map_err(|e| {
						error!("Failed to parse params: {:?}", e);
						DetailedError::new(PARSE_ERROR_CODE, "Parse error")
							.with_reason("Invalid JSON format or missing required fields")
							.to_error_object()
					})?;

				let email = match &params.user_id {
					UserId::Email(email) => email.clone(),
					_ => {
						error!("Invalid user_id type: expected Email, got {:?}", params.user_id);
						return Err(DetailedError::new(INVALID_PARAMS_CODE, "Invalid parameter")
							.with_field("user_id")
							.with_reason("user_id must be of type 'email'")
							.to_error_object());
					},
				};

				let verification_code = match &params.auth {
					UserAuth::Email(code) => code.clone(),
					_ => {
						error!("Invalid auth type: expected Email, got {:?}", params.auth);
						return Err(DetailedError::new(INVALID_PARAMS_CODE, "Invalid parameter")
							.with_field("auth")
							.with_reason("auth must be of type 'email' with verification code")
							.to_error_object());
					},
				};

				verify_email_authentication(ctx, &params.client_id, &email, &verification_code)
					.map_err(|e| {
						error!(
							"Email verification failed for {} (client: {}): {}",
							email, params.client_id, e
						);
						DetailedError::new(AUTH_VERIFICATION_FAILED_CODE, "Authentication failed")
							.with_reason(format!("Email verification failed: {}", e))
							.to_error_object()
					})?;

				Ok::<(), ErrorObject>(())
			},
		)
		.expect("Failed to register omni_verifyEmailVerificationCodeTest method");
}
