use crate::{
	detailed_error::DetailedError, error_code::*, server::RpcContext, verify_auth::verify_auth,
	Deserialize, Serialize,
};
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::{to_omni_auth, UserAuth, UserId};
use executor_storage::PasskeyChallengeStorage;
use heima_primitives::Identity;
use jsonrpsee::{types::ErrorObject, RpcModule};
use tracing::error;

const CHALLENGE_TIMEOUT_SECONDS: u64 = 300; // 5 minutes

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RequestPasskeyChallengeParams {
	pub user_id: UserId,
	pub user_auth: UserAuth,
	pub client_id: String,
}

#[derive(Serialize, Clone)]
pub struct RequestPasskeyChallengeResponse {
	pub challenge: String, // Base64 encoded 32 bytes
	pub timeout: u64,      // Challenge validity in seconds
}

pub fn register_request_passkey_challenge<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_requestPasskeyChallenge", |params, ctx, _| async move {
			let params = params.parse::<RequestPasskeyChallengeParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				DetailedError::new(PARSE_ERROR_CODE, "Failed to parse request parameters")
					.with_reason(format!("Invalid JSON structure: {}", e))
					.to_error_object()
			})?;

			let identity = Identity::try_from(params.user_id.clone()).map_err(|_| {
				error!("Invalid user ID format");
				DetailedError::new(PARSE_ERROR_CODE, "Failed to parse user identity")
					.with_field("user_id")
					.with_reason("Invalid user ID format")
					.with_suggestion(
						"Ensure user_id follows the correct format for the specified type",
					)
					.to_error_object()
			})?;

			let auth = to_omni_auth(&params.user_auth, &params.user_id, &params.client_id)
				.map_err(|e| {
					error!("Failed to convert to OmniAuth: {:?}", e);
					DetailedError::new(PARSE_ERROR_CODE, "Failed to convert authentication data")
						.with_field("user_auth")
						.with_reason(format!("Authentication conversion failed: {}", e))
						.with_suggestion("Ensure user_auth matches the user_id type")
						.to_error_object()
				})?;

			verify_auth(ctx.clone(), &auth).await.map_err(|_| {
				error!("Failed to verify user authentication");
				DetailedError::new(
					AUTH_VERIFICATION_FAILED_CODE,
					"Authentication verification failed",
				)
				.with_reason("User authentication could not be verified")
				.with_suggestion("Check your authentication credentials and try again")
				.to_error_object()
			})?;

			// Generate a random 32-byte challenge
			use rand::RngCore;
			let mut challenge_bytes = [0u8; 32];
			rand::thread_rng().fill_bytes(&mut challenge_bytes);

			// Base64 encode the challenge (URL-safe, no padding)
			use base64::Engine;
			let challenge =
				base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(challenge_bytes);

			// Store challenge with expiration
			let timeout = CHALLENGE_TIMEOUT_SECONDS;
			let challenge_storage = PasskeyChallengeStorage::new(ctx.storage_db.clone());
			let omni_account = identity.to_omni_account(&params.client_id);

			challenge_storage.store_challenge(&omni_account, &challenge, timeout).map_err(
				|_| {
					error!("Failed to store challenge");
					DetailedError::new(INTERNAL_ERROR_CODE, "Failed to store challenge")
						.with_reason("Challenge storage operation failed")
						.with_suggestion(
							"Please try again later or contact support if the issue persists",
						)
						.to_error_object()
				},
			)?;

			Ok::<RequestPasskeyChallengeResponse, ErrorObject>(RequestPasskeyChallengeResponse {
				challenge,
				timeout,
			})
		})
		.expect("Failed to register omni_requestPasskeyChallenge method");
}
