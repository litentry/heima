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

use crate::{AuthOptions, BlockNumber};
use alloc::{
	string::{String, ToString},
	vec::Vec,
};
use base64::prelude::{Engine, BASE64_URL_SAFE_NO_PAD};
use ring::hmac;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
pub enum Error {
	InvalidToken,
	InvalidSignature,
	ExpiredToken,
	InvalidSubject,
	Base64DecodeError,
	JsonError,
}

impl From<base64::DecodeError> for Error {
	fn from(_: base64::DecodeError) -> Self {
		Error::Base64DecodeError
	}
}

#[derive(Serialize, Deserialize)]
pub struct Header {
	alg: String,
	typ: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Payload {
	sub: String,
	exp: BlockNumber,
}

impl Payload {
	pub fn new(sub: String, options: AuthOptions) -> Self {
		Self { sub, exp: options.expires_at }
	}
}

pub struct Validation {
	pub sub: String,
	pub current_block: BlockNumber,
}

impl Validation {
	pub fn new(sub: String, current_block: BlockNumber) -> Self {
		Self { sub, current_block }
	}

	pub fn validate(&self, payload: &Payload) -> Result<(), Error> {
		if self.sub != payload.sub {
			return Err(Error::InvalidSubject)
		}

		if self.current_block > payload.exp {
			return Err(Error::ExpiredToken)
		}

		Ok(())
	}
}

fn base64_encode<T: AsRef<[u8]>>(input: T) -> String {
	BASE64_URL_SAFE_NO_PAD.encode(input)
}

fn base64_decode<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, base64::DecodeError> {
	BASE64_URL_SAFE_NO_PAD.decode(input)
}

pub fn create(payload: &Payload, secret: &[u8]) -> Result<String, Error> {
	let header = Header { alg: "HS256".to_string(), typ: "JWT".to_string() };
	let encoded_header =
		base64_encode(&serde_json::to_string(&header).map_err(|_| Error::JsonError)?);
	let encoded_payload =
		base64_encode(&serde_json::to_string(&payload).map_err(|_| Error::JsonError)?);
	let data = [encoded_header, encoded_payload].join(".");
	let key = hmac::Key::new(hmac::HMAC_SHA256, secret);
	let signature = hmac::sign(&key, data.as_bytes());
	let encoded_signature = base64_encode(signature);

	Ok([data, encoded_signature].join("."))
}

pub fn decode(jwt: &str, secret: &[u8], validation: Validation) -> Result<Payload, Error> {
	let parts: Vec<&str> = jwt.split('.').collect();
	if parts.len() != 3 {
		return Err(Error::InvalidToken)
	}

	let data = [parts[0], parts[1]].join(".");
	let key = hmac::Key::new(hmac::HMAC_SHA256, secret);
	let decoded_signature = base64_decode(parts[2])?;

	hmac::verify(&key, data.as_bytes(), &decoded_signature).map_err(|_| Error::InvalidSignature)?;

	let payload = base64_decode(parts[1])?;
	let payload: Payload = serde_json::from_slice(&payload).map_err(|_| Error::JsonError)?;

	validation.validate(&payload)?;

	Ok(payload)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_jwt() {
		let secret = "secret".as_bytes();
		let payload = Payload::new("subject".to_string(), AuthOptions { expires_at: 10 });
		let jwt = create(&payload, secret).unwrap();

		let current_block = 5;
		let decoded_payload =
			decode(&jwt, secret, Validation::new("subject".to_string(), current_block)).unwrap();

		assert_eq!(decoded_payload, payload);
	}

	#[test]
	fn test_jwt_exp_validation() {
		let secret = "secret".as_bytes();
		let payload = Payload::new("subject".to_string(), AuthOptions { expires_at: 10 });
		let jwt = create(&payload, secret).unwrap();

		let current_block = 12;
		let result = decode(&jwt, secret, Validation::new("subject".to_string(), current_block));

		assert_eq!(result, Err(Error::ExpiredToken));
	}

	#[test]
	fn test_jwt_sub_validation() {
		let secret = "secret".as_bytes();
		let payload = Payload::new("subject".to_string(), AuthOptions { expires_at: 10 });
		let jwt = create(&payload, secret).unwrap();

		let current_block = 5;
		let result =
			decode(&jwt, secret, Validation::new("other-subject".to_string(), current_block));

		assert_eq!(result, Err(Error::InvalidSubject));
	}
}
