use crate::detailed_error::DetailedError;
use crate::error_code::AUTH_VERIFICATION_FAILED_CODE;
use crate::utils::validation::parse_rpc_params;
use crate::verify_auth::verify_email_authentication;
use crate::RpcContext;
use jsonrpsee::{types::ErrorObject, RpcModule};
use oe_core::intent::executor::IntentExecutor;
use oe_primitives::{UserAuth, UserId};
use tracing::error;

#[derive(Debug, serde::Deserialize)]
pub struct VerifyEmailVerificationCodeTestParams {
	pub auth: UserAuth,
	pub user_id: UserId,
	pub client_id: String,
}

pub fn register_verify_email_verification_code_test<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method(
			"omni_verifyEmailVerificationCodeTest",
			|params, ctx, _ext| async move {
				let params = parse_rpc_params::<VerifyEmailVerificationCodeTestParams>(params)?;

				let email = match &params.user_id {
					UserId::Email(email) => email.clone(),
					_ => {
						error!("Invalid user_id type: expected Email, got {:?}", params.user_id);
						return Err(
							DetailedError::invalid_params("user_id", "expect email").to_rpc_error()
						);
					},
				};

				let verification_code = match &params.auth {
					UserAuth::Email(code) => code.clone(),
					_ => {
						error!("Invalid auth type: expected Email, got {:?}", params.auth);
						return Err(
							DetailedError::invalid_params("auth", "expect email").to_rpc_error()
						);
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
							.to_rpc_error()
					})?;

				Ok::<(), ErrorObject>(())
			},
		)
		.expect("Failed to register omni_verifyEmailVerificationCodeTest method");
}
