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

use crate::{
	rsa::{
		errors::Error as RsaError, rand_core::OsRng, sha2::Sha256, Oaep, RsaPrivateKey,
		RsaPublicKey,
	},
	traits::Decrypt,
};

#[derive(Debug, Clone)]
pub struct ShieldingKey {
	key: RsaPrivateKey,
}

impl ShieldingKey {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn from(key: RsaPrivateKey) -> Self {
		Self { key }
	}

	pub fn public_key(&self) -> RsaPublicKey {
		self.key.to_public_key()
	}

	pub fn private_key(&self) -> &RsaPrivateKey {
		&self.key
	}
}

impl Decrypt for ShieldingKey {
	type Error = RsaError;

	fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Self::Error> {
		self.private_key().decrypt(Oaep::new::<Sha256>(), data)
	}
}

impl Default for ShieldingKey {
	fn default() -> Self {
		let mut rng = OsRng;
		let key = RsaPrivateKey::new(&mut rng, 3072).expect("Failed to generate RSA key");
		Self { key }
	}
}
