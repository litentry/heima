use executor_crypto::jwt;
use parity_scale_codec::{Decode, Encode};
use primitives::BlockNumber;
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

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct AuthOptions {
	expires_at: BlockNumber,
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
		let secret = b"secret";
		let claims = AuthTokenClaims::new("test".to_string(), AuthOptions { expires_at: 100 });
		let token = jwt::create(&claims, secret).unwrap();

		let current_block = 50;
		let validation = Validation::new("test".to_string(), current_block);
		let result = token.validate(secret, validation);

		assert_eq!(result, Ok(()));
	}

	#[test]
	fn test_auth_token_expired() {
		let secret = b"secret";
		let claims = AuthTokenClaims::new("test".to_string(), AuthOptions { expires_at: 100 });
		let token = jwt::create(&claims, secret).unwrap();

		let current_block = 150;
		let validation = Validation::new("test".to_string(), current_block);
		let result = token.validate(secret, validation);

		assert_eq!(result, Err(Error::ExpiredToken));
	}

	#[test]
	fn test_auth_token_invalid_subject() {
		let secret = b"secret";
		let claims = AuthTokenClaims::new("test".to_string(), AuthOptions { expires_at: 100 });
		let token = jwt::create(&claims, secret).unwrap();

		let current_block = 50;
		let validation = Validation::new("invalid-sub".to_string(), current_block);
		let result = token.validate(secret, validation);

		assert_eq!(result, Err(Error::InvalidSubject));
	}
}
