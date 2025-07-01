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
#[cfg(feature = "std")]
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, PartialEq, Eq, Clone, Debug)]
pub struct EthereumSignature(pub [u8; 65]);

impl TryFrom<&[u8]> for EthereumSignature {
	type Error = ();

	fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
		if data.len() == 65 {
			let mut inner = [0u8; 65];
			inner.copy_from_slice(data);
			Ok(EthereumSignature(inner))
		} else {
			Err(())
		}
	}
}

#[cfg(feature = "std")]
impl Serialize for EthereumSignature {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&hex::encode(self))
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for EthereumSignature {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let signature_hex = hex::decode(&String::deserialize(deserializer)?)
			.map_err(|e| de::Error::custom(format!("{:?}", e)))?;
		EthereumSignature::try_from(signature_hex.as_ref())
			.map_err(|e| de::Error::custom(format!("{:?}", e)))
	}
}

impl AsRef<[u8; 65]> for EthereumSignature {
	fn as_ref(&self) -> &[u8; 65] {
		&self.0
	}
}

impl AsRef<[u8]> for EthereumSignature {
	fn as_ref(&self) -> &[u8] {
		&self.0[..]
	}
}
