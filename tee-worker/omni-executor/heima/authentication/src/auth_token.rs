use executor_crypto::jwt;
use executor_primitives::BlockNumber;
use parity_scale_codec::{Decode, Encode};
use rsa::{
	pkcs1::{DecodeRsaPrivateKey, EncodeRsaPublicKey},
	RsaPrivateKey,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
pub enum Error {
	InvalidToken,
	InvalidSignature,
	ExpiredToken,
	InvalidSubject,
	Base64DecodeError,
	JsonError,
	InternalError,
}

pub const AUTH_TOKEN_EXPIRATION: u32 = 50_400; // 1 week in blocks
pub const AUTH_TOKEN_SESSION_TYPE: &str = "session_token";
pub const AUTH_TOKEN_TRADE_TYPE: &str = "trade_token"; // Used by pumpx

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct AuthOptions {
	pub expires_at: BlockNumber,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct AuthTokenClaims {
	sub: String,
	typ: String,
	pub exp: BlockNumber,
}

impl AuthTokenClaims {
	pub fn new(sub: String, typ: String, options: AuthOptions) -> Self {
		Self { sub, typ, exp: options.expires_at }
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
	fn validate(&self, private_key: &[u8], validation: Validation) -> Result<(), Error> {
		let rsa_private_key =
			RsaPrivateKey::from_pkcs1_der(private_key).map_err(|_| Error::InternalError)?;
		let public_key = rsa_private_key
			.to_public_key()
			.to_pkcs1_der()
			.map_err(|_| Error::InternalError)?;
		jwt::decode::<AuthTokenClaims>(self, public_key.as_bytes())
			.map_err(|_| Error::InvalidToken)
			.and_then(|claims| validation.validate(&claims))
	}
}

impl AuthTokenValidator for &str {
	fn validate(&self, private_key: &[u8], validation: Validation) -> Result<(), Error> {
		let rsa_private_key =
			RsaPrivateKey::from_pkcs1_der(private_key).map_err(|_| Error::InternalError)?;
		let public_key = rsa_private_key
			.to_public_key()
			.to_pkcs1_der()
			.map_err(|_| Error::InternalError)?;
		jwt::decode::<AuthTokenClaims>(self, public_key.as_bytes())
			.map_err(|_| Error::InvalidToken)
			.and_then(|claims| validation.validate(&claims))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use rsa::{pkcs1::EncodeRsaPrivateKey, RsaPrivateKey};

	#[test]
	fn test_auth_token() {
		let mut rng = rand::thread_rng();
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let private_key = rsa_private_key.to_pkcs1_der().unwrap();

		let claims = AuthTokenClaims::new(
			"test".to_string(),
			AUTH_TOKEN_SESSION_TYPE.to_string(),
			AuthOptions { expires_at: 100 },
		);
		let token = jwt::create(&claims, private_key.as_bytes()).unwrap();

		let current_block = 50;
		let validation = Validation::new("test".to_string(), current_block);
		let result = token.validate(private_key.as_bytes(), validation);

		assert_eq!(result, Ok(()));
	}

	#[test]
	fn test_auth_token_expired() {
		let mut rng = rand::thread_rng();
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let private_key = rsa_private_key.to_pkcs1_der().unwrap();

		let claims = AuthTokenClaims::new(
			"test".to_string(),
			AUTH_TOKEN_SESSION_TYPE.to_string(),
			AuthOptions { expires_at: 100 },
		);
		let token = jwt::create(&claims, private_key.as_bytes()).unwrap();

		let current_block = 150;
		let validation = Validation::new("test".to_string(), current_block);
		let result = token.validate(private_key.as_bytes(), validation);

		assert_eq!(result, Err(Error::ExpiredToken));
	}

	#[test]
	fn test_auth_token_invalid_subject() {
		let mut rng = rand::thread_rng();
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let private_key = rsa_private_key.to_pkcs1_der().unwrap();

		let claims = AuthTokenClaims::new(
			"test".to_string(),
			AUTH_TOKEN_SESSION_TYPE.to_string(),
			AuthOptions { expires_at: 100 },
		);
		let token = jwt::create(&claims, private_key.as_bytes()).unwrap();

		let current_block = 50;
		let validation = Validation::new("invalid-sub".to_string(), current_block);
		let result = token.validate(private_key.as_bytes(), validation);

		assert_eq!(result, Err(Error::InvalidSubject));
	}
}
