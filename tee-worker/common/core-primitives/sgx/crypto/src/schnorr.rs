// Copyright 2020-2024 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.
#[cfg(feature = "sgx")]
pub use sgx::*;

use crate::error::{Error, Result};
use k256::{elliptic_curve::group::GroupEncoding, schnorr::SigningKey, PublicKey};

/// File name of the sealed seed file.
pub const SEALED_SIGNER_SEED_FILE: &str = "schnorr_key_sealed.bin";

#[derive(Clone)]
pub struct Pair {
	pub public: PublicKey,
	private: SigningKey,
}

impl Pair {
	pub fn new(private: SigningKey) -> Self {
		let public = PublicKey::from(private.verifying_key());
		Self { private, public }
	}

	pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
		let private_key =
			SigningKey::from_bytes(bytes).map_err(|e| Error::Other(format!("{:?}", e).into()))?;
		Ok(Self::new(private_key))
	}

	pub fn public_bytes(&self) -> [u8; 33] {
		// safe to unwrap here
		self.public.as_affine().to_bytes().as_slice().try_into().unwrap()
	}

	pub fn private_bytes(&self) -> [u8; 32] {
		// safe to unwrap here
		self.private.to_bytes().as_slice().try_into().unwrap()
	}
}

#[cfg(feature = "sgx")]
pub mod sgx {
	use super::SEALED_SIGNER_SEED_FILE;
	use crate::{
		error::{Error, Result},
		key_repository::KeyRepository,
		schnorr::Pair,
		std::string::ToString,
	};
	use itp_sgx_io::{seal, unseal, SealedIO};
	use k256::schnorr::SigningKey;
	use log::*;
	use sgx_rand::{Rng, StdRng};
	use std::{path::PathBuf, string::String};

	/// Creates a repository for schnorr keypair and initializes
	/// a fresh private key if it doesn't exist at `path`.
	pub fn create_schnorr_repository(
		path: PathBuf,
		key_file_prefix: &str,
		key: Option<[u8; 32]>,
	) -> Result<KeyRepository<Pair, Seal>> {
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
		fn unseal_pair(&self) -> Result<Pair> {
			self.unseal()
		}

		pub fn exists(&self) -> bool {
			self.path().exists()
		}

		pub fn init(&self, key: Option<[u8; 32]>) -> Result<Pair> {
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
					let mut seed = [0u8; 32];
					let mut rand = StdRng::new()?;
					rand.fill_bytes(&mut seed);
					seed
				};
				seal(&key, self.path())?;
			}
			self.unseal_pair()
		}
	}

	impl SealedIO for Seal {
		type Error = Error;
		type Unsealed = Pair;

		fn unseal(&self) -> Result<Self::Unsealed> {
			let raw = unseal(self.path())?;
			let secret = SigningKey::from_bytes(&raw)
				.map_err(|e| Error::Other(format!("{:?}", e).into()))?;
			Ok(Pair::new(secret))
		}

		fn seal(&self, unsealed: &Self::Unsealed) -> Result<()> {
			let raw = unsealed.private.to_bytes();
			seal(&raw, self.path()).map_err(|e| e.into())
		}
	}
}

#[cfg(feature = "test")]
pub mod sgx_tests {
	use crate::{
		key_repository::AccessKey,
		schnorr::{create_schnorr_repository, Pair, Seal},
		std::string::ToString,
	};
	use itp_sgx_temp_dir::TempDir;
	use std::path::PathBuf;

	static PRIVATE_KEY: &str = "189ab2ba2ace8ee33cb578c200766628e24083c5996441ba50097f200b9ea7d2";
	static PUBLIC_KEY: &str = "027e82fc627f33650f8be418b52c35e304d706956afc4c9e4341248783319d8d1c";

	pub fn schnorr_creating_repository_with_same_path_and_prefix_results_in_same_key() {
		//given
		let key_file_prefix = "test";
		fn get_key_from_repo(path: PathBuf, prefix: &str) -> Pair {
			create_schnorr_repository(path, prefix, None).unwrap().retrieve_key().unwrap()
		}
		let temp_dir = TempDir::with_prefix(
			"schnorr_creating_repository_with_same_path_and_prefix_results_in_same_key",
		)
		.unwrap();
		let temp_path = temp_dir.path().to_path_buf();

		//when
		let first_key = get_key_from_repo(temp_path.clone(), key_file_prefix);
		let second_key = get_key_from_repo(temp_path.clone(), key_file_prefix);

		//then
		assert_eq!(first_key.public, second_key.public);
	}

	pub fn schnorr_creating_repository_with_same_path_and_prefix_but_new_key_results_in_new_key() {
		//given
		let key_file_prefix = "test";
		fn get_key_from_repo(path: PathBuf, prefix: &str, key: Option<[u8; 32]>) -> Pair {
			create_schnorr_repository(path, prefix, key).unwrap().retrieve_key().unwrap()
		}
		let temp_dir = TempDir::with_prefix(
			"schnorr_creating_repository_with_same_path_and_prefix_but_new_key_results_in_new_key",
		)
		.unwrap();
		let temp_path = temp_dir.path().to_path_buf();
		let new_key: [u8; 32] = hex::decode(PRIVATE_KEY).unwrap().try_into().unwrap();

		//when
		let first_key = get_key_from_repo(temp_path.clone(), key_file_prefix, None);
		let second_key = get_key_from_repo(temp_path.clone(), key_file_prefix, Some(new_key));

		//then
		assert_ne!(first_key.public, second_key.public);
		assert_eq!(hex::encode(second_key.public_bytes()), PUBLIC_KEY)
	}

	pub fn schnorr_seal_init_should_create_new_key_if_not_present() {
		//given
		let temp_dir =
			TempDir::with_prefix("schnorr_seal_init_should_create_new_key_if_not_present").unwrap();
		let seal = Seal::new(temp_dir.path().to_path_buf(), "test".to_string());
		assert!(!seal.exists());

		//when
		seal.init(None).unwrap();

		//then
		assert!(seal.exists());
	}

	pub fn schnorr_seal_init_should_seal_provided_key() {
		//given
		let temp_dir = TempDir::with_prefix("schnorr_seal_init_should_seal_provided_key").unwrap();
		let seal = Seal::new(temp_dir.path().to_path_buf(), "test".to_string());
		assert!(!seal.exists());
		let new_key: [u8; 32] = hex::decode(PRIVATE_KEY).unwrap().try_into().unwrap();

		//when
		let pair = seal.init(Some(new_key)).unwrap();

		//then
		assert!(seal.exists());
		assert_eq!(hex::encode(pair.public_bytes()), PUBLIC_KEY)
	}

	pub fn schnorr_seal_init_should_not_change_key_if_exists_and_not_provided() {
		//given
		let temp_dir = TempDir::with_prefix(
			"schnorr_seal_init_should_not_change_key_if_exists_and_not_provided",
		)
		.unwrap();
		let seal = Seal::new(temp_dir.path().to_path_buf(), "test".to_string());
		let pair = seal.init(None).unwrap();

		//when
		let new_pair = seal.init(None).unwrap();

		//then
		assert_eq!(pair.public, new_pair.public);
	}

	pub fn schnorr_seal_init_with_key_should_change_key_current_key() {
		//given
		let temp_dir =
			TempDir::with_prefix("schnorr_seal_init_with_key_should_change_key_current_key")
				.unwrap();
		let seal = Seal::new(temp_dir.path().to_path_buf(), "test".to_string());
		let _ = seal.init(None).unwrap();
		let new_key: [u8; 32] = hex::decode(PRIVATE_KEY).unwrap().try_into().unwrap();

		//when
		let new_pair = seal.init(Some(new_key)).unwrap();

		//then
		assert_eq!(hex::encode(new_pair.public_bytes()), PUBLIC_KEY)
	}
}
