use executor_crypto::jwt;
use executor_primitives::BlockNumber;
use parity_scale_codec::{Decode, Encode};
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

pub const AUTH_TOKEN_EXPIRATION: u32 = 50_400; // 1 week in blocks

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct AuthOptions {
	pub expires_at: BlockNumber,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct AuthTokenClaims {
	sub: String,
	pub exp: BlockNumber,
}

impl AuthTokenClaims {
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

	pub fn validate(&self, claims: &AuthTokenClaims) -> Result<(), Error> {
		if self.sub != claims.sub {
			return Err(Error::InvalidSubject);
		}

		if self.current_block > claims.exp {
			return Err(Error::ExpiredToken);
		}

		Ok(())
	}
}

pub trait AuthTokenValidator {
	fn validate(&self, secret: &[u8], validation: Validation) -> Result<(), Error>;
}

impl AuthTokenValidator for String {
	fn validate(&self, secret: &[u8], validation: Validation) -> Result<(), Error> {
		jwt::decode::<AuthTokenClaims>(self, secret)
			.map_err(|_| Error::InvalidToken)
			.and_then(|claims| validation.validate(&claims))
	}
}

impl AuthTokenValidator for &str {
	fn validate(&self, secret: &[u8], validation: Validation) -> Result<(), Error> {
		jwt::decode::<AuthTokenClaims>(self, secret)
			.map_err(|_| Error::InvalidToken)
			.and_then(|claims| validation.validate(&claims))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_auth_token() {
		let private_key = include_bytes!("../test_private_key.pem");
		let claims = AuthTokenClaims::new("test".to_string(), AuthOptions { expires_at: 100 });
		let token = jwt::create(&claims, private_key).unwrap();

		let current_block = 50;
		let validation = Validation::new("test".to_string(), current_block);
		let public_key = include_bytes!("../test_public_key.pem");
		let result = token.validate(public_key, validation);

		assert_eq!(result, Ok(()));
	}

	#[test]
	fn test_auth_token_expired() {
		let private_key = include_bytes!("../test_private_key.pem");
		let claims = AuthTokenClaims::new("test".to_string(), AuthOptions { expires_at: 100 });
		let token = jwt::create(&claims, private_key).unwrap();

		let current_block = 150;
		let validation = Validation::new("test".to_string(), current_block);
		let public_key = include_bytes!("../test_public_key.pem");
		let result = token.validate(public_key, validation);

		assert_eq!(result, Err(Error::ExpiredToken));
	}

	#[test]
	fn test_auth_token_invalid_subject() {
		let private_key = include_bytes!("../test_private_key.pem");
		let claims = AuthTokenClaims::new("test".to_string(), AuthOptions { expires_at: 100 });
		let token = jwt::create(&claims, private_key).unwrap();

		let current_block = 50;
		let validation = Validation::new("invalid-sub".to_string(), current_block);
		let public_key = include_bytes!("../test_public_key.pem");
		let result = token.validate(public_key, validation);

		assert_eq!(result, Err(Error::InvalidSubject));
	}
}
