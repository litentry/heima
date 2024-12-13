/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG

	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.

*/
#[cfg(all(not(feature = "std"), feature = "sgx"))]
use crate::sgx_reexport_prelude::*;

use codec::{Decode, Encode};
use rand::Rng;
pub use ring::aead::{MAX_TAG_LEN, NONCE_LEN};
use ring::{
	aead::{Aad, BoundKey, LessSafeKey, Nonce, NonceSequence, SealingKey, UnboundKey, AES_256_GCM},
	error::Unspecified,
};
use std::vec::Vec;

// we use 256-bit AES-GCM as request enc/dec key
pub const AES_KEY_LEN: usize = 32;

pub type Aes256Key = [u8; AES_KEY_LEN];
pub type Aes256KeyNonce = [u8; NONCE_LEN];

/// File name of the sealed seed file.
pub const SEALED_SIGNER_SEED_FILE: &str = "aes256_key_sealed.bin";

// all-in-one struct containing the encrypted ciphertext with other
// metadata that is required for decryption
//
// by default a postfix tag is used => last 16 bytes of ciphertext is MAC tag
#[derive(Debug, Default, Clone, Eq, PartialEq, Encode, Decode)]
pub struct AesOutput {
	pub ciphertext: Vec<u8>,
	pub aad: Vec<u8>,
	pub nonce: Aes256KeyNonce, // IV
}

// Returns the default if any error happens
// We don't propagate the error to upper level as this function is used in too many places,
// it's too verbose to handle them all and pass back to the parentchain as events.
// We rely on the parentchain event consumers to handle them correctly (and they kind of
// have to, because they'll find all fields are 0)
pub fn aes_encrypt_default(key: &Aes256Key, data: &[u8]) -> AesOutput {
	let mut in_out = data.to_vec();

	let mut nonce = RingAeadNonceSequence::new();
	if nonce.advance().is_ok() {
		let aad = b"";
		if let Ok(unbound_key) = UnboundKey::new(&AES_256_GCM, key.as_slice()) {
			let mut sealing_key = SealingKey::new(unbound_key, nonce.clone());
			if sealing_key.seal_in_place_append_tag(Aad::from(aad), &mut in_out).is_ok() {
				return AesOutput {
					ciphertext: in_out.to_vec(),
					aad: aad.to_vec(),
					nonce: nonce.nonce,
				}
			}
		}
	}

	AesOutput::default()
}

// use LessSafeKey::seal_in_place_append_tag to encrypt the data using the given nonce
// don't be scared by the name, it's similar to `SealingKey::seal_in_place_append_tag`,
// except that it accepts an arbitrary nonce.
// It's only used by the one-off verification message calculation.
pub fn aes_encrypt_nonce(key: &Aes256Key, data: &[u8], nonce: Aes256KeyNonce) -> AesOutput {
	let mut in_out = data.to_vec();
	let aad = b"";
	if let Ok(unbound_key) = UnboundKey::new(&AES_256_GCM, key.as_slice()) {
		let less_safe_key = LessSafeKey::new(unbound_key);
		if less_safe_key
			.seal_in_place_append_tag(
				Nonce::assume_unique_for_key(nonce),
				Aad::from(aad),
				&mut in_out,
			)
			.is_ok()
		{
			return AesOutput { ciphertext: in_out.to_vec(), aad: aad.to_vec(), nonce }
		}
	}

	AesOutput::default()
}

pub fn aes_decrypt(key: &Aes256Key, data: &mut AesOutput) -> Option<Vec<u8>> {
	let in_out = data.ciphertext.as_mut();
	if let Ok(unbound_key) = UnboundKey::new(&AES_256_GCM, key.as_slice()) {
		let less_safe_key = LessSafeKey::new(unbound_key);
		return less_safe_key
			.open_in_place(
				Nonce::assume_unique_for_key(data.nonce),
				Aad::from(data.aad.clone()),
				in_out,
			)
			.ok()
			.map(|data| data.to_vec())
	}
	None
}

#[derive(Clone)]
struct RingAeadNonceSequence {
	pub nonce: Aes256KeyNonce,
}

impl RingAeadNonceSequence {
	fn new() -> RingAeadNonceSequence {
		RingAeadNonceSequence { nonce: [0u8; NONCE_LEN] }
	}
}

impl NonceSequence for RingAeadNonceSequence {
	fn advance(&mut self) -> core::result::Result<Nonce, Unspecified> {
		let nonce = Nonce::assume_unique_for_key(self.nonce);
		let nonce_vec = rand::thread_rng().gen::<Aes256KeyNonce>();
		self.nonce.copy_from_slice(&nonce_vec[0..NONCE_LEN]);
		Ok(nonce)
	}
}

#[cfg(feature = "sgx")]
pub mod sgx {
	use super::*;
	use crate::{
		error::{Error, Result},
		key_repository::KeyRepository,
	};
	use itp_sgx_io::{seal, unseal, SealedIO};
	use log::*;
	use sgx_rand::{Rng, StdRng};
	use std::{
		path::PathBuf,
		sgxfs::SgxFile,
		string::{String, ToString},
	};

	/// Gets a repository for an Rsa3072 keypair and initializes
	/// a fresh key pair if it doesn't exist at `path`.
	pub fn create_aes256_repository(
		path: PathBuf,
		key_file_prefix: &str,
		key: Option<Aes256Key>,
	) -> Result<KeyRepository<Aes256Key, Seal>> {
		let seal = Seal::new(path, key_file_prefix.to_string());
		Ok(KeyRepository::new(seal.init(key)?, seal.into()))
	}

	#[derive(Clone, Debug)]
	pub struct Seal {
		base_path: PathBuf,
		key_file_prefix: String,
	}

	impl Seal {
		pub fn new(base_path: PathBuf, key_file_prefix: String) -> Self {
			Self { base_path, key_file_prefix }
		}

		pub fn path(&self) -> PathBuf {
			self.base_path
				.join(self.key_file_prefix.clone() + "_" + SEALED_SIGNER_SEED_FILE)
		}
	}

	impl Seal {
		fn unseal_key(&self) -> Result<Aes256Key> {
			self.unseal()
		}

		pub fn exists(&self) -> bool {
			SgxFile::open(self.path()).is_ok()
		}

		pub fn init(&self, key: Option<Aes256Key>) -> Result<Aes256Key> {
			if !self.exists() || key.is_some() {
				if !self.exists() {
					info!("Keyfile not found, creating new! {}", self.path().display());
				}
				if key.is_some() {
					info!("New key provided, it will be sealed!");
				}
				let key = if let Some(key) = key {
					key
				} else {
					let mut seed = Aes256Key::default();
					let mut rand = StdRng::new()?;
					rand.fill_bytes(&mut seed);
					seed
				};
				seal(&key, self.path())?;
			}
			self.unseal_key()
		}
	}

	impl SealedIO for Seal {
		type Error = Error;
		type Unsealed = Aes256Key;

		fn unseal(&self) -> Result<Self::Unsealed> {
			Ok(unseal(self.path()).map(|b| Decode::decode(&mut b.as_slice()))??)
		}

		fn seal(&self, unsealed: &Self::Unsealed) -> Result<()> {
			Ok(unsealed.using_encoded(|bytes| seal(bytes, self.path()))?)
		}
	}
}

#[cfg(feature = "test")]
pub mod sgx_tests {
	use super::sgx::*;
	use crate::key_repository::AccessKey;
	use itp_sgx_temp_dir::TempDir;

	pub fn aes256_creating_repository_with_same_path_and_prefix_results_in_same_key() {
		let prefix = "test";
		let temp_dir = TempDir::with_prefix(
			"aes256_creating_repository_with_same_path_and_prefix_results_in_same_key",
		)
		.unwrap();
		let temp_path = temp_dir.path().to_path_buf();
		let key1 = create_aes256_repository(temp_path.clone(), prefix, None)
			.unwrap()
			.retrieve_key()
			.unwrap();
		let key2 = create_aes256_repository(temp_path, prefix, None)
			.unwrap()
			.retrieve_key()
			.unwrap();
		assert_eq!(key1, key2);
	}

	pub fn aes256_creating_repository_with_same_path_and_prefix_but_new_key_results_in_new_key() {
		let prefix = "test";
		let temp_dir = TempDir::with_prefix(
			"aes256_creating_repository_with_same_path_and_prefix_but_new_key_results_in_new_key",
		)
		.unwrap();
		let temp_path = temp_dir.path().to_path_buf();

		let new_key: [u8; 32] = [1u8; 32];
		let first_key = create_aes256_repository(temp_path.clone(), prefix, None)
			.unwrap()
			.retrieve_key()
			.unwrap();
		let second_key = create_aes256_repository(temp_path, prefix, Some(new_key))
			.unwrap()
			.retrieve_key()
			.unwrap();

		assert_ne!(first_key, second_key);
		assert_eq!(second_key, new_key);
	}
}
