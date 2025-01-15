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

use hex::FromHexError;
use parity_scale_codec::{Decode, Encode, Error as CodecError};
use std::{string::String, vec::Vec};

#[derive(Debug)]
pub enum Error {
	Hex(FromHexError),
	Codec(CodecError),
}

/// Trait to encode a given value to a hex string, prefixed with "0x".
pub trait ToHexPrefixed {
	fn to_hex(&self) -> String;
}

impl<T: Encode> ToHexPrefixed for T {
	fn to_hex(&self) -> String {
		hex_encode(&self.encode())
	}
}

/// Trait to decode a hex string to a given output.
pub trait FromHexPrefixed {
	type Output;

	fn from_hex(msg: &str) -> Result<Self::Output, Error>;
}

impl<T: Decode> FromHexPrefixed for T {
	type Output = T;

	fn from_hex(msg: &str) -> Result<Self::Output, Error> {
		let byte_array = decode_hex(msg).map_err(Error::Hex)?;
		Decode::decode(&mut byte_array.as_slice()).map_err(Error::Codec)
	}
}

/// Hex encodes given data and preappends a "0x".
pub fn hex_encode(data: &[u8]) -> String {
	let mut hex_str = hex::encode(data);
	hex_str.insert_str(0, "0x");
	hex_str
}

/// Helper method for decoding hex.
pub fn decode_hex<T: AsRef<[u8]>>(message: T) -> Result<Vec<u8>, FromHexError> {
	let message = message.as_ref();
	let message = match message {
		[b'0', b'x', hex_value @ ..] => hex_value,
		_ => message,
	};

	let decoded_message = hex::decode(message)?;
	Ok(decoded_message)
}

#[cfg(test)]
mod tests {
	use super::*;
	use parity_scale_codec::{Decode, Encode};
	use std::string::ToString;

	#[test]
	fn hex_encode_decode_works() {
		let data = "Hello World!".to_string();

		let hex_encoded_data = hex_encode(&data.encode());
		let decoded_data =
			String::decode(&mut decode_hex(hex_encoded_data).unwrap().as_slice()).unwrap();

		assert_eq!(data, decoded_data);
	}

	#[test]
	fn hex_encode_decode_works_empty_input() {
		let data = String::new();

		let hex_encoded_data = hex_encode(&data.encode());
		let decoded_data =
			String::decode(&mut decode_hex(hex_encoded_data).unwrap().as_slice()).unwrap();

		assert_eq!(data, decoded_data);
	}

	#[test]
	fn to_hex_from_hex_works() {
		let data = "Hello World!".to_string();

		let hex_encoded_data = data.to_hex();
		let decoded_data = String::from_hex(&hex_encoded_data).unwrap();

		assert_eq!(data, decoded_data);
	}
}
