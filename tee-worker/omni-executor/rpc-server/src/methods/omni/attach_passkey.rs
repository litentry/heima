use crate::{
	error_code::*, server::RpcContext, verify_auth::verify_auth, Deserialize, ErrorCode, Serialize,
};

use executor_core::intent_executor::IntentExecutor;
use executor_crypto::passkey::{AttestationResult, PasskeyVerifier};
use executor_primitives::{to_omni_auth, utils::hex::ToHexPrefixed, UserAuth, UserId};
use executor_storage::{
	PasskeyChallengeError, PasskeyChallengeStorage, PasskeyError, PasskeyStorage,
};
use heima_primitives::Identity;
use jsonrpsee::{types::ErrorObject, RpcModule};
use tracing::error;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AttachPasskeyParams {
	pub user_id: UserId,
	pub user_auth: UserAuth,
	pub client_id: String,
	pub attestation_object: String, // Base64 encoded WebAuthn attestation object
	pub client_data_json: String,   // Base64 encoded WebAuthn client data JSON
}

#[derive(Serialize, Clone)]
pub struct AttachPasskeyResponse {
	pub success: bool,
	pub message: String,
}

pub fn register_attach_passkey<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>,
	>,
) {
	module
		.register_async_method("omni_attachPasskey", |params, ctx, _| async move {
			let params = params.parse::<AttachPasskeyParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				ErrorCode::ParseError
			})?;

			let identity = Identity::try_from(params.user_id.clone()).map_err(|_| {
				error!("Invalid existing user ID format");
				ErrorCode::ParseError
			})?;

			let auth = to_omni_auth(&params.user_auth, &params.user_id, &params.client_id)
				.map_err(|e| {
					error!("Failed to convert to OmniAuth: {:?}", e);
					ErrorCode::ParseError
				})?;

			verify_auth(ctx.clone(), &auth).await.map_err(|e| {
				error!("Failed to verify existing user authentication: {:?}", e);
				e.to_detailed_error().to_error_object()
			})?;

			let omni_account = identity.to_omni_account(&params.client_id);

			let expected_origin =
				crate::methods::omni::common::get_origin_for_client(&params.client_id);

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
									error!("Challenge not found");
								},
								PasskeyChallengeError::ChallengeExpired => {
									error!("Challenge expired");
								},
								PasskeyChallengeError::InvalidChallenge => {
									error!("Invalid challenge");
								},
								_ => {
									error!("Challenge verification failed: {:?}", e);
								},
							}
							executor_crypto::passkey::PasskeyError::ChallengeVerificationFailed
						})
				},
			)
			.map_err(|e| {
				error!("Client data verification failed: {:?}", e);
				match e {
					executor_crypto::passkey::PasskeyError::ChallengeVerificationFailed => {
						ErrorCode::ServerError(-32011) // Challenge mismatch
					},
					executor_crypto::passkey::PasskeyError::OriginVerificationFailed => {
						ErrorCode::ServerError(-32012) // Origin mismatch
					},
					executor_crypto::passkey::PasskeyError::AttestationParseError(_) => {
						ErrorCode::ServerError(-32013) // Attestation parse error
					},
					_ => ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE),
				}
			})?;

			let AttestationResult { credential_id, public_key } =
				PasskeyVerifier::verify_attestation(&params.attestation_object).map_err(|e| {
					error!("WebAuthn attestation verification failed: {:?}", e);
					match e {
						executor_crypto::passkey::PasskeyError::AttestationParseError(_) => {
							ErrorCode::ServerError(-32013) // Attestation parse error
						},
						_ => ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE),
					}
				})?;

			let public_key_sec1_bytes = public_key.verifying_key.to_sec1_bytes();
			let passkey_storage = PasskeyStorage::new(ctx.storage_db.clone());
			passkey_storage
				.add_passkey(&omni_account, &credential_id, &public_key_sec1_bytes)
				.map_err(|e| match e {
					PasskeyError::DuplicatePasskey => {
						error!("Duplicate passkey (same omni_account + credential_id)");
						ErrorCode::ServerError(-32001)
					},
					_ => {
						error!("Failed to store passkey: {:?}", e);
						ErrorCode::ServerError(AUTH_VERIFICATION_FAILED_CODE)
					},
				})?;

			Ok::<AttachPasskeyResponse, ErrorObject>(AttachPasskeyResponse {
				success: true,
				message: format!(
					"Passkey ({}) successfully attached to account {}",
					credential_id,
					omni_account.to_hex()
				),
			})
		})
		.expect("Failed to register omni_attachPasskey method");
}
