use crate::{server::RpcContext, Encode};
use executor_core::native_task::{NativeTask, NativeTaskTrait, NativeTaskWrapper};
use executor_crypto::hashing::blake2_256;
use executor_primitives::{
	signature::HeimaMultiSignature,
	utils::hex::{hex_encode, ToHexPrefixed},
	Identity, MrEnclave, OAuth2Data, OAuth2Provider, OmniAuth, VerificationCode, Web2IdentityType,
};
use executor_storage::{OAuth2StateVerifierStorage, Storage, VerificationCodeStorage};
use heima_authentication::auth_token::{AuthTokenValidator, Error as AuthTokenError, Validation};
use heima_identity_verification::web2::google::decode_id_token;
use oauth_providers::google::GoogleOAuth2Client;
use std::{fmt::Display, sync::Arc};

#[derive(Debug)]
pub enum AuthenticationError {
	Web3InvalidSignature,
	EmailVerificationCodeNotFound,
	EmailInvalidVerificationCode,
	OAuth2Error(String),
	AuthTokenError(AuthTokenError),
	InvalidNonce,
	AuthNotExist,
}

impl Display for AuthenticationError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			AuthenticationError::Web3InvalidSignature => {
				write!(f, "Invalid Web3 signature")
			},
			AuthenticationError::EmailVerificationCodeNotFound => {
				write!(f, "Email verification code not found")
			},
			AuthenticationError::EmailInvalidVerificationCode => {
				write!(f, "Invalid email verification code")
			},
			AuthenticationError::OAuth2Error(msg) => {
				write!(f, "OAuth2 error: {}", msg)
			},
			AuthenticationError::AuthTokenError(err) => {
				write!(f, "Auth token error: {:?}", err)
			},
			AuthenticationError::InvalidNonce => {
				write!(f, "Invalid nonce")
			},
			AuthenticationError::AuthNotExist => {
				write!(f, "Auth not exist")
			},
		}
	}
}

pub async fn verify_auth(
	ctx: Arc<RpcContext>,
	wrapper: &NativeTaskWrapper<NativeTask>,
) -> Result<(), AuthenticationError> {
	match wrapper.auth {
		None => Err(AuthenticationError::AuthNotExist),
		Some(OmniAuth::Web3(ref signature)) => {
			verify_web3_authentication(signature, &wrapper.task, wrapper.nonce, ctx.mrenclave)
		},
		Some(OmniAuth::Email(ref verification_code)) => {
			verify_email_authentication(ctx, wrapper.task.sender(), verification_code)
		},
		Some(OmniAuth::OAuth2(ref oauth2_data)) => {
			verify_oauth2_authentication(ctx, wrapper.task.sender(), oauth2_data).await
		},
		Some(OmniAuth::AuthToken(ref auth_token)) => {
			verify_auth_token_authentication(ctx, wrapper.task.sender(), auth_token)
		},
	}
}

pub fn verify_web3_authentication<T: NativeTaskTrait>(
	signature: &HeimaMultiSignature,
	task: &T,
	nonce: Option<u32>,
	mrenclave: MrEnclave,
) -> Result<(), AuthenticationError> {
	let nonce = nonce.ok_or(AuthenticationError::InvalidNonce)?;

	let mut payload = task.encode();
	payload.append(&mut nonce.encode());
	payload.append(&mut mrenclave.encode());

	// The signature should be valid in either case:
	// 1. blake2_256(payload)
	// 2. Signature Prefix + blake2_256(payload)

	let hashed = blake2_256(&payload);

	let prettified_msg_hash = task.signature_message_prefix() + &hex_encode(&hashed);
	let prettified_msg_hash = prettified_msg_hash.as_bytes();

	// Most common signatures variants by clients are verified first (4 and 2).
	match signature.verify(prettified_msg_hash, task.sender())
		|| signature.verify(&hashed, task.sender())
	{
		true => Ok(()),
		false => Err(AuthenticationError::Web3InvalidSignature),
	}
}

pub fn verify_email_authentication(
	ctx: Arc<RpcContext>,
	sender: &Identity,
	verification_code: &VerificationCode,
) -> Result<(), AuthenticationError> {
	let verification_code_storage = VerificationCodeStorage::new(ctx.storage_db.clone());
	let Some(code) = verification_code_storage.get(&sender.hash()) else {
		return Err(AuthenticationError::EmailVerificationCodeNotFound);
	};
	if code != *verification_code {
		return Err(AuthenticationError::EmailInvalidVerificationCode);
	}
	let _ = verification_code_storage.remove(&sender.hash());

	Ok(())
}

pub fn verify_auth_token_authentication(
	ctx: Arc<RpcContext>,
	sender: &Identity,
	auth_token: &str,
) -> Result<(), AuthenticationError> {
	// TODO: once we start using the AccountStore, we should get the omni account from storage
	let validation = Validation::new(sender.to_omni_account().to_hex());
	auth_token
		.validate(&ctx.jwt_rsa_private_key, validation)
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
	let Some(state_verifier) = state_verifier_storage.get(&sender.hash()) else {
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
