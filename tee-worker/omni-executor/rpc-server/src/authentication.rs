use crate::{server::RpcContext, utils::hex::hex_encode};
use crypto::hashing::blake2_256;
use native_call_executor::NativeCall;
use parentchain_primitives::{
	signature::HeimaMultiSignature, AccountId, BlockNumber, Hash, Identity, ShardIdentifier,
};
use parity_scale_codec::{Decode, Encode};
use std::sync::Arc;

pub type VerificationCode = String;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum Authentication {
	Web3(HeimaMultiSignature),
	Email(VerificationCode),
	AuthToken(String),
	// OAuth2(OAuth2Data),
}

#[derive(Debug)]
pub enum AuthenticationError {
	Web3InvalidSignature,
	EmailVerificationCodeNotFound,
	EmailInvalidVerificationCode,
	AuthTokenError(String),
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

pub async fn verify_email_authentication(
	ctx: Arc<RpcContext>,
	sender_identity: &Identity,
	verification_code: &VerificationCode,
) -> Result<(), AuthenticationError> {
	todo!()
}

pub async fn verify_auth_token_authentication(
	ctx: Arc<RpcContext>,
	sender_identity: &Identity,
	auth_token: &str,
) -> Result<(), AuthenticationError> {
	todo!()
}
