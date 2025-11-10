use crate::{
	detailed_error::DetailedError, server::RpcContext, utils::types::RpcResultExt,
	utils::validation::parse_rpc_params, verify_auth::verify_auth, Deserialize, Serialize,
};

use executor_core::intent_executor::IntentExecutor;
use executor_crypto::passkey::{AttestationResult, PasskeyVerifier};
use executor_primitives::{to_omni_auth, utils::hex::hex_encode, UserAuth, UserId};
use executor_storage::{PasskeyChallengeError, PasskeyChallengeStorage, PasskeyStorage};
use heima_primitives::Identity;
use jsonrpsee::{types::ErrorObject, RpcModule};
use tracing::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AttachPasskeyParams {
	pub user_id: UserId,
	pub user_auth: UserAuth,
	pub client_id: String,
	pub attestation_object: String, // Base64 encoded WebAuthn attestation object
	pub client_data_json: String,   // Base64 encoded WebAuthn client data JSON
	pub alias_name: Option<String>, // Optional alias name for the passkey
}

#[derive(Serialize, Clone)]
pub struct AttachPasskeyResponse {
	pub success: bool,
	pub message: String,
}

pub fn register_attach_passkey<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_attachPasskey", |params, ctx, _| async move {
			let params = parse_rpc_params::<AttachPasskeyParams>(params)?;

			debug!("Received omni_attachPasskey, params: {:?}", params);

			// Reject UserId::Passkey type - passkeys cannot be attached to passkey identities
			if matches!(params.user_id, UserId::Passkey(_)) {
				error!("Cannot attach passkey to a Passkey user_id type");
				return Err(DetailedError::invalid_params(
					"user_id",
					"UserId::Passkey type is not allowed for passkey attachment",
				)
				.with_reason("Passkeys can only be attached to non-passkey identity types")
				.with_suggestion("Use a different identity type (e.g. Email) as user_id")
				.to_rpc_error());
			}

			let identity = Identity::try_from(params.user_id.clone())
				.map_err_parse("Invalid user ID format")?;

			let auth = to_omni_auth(&params.user_auth, &params.user_id, &params.client_id)
				.map_err_parse("Failed to convert to OmniAuth")?;

			verify_auth(ctx.clone(), &auth).await.map_err(|e| {
				error!("Failed to verify existing user authentication: {:?}", e);
				e.to_detailed_error().to_rpc_error()
			})?;

			let omni_account = identity.to_omni_account(&params.client_id);

			let expected_origin = super::get_origin_for_client(&params.client_id);

			// Verify client data JSON and consume challenge
			let challenge_storage = PasskeyChallengeStorage::new(ctx.storage_db.clone());
			PasskeyVerifier::verify_client_data_json(
				&params.client_data_json,
				omni_account.as_ref(),
				expected_origin,
				"webauthn.create", // For passkey registration/attachment
				|challenge, omni_account| {
					challenge_storage
						.verify_and_consume_challenge(challenge, &(*omni_account).into())
						.map_err(|e| {
							match e {
								PasskeyChallengeError::ChallengeNotFound => {
									error!("Challenge not found for passkey attachment");
								},
								PasskeyChallengeError::ChallengeExpired => {
									error!("Challenge expired for passkey attachment");
								},
								PasskeyChallengeError::InvalidChallenge => {
									error!("Invalid challenge for passkey attachment");
								},
								_ => {
									error!(
										"Challenge verification failed during passkey attachment: {:?}",
										e
									);
								},
							}
							executor_crypto::passkey::PasskeyError::ChallengeVerificationFailed
						})
				},
			)
			.map_err_internal("Client data verification failed")?;

			let AttestationResult { credential_id, public_key } =
				PasskeyVerifier::verify_attestation(&params.attestation_object)
					.map_err_parse("Attestation verification failed")?;

			let public_key_sec1_bytes = public_key.verifying_key.to_sec1_bytes();
			let passkey_storage = PasskeyStorage::new(ctx.storage_db.clone());
			passkey_storage
				.add_passkey(
					&omni_account,
					&credential_id,
					&public_key_sec1_bytes,
					params.alias_name,
				)
				.map_err_internal("Failed to attach passkey")?;

			Ok::<AttachPasskeyResponse, ErrorObject>(AttachPasskeyResponse {
				success: true,
				message: format!(
					"Passkey ({}) successfully attached to account {}",
					credential_id,
					hex_encode(omni_account.as_ref())
				),
			})
		})
		.expect("Failed to register omni_attachPasskey method");
}
