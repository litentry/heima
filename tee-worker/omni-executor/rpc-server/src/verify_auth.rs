use crate::{detailed_error::DetailedError, server::RpcContext};
use executor_core::intent_executor::IntentExecutor;
use executor_crypto::hashing::blake2_256;
use executor_primitives::{
	signature::HeimaMultiSignature, utils::hex::hex_encode, Hash, Hashable, Identity, OAuth2Data,
	OAuth2Provider, OmniAuth, PasskeyData, VerificationCode, Web2IdentityType,
};
use executor_storage::{
	OAuth2StateVerifierStorage, PasskeyChallengeStorage, Storage, StorageDB,
	VerificationCodeStorage,
};
use heima_authentication::{
	auth_token::{AuthTokenClaims, AuthTokenValidator, Error as AuthTokenError, Validation},
	constants::AUTH_TOKEN_ID_TYPE,
	web3::HeimaMessagePayload,
};
use heima_identity_verification::web2::{apple, google, oauth2_common};
use oauth_providers::{
	AppleProviderConfig, GoogleProviderConfig, OAuth2Client, OAuth2ProviderConfig,
};
use parity_scale_codec::Encode;
use std::{fmt::Display, sync::Arc};
use tracing::debug;

#[derive(Debug, PartialEq)]
pub enum AuthenticationError {
	Web3InvalidSignature,
	VerificationCodeNotFound,
	InvalidVerificationCode,
	OAuth2Error(String),
	OAuth2SubClaimMismatch,
	AuthTokenError(AuthTokenError),
	PasskeyError(String),
}

impl Display for AuthenticationError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			AuthenticationError::Web3InvalidSignature => {
				write!(f, "Invalid Web3 signature")
			},
			AuthenticationError::VerificationCodeNotFound => {
				write!(f, "Verification code not found")
			},
			AuthenticationError::InvalidVerificationCode => {
				write!(f, "Invalid email verification code")
			},
			AuthenticationError::OAuth2Error(msg) => {
				write!(f, "OAuth2 error: {}", msg)
			},
			AuthenticationError::OAuth2SubClaimMismatch => {
				write!(f, "OAuth2 sub claim mismatch between client and provider tokens")
			},
			AuthenticationError::AuthTokenError(err) => {
				write!(f, "Auth token error: {:?}", err)
			},
			AuthenticationError::PasskeyError(msg) => {
				write!(f, "Passkey error: {}", msg)
			},
		}
	}
}

impl AuthenticationError {
	/// Convert AuthenticationError to DetailedError with proper error codes and context
	pub fn to_detailed_error(&self) -> DetailedError {
		use crate::error_code::AUTH_VERIFICATION_FAILED_CODE;
		match self {
			AuthenticationError::PasskeyError(msg) => {
				// Try to match specific passkey error types for better error messages
				if msg.contains("Challenge not found")
					|| msg.contains("Challenge") && msg.contains("not found")
				{
					DetailedError::passkey_challenge_not_found()
				} else if msg.contains("Challenge expired") {
					DetailedError::passkey_challenge_expired()
				} else if msg.contains("Invalid challenge") {
					DetailedError::passkey_invalid_challenge(msg)
				} else if msg.contains("Passkey not found") {
					// Extract credential_id if present in message
					DetailedError::passkey_not_found("")
				} else if msg.contains("signature verification failed")
					|| msg.contains("Invalid signature")
				{
					DetailedError::passkey_signature_invalid(msg)
				} else if msg.contains("Failed to parse") {
					DetailedError::passkey_parse_error("", msg)
				} else if msg.contains("RP ID") {
					// Try to extract RP ID info from message
					let client_id = msg
						.split("client_id: '")
						.nth(1)
						.and_then(|s| s.split('\'').next())
						.unwrap_or("");
					let expected_rp_id = msg
						.split("Expected RP ID: '")
						.nth(1)
						.and_then(|s| s.split('\'').next())
						.unwrap_or("");
					DetailedError::passkey_rp_id_mismatch(expected_rp_id, client_id)
				} else if msg.contains("Replay attack detected") {
					// Extract counter if present
					let counter = msg
						.split("counter (")
						.nth(1)
						.and_then(|s| s.split(')').next())
						.and_then(|s| s.parse::<u32>().ok())
						.unwrap_or(0);
					DetailedError::passkey_replay_attack(counter)
				} else if msg.contains("Counter validation failed")
					|| msg.contains("cloned or downgraded")
				{
					DetailedError::passkey_counter_validation_failed()
				} else if msg.contains("User presence flag") {
					DetailedError::passkey_user_verification_failed("user_presence")
				} else if msg.contains("User verification flag") {
					DetailedError::passkey_user_verification_failed("user_verification")
				} else {
					// Generic passkey error
					DetailedError::new(
						AUTH_VERIFICATION_FAILED_CODE,
						"Passkey authentication failed",
					)
					.with_reason(msg)
				}
			},
			_ => DetailedError::new(AUTH_VERIFICATION_FAILED_CODE, self.to_string()),
		}
	}
}

pub async fn verify_auth<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	ctx: Arc<RpcContext<CrossChainIntentExecutor>>,
	auth: &OmniAuth,
) -> Result<(), AuthenticationError> {
	match auth {
		OmniAuth::Web3(ref client_id, ref signer, ref signature) => {
			verify_web3_authentication(ctx.storage_db.clone(), client_id, signer, signature)
		},
		OmniAuth::Email(ref client_id, ref email, ref verification_code) => {
			verify_email_authentication(ctx, client_id, email, verification_code)
		},
		OmniAuth::OAuth2(ref client_id, ref oauth2_data) => {
			verify_oauth2_authentication(ctx, client_id, oauth2_data).await.map(|_| ())
		},
		OmniAuth::AuthToken(ref auth_token) => verify_auth_token_authentication(
			&ctx.jwt_rsa_private_key,
			auth_token,
			AUTH_TOKEN_ID_TYPE,
			false,
		)
		.map(|_| ()),
		OmniAuth::Passkey(ref passkey_data) => {
			verify_passkey_authentication(ctx, passkey_data).map(|_| ())
		},
	}
}

pub fn verify_web3_authentication(
	storage_db: Arc<StorageDB>,
	client_id: &str,
	signer: &Identity,
	signature: &HeimaMultiSignature,
) -> Result<(), AuthenticationError> {
	let omni_account = signer.to_omni_account(client_id);
	let storage_key = omni_account.hash();
	let verification_code_storage = VerificationCodeStorage::new(storage_db);
	let Ok(Some(message_code)) = verification_code_storage.get(&storage_key) else {
		return Err(AuthenticationError::VerificationCodeNotFound);
	};
	verification_code_storage
		.remove(&storage_key)
		.map_err(|_| AuthenticationError::VerificationCodeNotFound)?;
	let message = HeimaMessagePayload {
		client_id: client_id.to_string(),
		omni_account: hex_encode(omni_account.as_ref()),
		message_code,
	};
	let payload = serde_json::to_string(&message).expect("Failed to serialize payload");

	// Most common signatures variants by clients are verified first (4 and 2).
	match signature.verify(payload.as_bytes(), signer) {
		true => Ok(()),
		false => Err(AuthenticationError::Web3InvalidSignature),
	}
}

pub fn verify_email_authentication<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	ctx: Arc<RpcContext<CrossChainIntentExecutor>>,
	client_id: &str,
	email: &str,
	verification_code: &VerificationCode,
) -> Result<(), AuthenticationError> {
	let email_identity = Identity::from_web2_account(email, Web2IdentityType::Email);
	let omni_account = email_identity.to_omni_account(client_id);
	let storage_key = omni_account.hash();
	let verification_code_storage = VerificationCodeStorage::new(ctx.storage_db.clone());
	let Ok(Some(code)) = verification_code_storage.get(&storage_key) else {
		return Err(AuthenticationError::VerificationCodeNotFound);
	};
	if code != *verification_code {
		return Err(AuthenticationError::InvalidVerificationCode);
	}
	let _ = verification_code_storage.remove(&storage_key);

	Ok(())
}

pub fn verify_auth_token_authentication(
	rsa_private_key: &[u8],
	auth_token: &str,
	token_typ: &str,
	skip_exp_check: bool,
) -> Result<AuthTokenClaims, AuthenticationError> {
	let validation = Validation::new(token_typ.to_string(), skip_exp_check);
	auth_token
		.validate(rsa_private_key, validation)
		.map_err(AuthenticationError::AuthTokenError)
}

pub async fn verify_oauth2_authentication<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	ctx: Arc<RpcContext<CrossChainIntentExecutor>>,
	client_id: &str,
	payload: &OAuth2Data,
) -> Result<Identity, AuthenticationError> {
	verify_oauth2_provider(ctx, client_id, payload).await
}

async fn verify_oauth2_provider<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	ctx: Arc<RpcContext<CrossChainIntentExecutor>>,
	client_id: &str,
	payload: &OAuth2Data,
) -> Result<Identity, AuthenticationError> {
	let state_verifier_storage = OAuth2StateVerifierStorage::new(ctx.storage_db.clone());
	let key: Hash = blake2_256((client_id, &payload.uid).encode().as_slice()).into();
	let Ok(Some(verification_data)) = state_verifier_storage.get(&key) else {
		return Err(AuthenticationError::OAuth2Error("State verifier not found".to_string()));
	};

	if let Err(e) = state_verifier_storage.remove(&key) {
		tracing::warn!("Failed to remove OAuth2 verification data: {:?}", e);
	}

	if verification_data.state != payload.state {
		return Err(AuthenticationError::OAuth2Error("State verifier mismatch".to_string()));
	}

	let provider_str = match payload.provider {
		OAuth2Provider::Google => "google",
		OAuth2Provider::Apple => "apple",
	};

	let oauth2_config =
		ctx.oauth2_factory.get_config(client_id, payload.provider).map_err(|e| {
			AuthenticationError::OAuth2Error(format!(
				"Failed to get {} OAuth2 config for client '{}': {}",
				provider_str, client_id, e
			))
		})?;

	let client_sub = match payload.provider {
		OAuth2Provider::Google => {
			let id_token: google::IdToken = oauth2_common::decode_id_token(&payload.id_token)
				.map_err(|_| {
					AuthenticationError::OAuth2Error("Could not decode Google id token".to_string())
				})?;

			verify_id_token_claims(
				&id_token.aud,
				id_token.nonce.as_deref(),
				&oauth2_config.client_id,
				&verification_data.nonce,
			)?;

			id_token.sub
		},
		OAuth2Provider::Apple => {
			let id_token: apple::IdToken = oauth2_common::decode_id_token(&payload.id_token)
				.map_err(|_| {
					AuthenticationError::OAuth2Error("Could not decode Apple id token".to_string())
				})?;

			verify_id_token_claims(
				&id_token.aud,
				id_token.nonce.as_deref(),
				&oauth2_config.client_id,
				&verification_data.nonce,
			)?;

			id_token.sub
		},
	};

	let token_endpoint = match payload.provider {
		OAuth2Provider::Google => GoogleProviderConfig.token_endpoint(),
		OAuth2Provider::Apple => AppleProviderConfig.token_endpoint(),
	};

	let oauth2_client = OAuth2Client::new(
		oauth2_config.client_id,
		oauth2_config.client_secret,
		token_endpoint.to_string(),
	);

	let code = payload.code.clone();
	let redirect_uri = payload.redirect_uri.clone();
	let provider_id_token =
		oauth2_client.exchange_code_for_token(code, redirect_uri).await.map_err(|e| {
			AuthenticationError::OAuth2Error(format!("Could not exchange code for token: {}", e))
		})?;

	let provider_sub = match payload.provider {
		OAuth2Provider::Google => {
			let id_token: google::IdToken = oauth2_common::decode_id_token(&provider_id_token)
				.map_err(|_| {
					AuthenticationError::OAuth2Error(
						"Could not decode Google id token from provider".to_string(),
					)
				})?;

			id_token.sub
		},
		OAuth2Provider::Apple => {
			let id_token: apple::IdToken = oauth2_common::decode_id_token(&provider_id_token)
				.map_err(|_| {
					AuthenticationError::OAuth2Error(
						"Could not decode Apple id token from provider".to_string(),
					)
				})?;

			id_token.sub
		},
	};

	if client_sub != provider_sub {
		tracing::warn!(
			"OAuth2 sub claim mismatch: client_sub={}, provider_sub={}",
			client_sub,
			provider_sub
		);
		return Err(AuthenticationError::OAuth2SubClaimMismatch);
	}

	let identity_type = match payload.provider {
		OAuth2Provider::Google => Web2IdentityType::Google,
		OAuth2Provider::Apple => Web2IdentityType::Apple,
	};

	let identity = Identity::from_web2_account(&provider_sub, identity_type);

	Ok(identity)
}

fn verify_id_token_claims(
	aud: &str,
	nonce: Option<&str>,
	client_id: &str,
	expected_nonce: &str,
) -> Result<(), AuthenticationError> {
	if aud != client_id {
		return Err(AuthenticationError::OAuth2Error(
			"ID token audience does not match client_id".to_string(),
		));
	}
	let Some(nonce) = nonce else {
		return Err(AuthenticationError::OAuth2Error("ID token missing nonce".to_string()));
	};
	if nonce != expected_nonce {
		return Err(AuthenticationError::OAuth2Error("ID token nonce mismatch".to_string()));
	}
	Ok(())
}

pub fn verify_passkey_authentication<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	ctx: Arc<RpcContext<CrossChainIntentExecutor>>,
	passkey_data: &PasskeyData,
) -> Result<(), AuthenticationError> {
	use crate::methods::omni::{get_origin_for_client, get_rp_id_for_client};
	use executor_crypto::passkey::{ClientData, PasskeyVerifier};
	use executor_storage::PasskeyStorage;

	let passkey_identity =
		Identity::from_web2_account(&passkey_data.user_id, Web2IdentityType::Passkey);
	let omni_account = passkey_identity.to_omni_account(&passkey_data.client_id);
	let client_data: ClientData =
		PasskeyVerifier::parse_client_data_json(&passkey_data.client_data_json).map_err(|e| {
			AuthenticationError::PasskeyError(format!("Failed to parse client data: {}", e))
		})?;

	let expected_origin = get_origin_for_client(&passkey_data.client_id);
	if client_data.origin != expected_origin {
		return Err(AuthenticationError::PasskeyError(format!(
			"Client data origin mismatch: expected '{}', got '{}'",
			expected_origin,
			client_data.origin.as_str()
		)));
	}

	const EXPECTED_PASSKEY_TYPE: &str = "webauthn.get";
	if client_data.type_ != EXPECTED_PASSKEY_TYPE {
		return Err(AuthenticationError::PasskeyError(format!(
			"Invalid client data type: expected '{}', got '{}'",
			EXPECTED_PASSKEY_TYPE,
			client_data.type_.as_str()
		)));
	}

	debug!(
		"TEST, client_data: {:?}, user_id: {:?}, client_id: {:?}, omni: {:?}",
		client_data, passkey_data.user_id, passkey_data.client_id, omni_account
	);

	// Verify challenge
	let challenge_storage = PasskeyChallengeStorage::new(ctx.storage_db.clone());
	challenge_storage
		.verify_and_consume_challenge(&client_data.challenge, &omni_account)
		.map_err(|e| {
			use executor_storage::PasskeyChallengeError;
			match e {
				PasskeyChallengeError::ChallengeNotFound => {
					AuthenticationError::PasskeyError("Challenge not found".to_string())
				},
				PasskeyChallengeError::ChallengeExpired => {
					AuthenticationError::PasskeyError("Challenge expired".to_string())
				},
				PasskeyChallengeError::InvalidChallenge => {
					AuthenticationError::PasskeyError("Invalid challenge".to_string())
				},
				_ => AuthenticationError::PasskeyError("Challenge verification failed".to_string()),
			}
		})?;

	// Look up the stored passkey record using omni_account + credential_id
	let passkey_record = PasskeyStorage::new(ctx.storage_db.clone())
		.get_passkey(&omni_account, &passkey_data.credential_id)
		.map_err(|_| AuthenticationError::PasskeyError("Storage error".to_string()))?
		.ok_or_else(|| {
			AuthenticationError::PasskeyError(
				"Passkey not found for this account and credential".to_string(),
			)
		})?;
	let public_key = PasskeyVerifier::from_sec1_bytes(&passkey_record.pubkey).map_err(|e| {
		AuthenticationError::PasskeyError(format!("Invalid stored public key: {}", e))
	})?;

	// Parse auth_data bytes for validation
	let auth_data_bytes = hex::decode(&passkey_data.auth_data).map_err(|_| {
		AuthenticationError::PasskeyError("Invalid auth data hex format".to_string())
	})?;

	if auth_data_bytes.len() < 37 {
		return Err(AuthenticationError::PasskeyError("Auth data too short".to_string()));
	}

	// CRITICAL SECURITY CHECK: Verify RP ID hash
	// The first 32 bytes of auth data must be SHA-256(RP ID) to prevent phishing attacks
	// This ensures the authenticator signed for the correct domain
	let expected_rp_id = get_rp_id_for_client(&passkey_data.client_id);

	PasskeyVerifier::verify_rp_id_hash(&auth_data_bytes, expected_rp_id).map_err(|e| {
		AuthenticationError::PasskeyError(format!(
			"RP ID validation failed: {}. Expected RP ID: '{}' for client_id: '{}'",
			e, expected_rp_id, passkey_data.client_id
		))
	})?;

	// Check flags (UP bit must be set, UV bit for security)
	let flags = auth_data_bytes[32];
	let up_flag = (flags & 0x01) != 0;
	let uv_flag = (flags & 0x04) != 0;

	if !up_flag {
		return Err(AuthenticationError::PasskeyError("User presence flag not set".to_string()));
	}

	if !uv_flag {
		return Err(AuthenticationError::PasskeyError(
			"User verification flag not set".to_string(),
		));
	}

	// Verify the passkey signature (pure cryptographic verification)
	let is_valid = PasskeyVerifier::verify_passkey_signature_only(
		&passkey_data.auth_data,
		&passkey_data.client_data_json,
		&passkey_data.signature,
		&public_key,
	)
	.map_err(|e| {
		AuthenticationError::PasskeyError(format!("Passkey signature verification failed: {}", e))
	})?;

	if !is_valid {
		return Err(AuthenticationError::Web3InvalidSignature);
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use alloy_signer::SignerSync;
	use alloy_signer_local::PrivateKeySigner;
	use executor_crypto::{ed25519, sr25519, PairTrait};
	use executor_primitives::{
		signature::EthereumSignature, utils::hex::hex_encode, Hashable, Identity,
	};
	use heima_identity_verification::helpers::generate_otp;
	use tempfile::tempdir;

	#[test]
	fn test_verify_substrate_authentication() {
		let tmp_dir = tempdir().unwrap();
		let storage_db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		let alice = sr25519::Pair::from_string("//Alice", None).unwrap();
		let public_key: [u8; 32] = alice.public().into();
		let alice_identity = Identity::Substrate(public_key.into());
		let client_id = "test_client".to_string();
		let alice_omni_account = alice_identity.to_omni_account(&client_id);
		let verification_code_storage = VerificationCodeStorage::new(storage_db.clone());
		let message_code = generate_otp(8);

		verification_code_storage
			.insert(&alice_omni_account.hash(), message_code.clone())
			.expect("insert");

		let message = HeimaMessagePayload {
			message_code,
			omni_account: hex_encode(alice_omni_account.as_ref()),
			client_id: client_id.to_string(),
		};

		let payload = serde_json::to_string(&message).expect("serialize");

		let signature = alice.sign(payload.as_bytes());
		let multi_signature = HeimaMultiSignature::from(signature);

		let result =
			verify_web3_authentication(storage_db, &client_id, &alice_identity, &multi_signature);
		assert!(result.is_ok());
	}

	#[test]
	fn test_verify_solana_authentication() {
		let tmp_dir = tempdir().unwrap();
		let storage_db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		// Create Ed25519 keypair for Solana
		let (keypair, _) = ed25519::Pair::generate();
		let public_key: [u8; 32] = keypair.public().into();
		let solana_identity = Identity::Solana(public_key.into());
		let client_id = "test_client_solana".to_string();
		let solana_omni_account = solana_identity.to_omni_account(&client_id);
		let verification_code_storage = VerificationCodeStorage::new(storage_db.clone());
		let message_code = generate_otp(8);

		verification_code_storage
			.insert(&solana_omni_account.hash(), message_code.clone())
			.expect("insert");

		let message = HeimaMessagePayload {
			message_code,
			omni_account: hex_encode(solana_omni_account.as_ref()),
			client_id: client_id.to_string(),
		};

		let payload = serde_json::to_string(&message).expect("serialize");

		let signature = keypair.sign(payload.as_bytes());
		let multi_signature = HeimaMultiSignature::from(signature);

		let result =
			verify_web3_authentication(storage_db, &client_id, &solana_identity, &multi_signature);
		assert!(result.is_ok());
	}

	#[test]
	fn test_verify_evm_authentication() {
		let tmp_dir = tempdir().unwrap();
		let storage_db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		let evm_signer = PrivateKeySigner::random();
		let signer_address = evm_signer.address();
		let evm_identity = Identity::Evm(signer_address.0.as_slice().try_into().unwrap());
		let client_id = "test_client_evm".to_string();
		let evm_omni_account = evm_identity.to_omni_account(&client_id);
		let verification_code_storage = VerificationCodeStorage::new(storage_db.clone());
		let message_code = generate_otp(8);

		verification_code_storage
			.insert(&evm_omni_account.hash(), message_code.clone())
			.expect("insert");

		let message = HeimaMessagePayload {
			message_code,
			omni_account: hex_encode(evm_omni_account.as_ref()),
			client_id: client_id.to_string(),
		};

		let payload = serde_json::to_string(&message).expect("serialize");
		let signature = evm_signer.sign_message_sync(payload.as_bytes()).expect("sign message");

		let ethereum_signature = EthereumSignature(signature.into());
		let multi_signature = HeimaMultiSignature::Ethereum(ethereum_signature);

		let result =
			verify_web3_authentication(storage_db, &client_id, &evm_identity, &multi_signature);
		assert!(result.is_ok());
	}

	#[test]
	fn test_verify_web3_authentication_invalid_signature() {
		let tmp_dir = tempdir().unwrap();
		let storage_db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		let alice = sr25519::Pair::from_string("//Alice", None).unwrap();
		let bob = sr25519::Pair::from_string("//Bob", None).unwrap();

		let alice_public_key: [u8; 32] = alice.public().into();
		let alice_identity = Identity::from(alice_public_key);
		let client_id = "test_client".to_string();
		let alice_omni_account = alice_identity.to_omni_account(&client_id);
		let verification_code_storage = VerificationCodeStorage::new(storage_db.clone());
		let message_code = generate_otp(8);

		verification_code_storage
			.insert(&alice_omni_account.hash(), message_code.clone())
			.expect("insert");

		let message = HeimaMessagePayload {
			message_code,
			omni_account: hex_encode(alice_omni_account.as_ref()),
			client_id: client_id.to_string(),
		};

		let payload = serde_json::to_string(&message).expect("serialize");

		// Sign with Bob's key but try to verify with Alice's identity
		let signature = bob.sign(payload.as_bytes());
		let multi_signature = HeimaMultiSignature::from(signature);

		let result =
			verify_web3_authentication(storage_db, &client_id, &alice_identity, &multi_signature);
		assert_eq!(result, Err(AuthenticationError::Web3InvalidSignature));
	}

	#[test]
	fn test_verify_web3_authentication_missing_verification_code() {
		let tmp_dir = tempdir().unwrap();
		let storage_db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		let alice = sr25519::Pair::from_string("//Alice", None).unwrap();
		let public_key: [u8; 32] = alice.public().into();
		let alice_identity = Identity::from(public_key);
		let client_id = "test_client".to_string();

		let alice_omni_account = alice_identity.to_omni_account(&client_id);
		let message_code = generate_otp(8);

		let message = HeimaMessagePayload {
			message_code,
			omni_account: hex_encode(alice_omni_account.as_ref()),
			client_id: client_id.to_string(),
		};

		let payload = serde_json::to_string(&message).expect("serialize");
		let signature = alice.sign(payload.as_bytes());
		let multi_signature = HeimaMultiSignature::from(signature);

		// Don't insert verification code
		let result =
			verify_web3_authentication(storage_db, &client_id, &alice_identity, &multi_signature);
		assert_eq!(result, Err(AuthenticationError::VerificationCodeNotFound));
	}

	#[test]
	fn test_verify_web3_authentication_invalid_verification_code() {
		let tmp_dir = tempdir().unwrap();
		let storage_db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		let alice = sr25519::Pair::from_string("//Alice", None).unwrap();
		let public_key: [u8; 32] = alice.public().into();
		let alice_identity = Identity::from(public_key);
		let client_id = "test_client".to_string();

		let alice_omni_account = alice_identity.to_omni_account(&client_id);
		let verification_code_storage = VerificationCodeStorage::new(storage_db.clone());
		let message_code = generate_otp(8);

		verification_code_storage
			.insert(&alice_omni_account.hash(), message_code.clone())
			.expect("insert");

		let message = HeimaMessagePayload {
			message_code: "invalid_code".to_string(), // Use an invalid code
			omni_account: hex_encode(alice_omni_account.as_ref()),
			client_id: client_id.to_string(),
		};

		let payload = serde_json::to_string(&message).expect("serialize");
		let signature = alice.sign(payload.as_bytes());
		let multi_signature = HeimaMultiSignature::from(signature);

		let result =
			verify_web3_authentication(storage_db, &client_id, &alice_identity, &multi_signature);
		assert_eq!(result, Err(AuthenticationError::Web3InvalidSignature));
	}
}
