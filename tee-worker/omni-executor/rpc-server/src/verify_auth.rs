use crate::server::RpcContext;
use executor_core::native_task::NativeTaskTrait;
use executor_crypto::hashing::blake2_256;
use executor_primitives::{
	signature::HeimaMultiSignature, utils::hex::hex_encode, Identity, MrEnclave, OAuth2Data,
	OAuth2Provider, VerificationCode, Web2IdentityType,
};
use executor_storage::{OAuth2StateVerifierStorage, Storage, VerificationCodeStorage};
use heima_authentication::auth_token::{AuthTokenValidator, Validation};
use heima_identity_verification::web2::google::decode_id_token;
use oauth_providers::google::GoogleOAuth2Client;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use parity_scale_codec::Encode;
use std::{fmt::Display, sync::Arc};
use tokio::runtime::Handle;

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

#[derive(Debug)]
pub enum AuthTokenError {
	InvalidToken,
	BlockNumberError,
	InvalidIdentity,
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

pub fn verify_email_authentication<
	Header,
	RpcClient: SubstrateRpcClient<Header>,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient>,
>(
	ctx: Arc<RpcContext<Header, RpcClient, RpcClientFactory>>,
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

pub fn verify_auth_token_authentication<
	Header,
	RpcClient: SubstrateRpcClient<Header>,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient>,
>(
	ctx: Arc<RpcContext<Header, RpcClient, RpcClientFactory>>,
	handle: Handle,
	sender: &Identity,
	auth_token: &str,
) -> Result<(), AuthenticationError> {
	let validation = match sender {
		Identity::Email(identity_string) => {
			let Ok(email) = std::str::from_utf8(identity_string.inner_ref()) else {
				return Err(AuthenticationError::AuthTokenError(AuthTokenError::InvalidIdentity));
			};
			Validation::new(email.to_string())
		},
		_ => Validation::new(sender.hash().to_string()),
	};

	if auth_token.validate(&ctx.jwt_rsa_private_key, validation).is_err() {
		return Err(AuthenticationError::AuthTokenError(AuthTokenError::InvalidToken));
	}

	Ok(())
}

pub fn verify_oauth2_authentication<
	Header,
	RpcClient: SubstrateRpcClient<Header>,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient>,
>(
	ctx: Arc<RpcContext<Header, RpcClient, RpcClientFactory>>,
	handle: Handle,
	sender: &Identity,
	payload: &OAuth2Data,
) -> Result<(), AuthenticationError> {
	match payload.provider {
		OAuth2Provider::Google => verify_google_oauth2(ctx, handle, sender, payload),
	}
}

fn verify_google_oauth2<
	Header,
	RpcClient: SubstrateRpcClient<Header>,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient>,
>(
	ctx: Arc<RpcContext<Header, RpcClient, RpcClientFactory>>,
	handle: Handle,
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
	let token = handle
		.block_on(async { google_client.exchange_code_for_token(code, redirect_uri).await })
		.map_err(|_| {
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
