use crate::server::RpcContext;
use executor_core::native_call::NativeCall;
use executor_crypto::hashing::blake2_256;
use executor_primitives::{
	signature::HeimaMultiSignature, utils::hex::hex_encode, Identity, OmniAccountAuthType,
	ShardIdentifier, Web2IdentityType,
};
use executor_storage::{OAuth2StateVerifierStorage, Storage, VerificationCodeStorage};
use heima_authentication::auth_token::{AuthTokenValidator, Validation};
use heima_identity_verification::web2::google::decode_id_token;
use oauth_providers::google::GoogleOAuth2Client;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use parity_scale_codec::{Decode, Encode};
use std::{fmt::Display, sync::Arc};
use tokio::runtime::Handle;

pub type VerificationCode = String;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum Authentication {
	Web3(HeimaMultiSignature),
	Email(VerificationCode),
	AuthToken(String),
	OAuth2(OAuth2Data),
}

impl From<Authentication> for OmniAccountAuthType {
	fn from(value: Authentication) -> Self {
		match value {
			Authentication::Web3(_) => OmniAccountAuthType::Web3,
			Authentication::Email(_) => OmniAccountAuthType::Email,
			Authentication::OAuth2(_) => OmniAccountAuthType::OAuth2,
			Authentication::AuthToken(_) => OmniAccountAuthType::AuthToken,
		}
	}
}

#[derive(Debug)]
pub enum AuthenticationError {
	Web3InvalidSignature,
	EmailVerificationCodeNotFound,
	EmailInvalidVerificationCode,
	OAuth2Error(String),
	AuthTokenError(AuthTokenError),
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
		}
	}
}

#[derive(Debug)]
pub enum AuthTokenError {
	InvalidToken,
	BlockNumberError,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum OAuth2Provider {
	Google,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct OAuth2Data {
	pub provider: OAuth2Provider,
	pub code: String,
	pub state: String,
	pub redirect_uri: String,
}

pub fn verify_web3_authentication(
	signature: &HeimaMultiSignature,
	call: &NativeCall,
	nonce: u32,
	mrenclave: &[u8; 32],
	shard: &ShardIdentifier,
) -> Result<(), AuthenticationError> {
	let mut payload = call.encode();
	payload.append(&mut nonce.encode());
	payload.append(&mut mrenclave.encode());
	payload.append(&mut shard.encode());

	// The signature should be valid in either case:
	// 1. blake2_256(payload)
	// 2. Signature Prefix + blake2_256(payload)

	let hashed = blake2_256(&payload);

	let prettified_msg_hash = call.signature_message_prefix() + &hex_encode(&hashed);
	let prettified_msg_hash = prettified_msg_hash.as_bytes();

	// Most common signatures variants by clients are verified first (4 and 2).
	match signature.verify(prettified_msg_hash, call.sender_identity())
		|| signature.verify(&hashed, call.sender_identity())
	{
		true => Ok(()),
		false => Err(AuthenticationError::Web3InvalidSignature),
	}
}

pub fn verify_email_authentication<
	AccountId,
	Header,
	Hash,
	RpcClient: SubstrateRpcClient<AccountId, Header, Hash>,
	RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, Hash, RpcClient>,
>(
	ctx: Arc<RpcContext<AccountId, Header, Hash, RpcClient, RpcClientFactory>>,
	sender_identity: &Identity,
	verification_code: &VerificationCode,
) -> Result<(), AuthenticationError> {
	let verification_code_storage = VerificationCodeStorage::new(ctx.storage_db.clone());
	let Some(code) = verification_code_storage.get(&sender_identity.hash()) else {
		return Err(AuthenticationError::EmailVerificationCodeNotFound);
	};
	if code != *verification_code {
		return Err(AuthenticationError::EmailInvalidVerificationCode);
	}
	let _ = verification_code_storage.remove(&sender_identity.hash());

	Ok(())
}

pub fn verify_auth_token_authentication<
	AccountId,
	Header,
	Hash,
	RpcClient: SubstrateRpcClient<AccountId, Header, Hash>,
	RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, Hash, RpcClient>,
>(
	ctx: Arc<RpcContext<AccountId, Header, Hash, RpcClient, RpcClientFactory>>,
	handle: Handle,
	sender_identity: &Identity,
	auth_token: &str,
) -> Result<(), AuthenticationError> {
	let current_block = handle
		.block_on(async {
			let client = ctx.parentchain_rpc_client_factory.new_client().await.map_err(|e| {
				log::error!("Could not create client: {:?}", e);
			})?;
			client.get_last_finalized_block_num().await.map_err(|e| {
				log::error!("Could not get last finalized block number: {:?}", e);
			})
		})
		.map_err(|_| AuthenticationError::AuthTokenError(AuthTokenError::BlockNumberError))?;

	let validation = Validation::new(sender_identity.hash().to_string(), current_block);

	if auth_token.validate(ctx.jwt_secret.as_bytes(), validation).is_err() {
		return Err(AuthenticationError::AuthTokenError(AuthTokenError::InvalidToken));
	}

	Ok(())
}

pub fn verify_oauth2_authentication<
	AccountId,
	Header,
	Hash,
	RpcClient: SubstrateRpcClient<AccountId, Header, Hash>,
	RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, Hash, RpcClient>,
>(
	ctx: Arc<RpcContext<AccountId, Header, Hash, RpcClient, RpcClientFactory>>,
	handle: Handle,
	sender_identity: &Identity,
	payload: &OAuth2Data,
) -> Result<(), AuthenticationError> {
	match payload.provider {
		OAuth2Provider::Google => verify_google_oauth2(ctx, handle, sender_identity, payload),
	}
}

fn verify_google_oauth2<
	AccountId,
	Header,
	Hash,
	RpcClient: SubstrateRpcClient<AccountId, Header, Hash>,
	RpcClientFactory: SubstrateRpcClientFactory<AccountId, Header, Hash, RpcClient>,
>(
	ctx: Arc<RpcContext<AccountId, Header, Hash, RpcClient, RpcClientFactory>>,
	handle: Handle,
	sender_identity: &Identity,
	payload: &OAuth2Data,
) -> Result<(), AuthenticationError> {
	let state_verifier_storage = OAuth2StateVerifierStorage::new(ctx.storage_db.clone());
	let Some(state_verifier) = state_verifier_storage.get(&sender_identity.hash()) else {
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

	match sender_identity.hash() == google_identity.hash() {
		true => Ok(()),
		false => Err(AuthenticationError::OAuth2Error("Identity mismatch".to_string())),
	}
}
