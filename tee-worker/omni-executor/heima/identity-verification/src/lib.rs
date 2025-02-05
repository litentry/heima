mod helpers;
pub mod web2;

use executor_crypto::hashing::blake2_256;
use executor_primitives::{utils::hex::hex_encode, Identity, Web3ValidationData};
use parity_scale_codec::Encode;

// verification message format:
// ```
// blake2_256(<parentchain nonce> + <sender identity> + <identity-to-be-added>)
// ```
// where <> means SCALE-encoded
pub fn get_expected_raw_message(
	member: &Identity,
	identity_to_add: &Identity,
	omni_account_nonce: u64,
) -> Vec<u8> {
	let mut payload = Vec::new();
	payload.append(&mut omni_account_nonce.encode());
	payload.append(&mut member.encode());
	payload.append(&mut identity_to_add.encode());
	blake2_256(payload.as_slice()).to_vec()
}

// This function validates the signature with both the raw message and its prettified format. Any of them is valid.
// The prettified version was introduced to extend the support for utf-8 signatures for browser wallets that
// do not play well with raw bytes signing, like Solana's Phantom and OKX wallets.
//
// The prettified format is the raw message with a prefix "Token: ".
pub fn verify_web3_identity(
	identity: &Identity,
	expected_raw_msg: &[u8],
	validation_data: &Web3ValidationData,
) -> Result<(), ()> {
	let mut expected_prettified_msg = hex_encode(expected_raw_msg);
	expected_prettified_msg.insert_str(0, "Token: ");

	let expected_prettified_msg = expected_prettified_msg.as_bytes();

	let received_message = validation_data.message().as_slice();

	if expected_raw_msg != received_message && expected_prettified_msg != received_message {
		return Err(());
	}

	let signature = validation_data.signature();

	if !signature.verify(expected_raw_msg, identity)
		&& !signature.verify(expected_prettified_msg, identity)
	{
		return Err(());
	}

	Ok(())
}
