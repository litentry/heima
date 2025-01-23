use crate::server::RpcContext;
use crypto::hashing::blake2_256;
use executor_core::native_call::NativeCall;
use heima_authentication::auth_token::{AuthTokenValidator, Validation};
use parentchain_rpc_client::SubstrateRpcClient;
use parity_scale_codec::{Decode, Encode};
use primitives::{
	signature::HeimaMultiSignature,
	utils::hex::{hex_encode, ToHexPrefixed},
	// AccountId,
	Hash,
	Identity,
	OmniAccountAuthType,
	ShardIdentifier,
};
use std::sync::Arc;
use storage::{MemberOmniAccountStorage, Storage, VerificationCodeStorage};
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
	// OAuth2Error(String),
	#[allow(dead_code)]
	AuthTokenError(AuthTokenError),
}

#[derive(Debug)]
pub enum AuthTokenError {
	InvalidToken,
	OmniAccountNotFound,
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
	ctx: Arc<RpcContext>,
	handle: Handle,
	sender_identity: &Identity,
	auth_token: &str,
) -> Result<(), AuthenticationError> {
	let rpc_client = ctx.rpc_client.clone();
	let current_block = handle
		.block_on(async {
			rpc_client.get_last_finalized_block_num().await.map_err(|e| {
				log::error!("Could not get last finalized block number: {:?}", e);
			})
		})
		.map_err(|_| AuthenticationError::AuthTokenError(AuthTokenError::BlockNumberError))?;
	let member_omni_account_storage = MemberOmniAccountStorage::new(ctx.storage_db.clone());
	let Some(omni_account) = member_omni_account_storage.get(&sender_identity.hash()) else {
		return Err(AuthenticationError::AuthTokenError(AuthTokenError::OmniAccountNotFound));
	};
	let validation = Validation::new(omni_account.to_hex(), current_block);

	if auth_token.validate(ctx.jwt_secret.as_bytes(), validation).is_err() {
		return Err(AuthenticationError::AuthTokenError(AuthTokenError::InvalidToken));
	}

	Ok(())
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
