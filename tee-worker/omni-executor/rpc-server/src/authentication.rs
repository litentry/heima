use crate::{server::RpcContext, utils::hex::hex_encode};
use executor_core::native_call::NativeCall;
use executor_crypto::hashing::blake2_256;
use executor_storage::{Storage, VerificationCodeStorage};
use parity_scale_codec::{Decode, Encode};
use primitives::{
	signature::HeimaMultiSignature,
	// AccountId,
	Hash,
	Identity,
	OmniAccountAuthType,
	ShardIdentifier,
};
use std::sync::Arc;

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
	// OAuth2Error(String),
	// AuthTokenError(String),
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

pub fn verify_email_authentication(
	ctx: Arc<RpcContext>,
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

pub fn verify_auth_token_authentication(
	_ctx: Arc<RpcContext>,
	_sender_identity: &Identity,
	_auth_token: &str,
) -> Result<(), AuthenticationError> {
	todo!()
}

pub fn verify_oauth2_authentication(
	_ctx: Arc<RpcContext>,
	_sender_identity_hash: Hash,
	_payload: &OAuth2Data,
) -> Result<(), AuthenticationError> {
	// TODO: get OmniAccount from storage
	todo!()
	// match payload.provider {
	// 	OAuth2Provider::Google => {
	// 		verify_google_oauth2(ctx, sender_identity_hash, omni_account, payload).await
	// 	},
	// }
}

// async fn verify_google_oauth2(
// 	_ctx: Arc<RpcContext>,
// 	_sender_identity_hash: Hash,
// 	_omni_account: AccountId,
// 	_payload: &OAuth2Data,
// ) -> Result<(), AuthenticationError> {
// 	todo!()
// }
