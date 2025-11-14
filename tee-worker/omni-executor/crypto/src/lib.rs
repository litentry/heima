pub mod aes256;
pub mod jwt;
pub mod passkey;
pub mod secp256k1;
pub mod shielding_key;
pub mod traits;

pub mod rsa {
	pub use rsa::*;

	use ethers::types::Bytes;
	use serde::{Deserialize, Serialize};
	use std::vec::Vec;

	#[derive(Debug, Serialize, Deserialize)]
	pub struct Rsa3072PubKey {
		pub n: Vec<u8>,
		pub e: Vec<u8>,
	}

	#[derive(Clone, Serialize, Deserialize)]
	pub struct SerdeRsa3072PubKey {
		pub n: Bytes,
		pub e: Bytes,
	}

	impl From<Rsa3072PubKey> for SerdeRsa3072PubKey {
		fn from(k: Rsa3072PubKey) -> Self {
			Self { n: k.n.into(), e: k.e.into() }
		}
	}
}

pub use sp_core::{crypto::Pair as PairTrait, ecdsa, ed25519, hashing, sr25519, ByteArray};
