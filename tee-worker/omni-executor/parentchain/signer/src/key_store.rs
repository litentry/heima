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

use executor_core::key_store::KeyStore;
use executor_crypto::{sr25519, PairTrait};
use subxt_signer::sr25519::SecretKeyBytes;

/// Generates and stores keys used by for communication with parentchain`
pub struct SubstrateKeyStore {
	path: String,
}

impl SubstrateKeyStore {
	pub fn new(path: String) -> Self {
		if std::path::Path::new(&path).exists() {
			Self { path }
		} else {
			let key = Self::generate_key().unwrap();
			let store = Self { path };
			store.write(&key).unwrap();
			store
		}
	}
}

impl KeyStore<SecretKeyBytes> for SubstrateKeyStore {
	fn generate_key() -> Result<SecretKeyBytes, ()> {
		Ok(sr25519::Pair::generate().1)
	}

	fn serialize(k: &SecretKeyBytes) -> Result<Vec<u8>, ()> {
		Ok(Vec::from(k))
	}

	fn deserialize(sealed: Vec<u8>) -> Result<SecretKeyBytes, ()> {
		sealed.as_slice().try_into().map_err(|_| ())
	}

	fn path(&self) -> String {
		self.path.clone()
	}
}
