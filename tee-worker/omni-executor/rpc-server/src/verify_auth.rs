use crate::server::RpcContext;
use executor_core::intent_executor::IntentExecutor;
use executor_crypto::hashing::blake2_256;
use executor_primitives::{
	signature::HeimaMultiSignature, utils::hex::ToHexPrefixed, Hash, Hashable, Identity,
	OAuth2Data, OAuth2Provider, OmniAuth, VerificationCode, Web2IdentityType,
};
use executor_storage::{OAuth2StateVerifierStorage, Storage, StorageDB, VerificationCodeStorage};
use heima_authentication::{
	auth_token::{AuthTokenClaims, AuthTokenValidator, Error as AuthTokenError, Validation},
	constants::AUTH_TOKEN_ID_TYPE,
	web3::HeimaMessagePayload,
};
use heima_identity_verification::web2::google::decode_id_token;
use oauth_providers::google::GoogleOAuth2Client;
use parity_scale_codec::Encode;
use std::{fmt::Display, sync::Arc};

#[derive(Debug, PartialEq)]
pub enum AuthenticationError {
	Web3InvalidSignature,
	VerificationCodeNotFound,
	InvalidVerificationCode,
	OAuth2Error(String),
	AuthTokenError(AuthTokenError),
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
			AuthenticationError::AuthTokenError(err) => {
				write!(f, "Auth token error: {:?}", err)
			},
		}
	}
}

pub async fn verify_auth<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	ctx: Arc<RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>>,
	auth: &OmniAuth,
) -> Result<(), AuthenticationError> {
	match auth {
		OmniAuth::Web3(ref client_id, ref signer, ref signature) => {
			verify_web3_authentication(ctx.storage_db.clone(), client_id, signer, signature)
		},
		OmniAuth::Email(ref client_id, ref email, ref verification_code) => {
			verify_email_authentication(ctx, client_id, email, verification_code)
		},
		OmniAuth::OAuth2(ref client_id, ref sender, ref oauth2_data) => {
			let verified_identity =
				verify_oauth2_authentication(ctx, client_id, oauth2_data).await?;
			if sender.hash() == verified_identity.hash() {
				Ok(())
			} else {
				Err(AuthenticationError::OAuth2Error("Identity mismatch".to_string()))
			}
		},
		OmniAuth::AuthToken(ref auth_token) => verify_auth_token_authentication(
			&ctx.jwt_rsa_private_key,
			auth_token,
			AUTH_TOKEN_ID_TYPE,
			false,
		)
		.map(|_| ()),
		OmniAuth::Passkey(ref _passkey_data) => {
			todo!()
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
		omni_account: omni_account.to_hex(),
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
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	ctx: Arc<RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>>,
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
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	ctx: Arc<RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>>,
	client_id: &str,
	payload: &OAuth2Data,
) -> Result<Identity, AuthenticationError> {
	match payload.provider {
		OAuth2Provider::Google => verify_google_oauth2(ctx, client_id, payload).await,
	}
}

async fn verify_google_oauth2<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	ctx: Arc<RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>>,
	client_id: &str,
	payload: &OAuth2Data,
) -> Result<Identity, AuthenticationError> {
	let state_verifier_storage = OAuth2StateVerifierStorage::new(ctx.storage_db.clone());
	let key: Hash = blake2_256((client_id, &payload.uid).encode().as_slice()).into();
	let Ok(Some(stored_state)) = state_verifier_storage.get(&key) else {
		return Err(AuthenticationError::OAuth2Error("State verifier not found".to_string()));
	};

	if stored_state != payload.state {
		return Err(AuthenticationError::OAuth2Error("State verifier mismatch".to_string()));
	}

	let google_config =
		ctx.google_oauth2_factory.get_google_config_for_client(client_id).map_err(|e| {
			AuthenticationError::OAuth2Error(format!(
				"Failed to get Google OAuth2 config for client '{}': {}",
				client_id, e
			))
		})?;

	let google_client =
		GoogleOAuth2Client::new(google_config.client_id, google_config.client_secret);
	let code = payload.code.clone();
	let redirect_uri = payload.redirect_uri.clone();
	let token = google_client.exchange_code_for_token(code, redirect_uri).await.map_err(|_| {
		AuthenticationError::OAuth2Error("Could not exchange code for token".to_string())
	})?;
	let id_token = decode_id_token(&token)
		.map_err(|_| AuthenticationError::OAuth2Error("Could not decode id token".to_string()))?;
	let google_identity = Identity::from_web2_account(&id_token.email, Web2IdentityType::Google);

	Ok(google_identity)
}

#[cfg(test)]
mod tests {
	use super::*;
	use alloy_signer::SignerSync;
	use alloy_signer_local::PrivateKeySigner;
	use executor_crypto::{ed25519, sr25519, PairTrait};
	use executor_primitives::{
		signature::EthereumSignature, utils::hex::ToHexPrefixed, Hashable, Identity,
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
			omni_account: alice_omni_account.to_hex(),
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
			omni_account: solana_omni_account.to_hex(),
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
			omni_account: evm_omni_account.to_hex(),
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
			omni_account: alice_omni_account.to_hex(),
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
			omni_account: alice_omni_account.to_hex(),
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
			omni_account: alice_omni_account.to_hex(),
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
