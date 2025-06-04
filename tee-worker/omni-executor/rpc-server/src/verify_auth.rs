use crate::server::RpcContext;
use executor_crypto::hashing::blake2_256;
use executor_primitives::{
	signature::HeimaMultiSignature, Hashable, Identity, OAuth2Data, OAuth2Provider, OmniAuth,
	VerificationCode, Web2IdentityType,
};
use executor_storage::{OAuth2StateVerifierStorage, Storage, StorageDB, VerificationCodeStorage};
use heima_authentication::{
	auth_token::{AuthTokenClaims, AuthTokenValidator, Error as AuthTokenError, Validation},
	constants::AUTH_TOKEN_ID_TYPE,
	web3::HeimaMessagePayload,
};
use heima_identity_verification::web2::google::decode_id_token;
use oauth_providers::google::GoogleOAuth2Client;
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

pub async fn verify_auth(ctx: Arc<RpcContext>, auth: &OmniAuth) -> Result<(), AuthenticationError> {
	match auth {
		OmniAuth::Web3(ref signer, ref signature) => {
			verify_web3_authentication(ctx.storage_db.clone(), signer, signature)
		},
		OmniAuth::Email(ref email, ref verification_code) => {
			verify_email_authentication(ctx, email, verification_code)
		},
		OmniAuth::OAuth2(ref sender, ref oauth2_data) => {
			verify_oauth2_authentication(ctx, sender, oauth2_data).await
		},
		OmniAuth::AuthToken(ref auth_token) => verify_auth_token_authentication(
			&ctx.jwt_rsa_private_key,
			auth_token,
			AUTH_TOKEN_ID_TYPE,
			false,
		)
		.map(|_| ()),
	}
}

pub fn verify_web3_authentication(
	storage_db: Arc<StorageDB>,
	signer: &Identity,
	signature: &HeimaMultiSignature,
) -> Result<(), AuthenticationError> {
	let storage_key = signer.to_omni_account().hash();
	let verification_code_storage = VerificationCodeStorage::new(storage_db);
	let Ok(Some(message_code)) = verification_code_storage.get(&storage_key) else {
		return Err(AuthenticationError::VerificationCodeNotFound);
	};
	verification_code_storage
		.remove(&storage_key)
		.map_err(|_| AuthenticationError::VerificationCodeNotFound)?;
	let message = HeimaMessagePayload { message_code };
	let payload = serde_json::to_string(&message).expect("Failed to serialize payload");
	let hashed = blake2_256(payload.as_bytes());

	// Most common signatures variants by clients are verified first (4 and 2).
	match signature.verify(&hashed, signer) {
		true => Ok(()),
		false => Err(AuthenticationError::Web3InvalidSignature),
	}
}

pub fn verify_email_authentication(
	ctx: Arc<RpcContext>,
	email: &str,
	verification_code: &VerificationCode,
) -> Result<(), AuthenticationError> {
	let storage_key = Identity::from_web2_account(email, Web2IdentityType::Email).hash();
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

pub async fn verify_oauth2_authentication(
	ctx: Arc<RpcContext>,
	sender: &Identity,
	payload: &OAuth2Data,
) -> Result<(), AuthenticationError> {
	match payload.provider {
		OAuth2Provider::Google => verify_google_oauth2(ctx, sender, payload).await,
	}
}

async fn verify_google_oauth2(
	ctx: Arc<RpcContext>,
	sender: &Identity,
	payload: &OAuth2Data,
) -> Result<(), AuthenticationError> {
	let state_verifier_storage = OAuth2StateVerifierStorage::new(ctx.storage_db.clone());
	let Ok(Some(state_verifier)) = state_verifier_storage.get(&sender.hash()) else {
		return Err(AuthenticationError::OAuth2Error("State verifier not found".to_string()));
	};
	if state_verifier != payload.state {
		return Err(AuthenticationError::OAuth2Error("State verifier mismatch".to_string()));
	}
	let google_client =
		GoogleOAuth2Client::new(ctx.google_client_id.clone(), ctx.google_client_secret.clone());
	let code = payload.code.clone();
	let redirect_uri = payload.redirect_uri.clone();
	let token = google_client.exchange_code_for_token(code, redirect_uri).await.map_err(|_| {
		AuthenticationError::OAuth2Error("Could not exchange code for token".to_string())
	})?;
	let id_token = decode_id_token(&token)
		.map_err(|_| AuthenticationError::OAuth2Error("Could not decode id token".to_string()))?;
	let google_identity = Identity::from_web2_account(&id_token.email, Web2IdentityType::Google);

	match sender.hash() == google_identity.hash() {
		true => Ok(()),
		false => Err(AuthenticationError::OAuth2Error("Identity mismatch".to_string())),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use executor_crypto::{hashing::blake2_256, sr25519::Pair, PairTrait};
	use executor_primitives::{Hashable, Identity};
	use heima_identity_verification::helpers::generate_otp;
	use tempfile::tempdir;

	#[test]
	fn test_verify_web3_authentication() {
		let tmp_dir = tempdir().unwrap();
		let storage_db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		let alice = Pair::from_string("//Alice", None).unwrap();
		let public_key: [u8; 32] = alice.public().into();
		let alice_identity = Identity::from(public_key);
		let alice_omni_account = alice_identity.to_omni_account();

		let verification_code_storage = VerificationCodeStorage::new(storage_db.clone());

		let message_code = generate_otp(8);

		verification_code_storage
			.insert(&alice_omni_account.hash(), message_code.clone())
			.expect("insert");

		let message = HeimaMessagePayload { message_code };
		let payload = serde_json::to_string(&message).expect("serialize");
		let hashed = blake2_256(payload.as_bytes());

		let signature = alice.sign(&hashed);
		let multi_signature = HeimaMultiSignature::from(signature);

		let result = verify_web3_authentication(storage_db, &alice_identity, &multi_signature);
		assert!(result.is_ok());
	}
}
