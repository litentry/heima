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

use executor_crypto::{
	aes256::{aes_decrypt, Aes256Key as RequestAesKey, AesOutput},
	traits::Decrypt,
};
use executor_primitives::MrEnclave;
use parity_scale_codec::{Decode, Encode};
use std::fmt::Debug;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct PlainRequest {
	pub mrenclave: MrEnclave,
	pub payload: Vec<u8>,
}

// Represent a request that can be decrypted by the enclave
pub trait DecryptableRequest {
	type Error;
	// the mrenclave getter
	fn mrenclave(&self) -> MrEnclave;
	// how to decrypt the payload
	fn decrypt<T: Debug>(
		&mut self,
		shielding_key: Box<dyn Decrypt<Error = T>>,
	) -> Result<Vec<u8>, Self::Error>;
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct AesRequest {
	pub mrenclave: MrEnclave,
	pub key: Vec<u8>,
	pub payload: AesOutput,
}

impl DecryptableRequest for AesRequest {
	type Error = ();

	fn mrenclave(&self) -> MrEnclave {
		self.mrenclave
	}

	fn decrypt<T: Debug>(
		&mut self,
		enclave_shielding_key: Box<dyn Decrypt<Error = T>>,
	) -> core::result::Result<Vec<u8>, ()> {
		let aes_key: RequestAesKey = self.decrypt_aes_key(enclave_shielding_key)?;
		aes_decrypt(&aes_key, &mut self.payload).ok_or(())
	}
}

impl AesRequest {
	#[allow(clippy::result_unit_err)]
	pub fn decrypt_aes_key<T: Debug>(
		&mut self,
		enclave_shielding_key: Box<dyn Decrypt<Error = T>>,
	) -> core::result::Result<RequestAesKey, ()> {
		enclave_shielding_key
			.decrypt(&self.key)
			.map_err(|_| ())?
			.try_into()
			.map_err(|_| ())
	}
}
