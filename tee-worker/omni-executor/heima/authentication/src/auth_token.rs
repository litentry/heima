use executor_crypto::jwt;
use parity_scale_codec::{Decode, Encode};
use rsa::{
	pkcs1::{DecodeRsaPrivateKey, EncodeRsaPublicKey},
	RsaPrivateKey,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
pub enum Error {
	Base64DecodeError,
	JsonError,
	InternalError,
	JwtError(jwt::ErrorKind),
}

pub const AUTH_TOKEN_EXPIRATION_DAYS: u64 = 7; // 1 week
pub const AUTH_TOKEN_ACCESS_TYPE: &str = "access";
pub const AUTH_TOKEN_ID_TYPE: &str = "id";

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct AuthOptions {
	pub expires_at: i64,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct AuthTokenClaims {
	pub sub: String,
	pub typ: String,
	pub exp: i64,
}

impl AuthTokenClaims {
	pub fn new(sub: String, typ: String, options: AuthOptions) -> Self {
		Self { sub, typ, exp: options.expires_at }
	}
}

pub struct Validation {
	pub sub: String,
	pub typ: String,
}

impl Validation {
	pub fn new(sub: String, typ: String) -> Self {
		Self { sub, typ }
	}

	pub fn validate(&self, claims: &AuthTokenClaims) -> Result<(), Error> {
		if self.sub != claims.sub {
			return Err(Error::JwtError(jwt::ErrorKind::InvalidSubject));
		}
		if self.typ != claims.typ {
			return Err(Error::JwtError(jwt::ErrorKind::InvalidToken));
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
			.map_err(|e| Error::JwtError(e.kind().clone()))
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
			.map_err(|e| Error::JwtError(e.kind().clone()))
			.and_then(|claims| validation.validate(&claims))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use chrono::{Days, Utc};
	use executor_primitives::{utils::hex::ToHexPrefixed, Identity, Web2IdentityType};
	use rsa::{pkcs1::EncodeRsaPrivateKey, RsaPrivateKey};

	#[test]
	fn test_auth_token() {
		let mut rng = rand::thread_rng();
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let private_key = rsa_private_key.to_pkcs1_der().unwrap();

		let expires_at = Utc::now()
			.checked_add_days(Days::new(1))
			.expect("Failed to calculate expiration")
			.timestamp();

		let email_identity = Identity::from_web2_account("test@test.com", Web2IdentityType::Email);
		let omni_account = email_identity.to_omni_account();

		let claims = AuthTokenClaims::new(
			omni_account.to_hex(),
			AUTH_TOKEN_ID_TYPE.to_string(),
			AuthOptions { expires_at },
		);
		let token = jwt::create(&claims, private_key.as_bytes()).unwrap();

		let validation = Validation::new(omni_account.to_hex(), AUTH_TOKEN_ID_TYPE.to_string());
		let result = token.validate(private_key.as_bytes(), validation);

		assert_eq!(result, Ok(()));
	}

	#[test]
	fn test_auth_token_expired() {
		let mut rng = rand::thread_rng();
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let private_key = rsa_private_key.to_pkcs1_der().unwrap();

		let email_identity = Identity::from_web2_account("test@test.com", Web2IdentityType::Email);
		let omni_account = email_identity.to_omni_account();

		let claims = AuthTokenClaims::new(
			omni_account.to_hex(),
			AUTH_TOKEN_ID_TYPE.to_string(),
			AuthOptions { expires_at: 100 },
		);
		let token = jwt::create(&claims, private_key.as_bytes()).unwrap();

		let validation = Validation::new(omni_account.to_hex(), AUTH_TOKEN_ID_TYPE.to_string());
		let result = token.validate(private_key.as_bytes(), validation);

		assert_eq!(result, Err(Error::JwtError(jwt::ErrorKind::ExpiredSignature)));
	}

	#[test]
	fn test_auth_token_invalid_subject() {
		let mut rng = rand::thread_rng();
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let private_key = rsa_private_key.to_pkcs1_der().unwrap();

		let expires_at = Utc::now()
			.checked_add_days(Days::new(1))
			.expect("Failed to calculate expiration")
			.timestamp();

		let email_identity = Identity::from_web2_account("test@test.com", Web2IdentityType::Email);
		let omni_account = email_identity.to_omni_account();

		let claims = AuthTokenClaims::new(
			omni_account.to_hex(),
			AUTH_TOKEN_ID_TYPE.to_string(),
			AuthOptions { expires_at },
		);
		let token = jwt::create(&claims, private_key.as_bytes()).unwrap();

		let validation = Validation::new("invalid-sub".to_string(), AUTH_TOKEN_ID_TYPE.to_string());
		let result = token.validate(private_key.as_bytes(), validation);

		assert_eq!(result, Err(Error::JwtError(jwt::ErrorKind::InvalidSubject)));
	}

	#[test]
	fn test_auth_token_invalid_type() {
		let mut rng = rand::thread_rng();
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let private_key = rsa_private_key.to_pkcs1_der().unwrap();

		let expires_at = Utc::now()
			.checked_add_days(Days::new(1))
			.expect("Failed to calculate expiration")
			.timestamp();

		let email_identity = Identity::from_web2_account("test@test.com", Web2IdentityType::Email);
		let omni_account = email_identity.to_omni_account();

		let claims = AuthTokenClaims::new(
			omni_account.to_hex(),
			AUTH_TOKEN_ACCESS_TYPE.to_string(),
			AuthOptions { expires_at },
		);
		let token = jwt::create(&claims, private_key.as_bytes()).unwrap();

		let validation = Validation::new(omni_account.to_hex(), AUTH_TOKEN_ID_TYPE.to_string());
		let result = token.validate(private_key.as_bytes(), validation);

		assert_eq!(result, Err(Error::JwtError(jwt::ErrorKind::InvalidToken)));
	}
}
