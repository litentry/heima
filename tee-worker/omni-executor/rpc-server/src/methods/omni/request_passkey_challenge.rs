use crate::{
	server::RpcContext, utils::types::RpcResultExt, utils::validation::parse_rpc_params,
	Deserialize, Serialize,
};
use executor_core::intent_executor::IntentExecutor;
use executor_primitives::UserId;
use executor_storage::PasskeyChallengeStorage;
use jsonrpsee::{types::ErrorObject, RpcModule};
use tracing::*;

const CHALLENGE_TIMEOUT_SECONDS: u64 = 300; // 5 minutes

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RequestPasskeyChallengeParams {
	pub user_id: UserId,
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
			let params = parse_rpc_params::<RequestPasskeyChallengeParams>(params)?;

			debug!("Received omni_requestPasskeyChallenge, params: {:?}", params);

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
			let omni_account = params
				.user_id
				.to_omni_account(&params.client_id)
				.map_err_parse("Failed to convert to omni_account")?;

			challenge_storage
				.store_challenge(&omni_account, &challenge, timeout)
				.map_err_internal("Failed to store challenge")?;

			Ok::<RequestPasskeyChallengeResponse, ErrorObject>(RequestPasskeyChallengeResponse {
				challenge,
				timeout,
			})
		})
		.expect("Failed to register omni_requestPasskeyChallenge method");
}
