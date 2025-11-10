use crate::constants::{CLIENT_ID_HEIMA, CLIENT_ID_PUMPX, CLIENT_ID_WILDMETA};
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

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct AuthOptions {
	pub expires_at: i64,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct AuthTokenClaims {
	pub sub: String,
	pub typ: String,
	pub exp: i64,
	pub aud: String,
}

impl AuthTokenClaims {
	pub fn new(sub: String, typ: String, aud: String, options: AuthOptions) -> Self {
		Self { sub, typ, exp: options.expires_at, aud }
	}
}

pub struct Validation {
	pub typ: String,
	pub skip_exp_check: bool,
}

impl Validation {
	pub fn new(typ: String, skip_exp_check: bool) -> Self {
		Self { typ, skip_exp_check }
	}

	pub fn validate(&self, claims: &AuthTokenClaims) -> Result<(), Error> {
		if self.typ != claims.typ {
			return Err(Error::JwtError(jwt::ErrorKind::InvalidToken));
		}

		Ok(())
	}
}

pub trait AuthTokenValidator<T> {
	fn validate(&self, secret: &[u8], validation: Validation) -> Result<T, Error>;
}

impl AuthTokenValidator<AuthTokenClaims> for String {
	fn validate(
		&self,
		private_key: &[u8],
		validation: Validation,
	) -> Result<AuthTokenClaims, Error> {
		let rsa_private_key =
			RsaPrivateKey::from_pkcs1_der(private_key).map_err(|_| Error::InternalError)?;
		let public_key = rsa_private_key
			.to_public_key()
			.to_pkcs1_der()
			.map_err(|_| Error::InternalError)?;
		let claims = jwt::decode::<AuthTokenClaims>(
			self,
			public_key.as_bytes(),
			Some(&[CLIENT_ID_HEIMA, CLIENT_ID_PUMPX, CLIENT_ID_WILDMETA]),
			validation.skip_exp_check,
		)
		.map_err(|e| Error::JwtError(e.kind().clone()))?;
		validation.validate(&claims)?;
		Ok(claims)
	}
}

impl AuthTokenValidator<AuthTokenClaims> for &str {
	fn validate(
		&self,
		private_key: &[u8],
		validation: Validation,
	) -> Result<AuthTokenClaims, Error> {
		let rsa_private_key =
			RsaPrivateKey::from_pkcs1_der(private_key).map_err(|_| Error::InternalError)?;
		let public_key = rsa_private_key
			.to_public_key()
			.to_pkcs1_der()
			.map_err(|_| Error::InternalError)?;
		let claims = jwt::decode::<AuthTokenClaims>(
			self,
			public_key.as_bytes(),
			Some(&[CLIENT_ID_HEIMA, CLIENT_ID_PUMPX, CLIENT_ID_WILDMETA]),
			validation.skip_exp_check,
		)
		.map_err(|e| Error::JwtError(e.kind().clone()))?;
		validation.validate(&claims)?;
		Ok(claims)
	}
}

#[cfg(test)]
mod tests {
	use crate::constants::{AUTH_TOKEN_ACCESS_TYPE, AUTH_TOKEN_ID_TYPE, CLIENT_ID_HEIMA};

	use super::*;
	use chrono::{Days, Utc};
	use executor_primitives::{utils::hex::hex_encode, Identity, Web2IdentityType};
	use rsa::{pkcs1::EncodeRsaPrivateKey, RsaPrivateKey};

	#[derive(PartialEq, Debug, Serialize, Deserialize)]
	pub struct TestAuthTokenNoSubClaims {
		pub typ: String,
		pub exp: i64,
	}

	#[derive(PartialEq, Debug, Serialize, Deserialize)]
	pub struct TestAuthTokenNoTypClaims {
		pub sub: String,
		pub exp: i64,
	}

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

		let omni_account = Identity::Pumpx("012345".into()).to_omni_account(CLIENT_ID_HEIMA);

		let claims = AuthTokenClaims::new(
			hex_encode(omni_account.as_ref()),
			AUTH_TOKEN_ID_TYPE.to_string(),
			CLIENT_ID_HEIMA.to_string(),
			AuthOptions { expires_at },
		);
		let token = jwt::create(&claims, private_key.as_bytes()).unwrap();

		let validation = Validation::new(AUTH_TOKEN_ID_TYPE.to_string(), false);
		let result = token.validate(private_key.as_bytes(), validation);

		assert_eq!(result, Ok(claims));
	}

	#[test]
	fn test_auth_token_expired() {
		let mut rng = rand::thread_rng();
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let private_key = rsa_private_key.to_pkcs1_der().unwrap();

		let omni_account = Identity::Pumpx("012345".into())(CLIENT_ID_HEIMA);

		let claims = AuthTokenClaims::new(
			hex_encode(omni_account.as_ref()),
			AUTH_TOKEN_ID_TYPE.to_string(),
			CLIENT_ID_HEIMA.to_string(),
			AuthOptions { expires_at: 100 },
		);
		let token = jwt::create(&claims, private_key.as_bytes()).unwrap();

		let validation = Validation::new(AUTH_TOKEN_ID_TYPE.to_string(), false);
		let result = token.validate(private_key.as_bytes(), validation);

		assert_eq!(result, Err(Error::JwtError(jwt::ErrorKind::ExpiredSignature)));
	}

	#[test]
	fn test_auth_token_missing_subject() {
		let mut rng = rand::thread_rng();
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let private_key = rsa_private_key.to_pkcs1_der().unwrap();

		let expires_at = Utc::now()
			.checked_add_days(Days::new(1))
			.expect("Failed to calculate expiration")
			.timestamp();

		let claims =
			TestAuthTokenNoSubClaims { typ: AUTH_TOKEN_ID_TYPE.to_string(), exp: expires_at };
		let token = jwt::create(&claims, private_key.as_bytes()).unwrap();

		let validation = Validation::new(AUTH_TOKEN_ID_TYPE.to_string(), false);
		let result = token.validate(private_key.as_bytes(), validation);
		let Err(Error::JwtError(jwt::ErrorKind::Json(e))) = result else {
			panic!("Expected JsonError, got {:?}", result);
		};
		assert!(e.to_string().contains("missing field `sub`"));
	}

	#[test]
	fn test_auth_token_missing_typ() {
		let mut rng = rand::thread_rng();
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let private_key = rsa_private_key.to_pkcs1_der().unwrap();

		let expires_at = Utc::now()
			.checked_add_days(Days::new(1))
			.expect("Failed to calculate expiration")
			.timestamp();

		let claims = TestAuthTokenNoTypClaims { sub: "test-sub".to_string(), exp: expires_at };
		let token = jwt::create(&claims, private_key.as_bytes()).unwrap();

		let validation = Validation::new(AUTH_TOKEN_ID_TYPE.to_string(), false);
		let result = token.validate(private_key.as_bytes(), validation);
		let Err(Error::JwtError(jwt::ErrorKind::Json(e))) = result else {
			panic!("Expected JsonError, got {:?}", result);
		};
		assert!(e.to_string().contains("missing field `typ`"));
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

		let omni_account = Identity::Email("test@test.com".into()).to_omni_account(CLIENT_ID_HEIMA);

		let claims = AuthTokenClaims::new(
			hex_encode(omni_account.as_ref()),
			AUTH_TOKEN_ACCESS_TYPE.to_string(),
			CLIENT_ID_HEIMA.to_string(),
			AuthOptions { expires_at },
		);
		let token = jwt::create(&claims, private_key.as_bytes()).unwrap();

		let validation = Validation::new(AUTH_TOKEN_ID_TYPE.to_string(), false);
		let result = token.validate(private_key.as_bytes(), validation);

		assert_eq!(result, Err(Error::JwtError(jwt::ErrorKind::InvalidToken)));
	}
}
