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

use crate::key_store::KeyStore;

use oe_crypto::rsa::pkcs1::{DecodeRsaPrivateKey, EncodeRsaPrivateKey};
use oe_crypto::rsa::RsaPrivateKey;

use oe_crypto::shielding_key::ShieldingKey;

pub struct ShieldingKeyStore {
	path: String,
}

impl ShieldingKeyStore {
	pub fn new(path: String) -> Self {
		let store = Self { path: path.clone() };
		let std_path = std::path::Path::new(&path);
		if !std_path.exists() {
			let key: ShieldingKey = Self::generate_key().unwrap();
			// Create store file if not exists.
			if let Some(parent) = std_path.parent() {
				if !parent.exists() {
					std::fs::create_dir_all(parent).unwrap();
				}
			}
			let _file = std::fs::File::create(&path).unwrap();
			store.write(&key).unwrap();
		}

		store
	}
}

impl KeyStore<ShieldingKey> for ShieldingKeyStore {
	fn generate_key() -> Result<ShieldingKey, ()> {
		Ok(ShieldingKey::new())
	}

	fn serialize(k: &ShieldingKey) -> Result<Vec<u8>, ()> {
		let bytes = k.private_key().to_pkcs1_der().unwrap().as_bytes().to_vec();

		Ok(bytes)
	}

	fn deserialize(sealed: Vec<u8>) -> Result<ShieldingKey, ()> {
		let key = RsaPrivateKey::from_pkcs1_der(&sealed).unwrap();
		Ok(ShieldingKey::from(key))
	}

	fn path(&self) -> String {
		self.path.clone()
	}
}
