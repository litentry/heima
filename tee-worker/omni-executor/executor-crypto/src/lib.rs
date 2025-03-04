pub mod aes256;
pub mod jwt;
pub mod secp256k1;
pub mod traits;

pub mod rsa {
	pub use rsa::*;

	use serde::{Deserialize, Serialize};
	use std::vec::Vec;

	#[derive(Serialize, Deserialize)]
	pub struct Rsa3072PubKey {
		pub n: Vec<u8>,
		pub e: Vec<u8>,
	}
}

pub use sp_core::{crypto::Pair as PairTrait, ecdsa, ed25519, hashing, sr25519, ByteArray};
