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
use subxt_signer::sr25519::SecretKeyBytes;

/// Generates and stores keys used by for communication with parentchain`
pub struct SubstrateKeyStore {
	path: String,
}

impl SubstrateKeyStore {
	pub fn new(path: String) -> Self {
		let key = Self::generate_key().unwrap();
		let store = Self { path };
		store.write(&key).unwrap();

		store
	}
}

impl KeyStore<SecretKeyBytes> for SubstrateKeyStore {
	fn generate_key() -> Result<SecretKeyBytes, ()> {
		// Secret Key URI `//Alice` is account:
		// Network ID:        substrate
		// Secret seed:       0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a
		// Public key (hex):  0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
		// Account ID:        0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
		// Public key (SS58): 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
		// SS58 Address:      5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
		Ok([
			229, 190, 154, 80, 146, 184, 27, 202, 100, 190, 129, 210, 18, 231, 242, 249, 235, 161,
			131, 187, 122, 144, 149, 79, 123, 118, 54, 31, 110, 219, 92, 10,
		])
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
