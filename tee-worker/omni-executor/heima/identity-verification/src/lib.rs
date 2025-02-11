mod helpers;
pub mod web2;
pub mod web3;

use executor_crypto::hashing::blake2_256;
use executor_primitives::Identity;
use parity_scale_codec::Encode;

// verification message format:
// ```
// blake2_256(<parentchain nonce> + <sender identity> + <identity-to-be-added>)
// ```
// where <> means SCALE-encoded
pub fn get_verification_message(
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
