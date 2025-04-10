pub mod aes256;
pub mod jwt;
pub mod secp256k1;
pub mod traits;

pub mod rsa {
	pub use rsa::*;

	use serde::{Deserialize, Serialize};
	use serde_with::serde_as;
	use std::vec::Vec;

	#[serde_as]
	#[derive(Serialize, Deserialize, Debug)]
	pub struct Rsa3072PubKey {
		#[serde_as(as = "serde_with::hex::Hex")]
		pub n: Vec<u8>,
		#[serde_as(as = "serde_with::hex::Hex")]
		pub e: Vec<u8>,
	}
}

pub use sp_core::{crypto::Pair as PairTrait, ecdsa, ed25519, hashing, sr25519, ByteArray};
