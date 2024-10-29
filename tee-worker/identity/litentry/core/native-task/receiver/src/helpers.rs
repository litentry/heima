use crate::NativeTaskError;
use alloc::vec::Vec;
use codec::Encode;
use itp_types::AccountId;
use litentry_hex_utils::hex_encode;
use litentry_primitives::{Identity, ParentchainIndex, Web3ValidationData};
use sp_core::blake2_256;

// verification message format:
// ```
// blake2_256(<parentchain nonce> + <omni account> + <identity-to-be-added>)
// ```
// where <> means SCALE-encoded
// see https://github.com/litentry/litentry-parachain/issues/1739 and P-174
pub fn get_expected_raw_message(
	who: &AccountId,
	identity: &Identity,
	nonce: ParentchainIndex,
) -> Vec<u8> {
	let mut payload = Vec::new();
	payload.append(&mut nonce.encode());
	payload.append(&mut who.encode());
	payload.append(&mut identity.encode());
	blake2_256(payload.as_slice()).to_vec()
}

// [P-923] Verify a web3 identity
// This function validates the signature with both the raw message and its prettified format. Any of them is valid.
// The prettified version was introduced to extend the support for utf-8 signatures for browser wallets that
// do not play well with raw bytes signing, like Solana's Phantom and OKX wallets.
//
// The prettified format is the raw message with a prefix "Token: ".
pub fn verify_web3_identity(
	identity: &Identity,
	expected_raw_msg: &[u8],
	validation_data: &Web3ValidationData,
) -> Result<(), &'static str> {
	let mut expected_prettified_msg = hex_encode(expected_raw_msg);
	expected_prettified_msg.insert_str(0, "Token: ");

	let expected_prettified_msg = expected_prettified_msg.as_bytes();

	let received_message = validation_data.message().as_slice();

	if expected_raw_msg != received_message || expected_prettified_msg != received_message {
		return Err("UnexpectedMessage")
	}

	let signature = validation_data.signature();

	if !signature.verify(expected_raw_msg, identity)
		|| !signature.verify(expected_prettified_msg, identity)
	{
		return Err("VerifyWeb3SignatureFailed")
	}

	Ok(())
}
