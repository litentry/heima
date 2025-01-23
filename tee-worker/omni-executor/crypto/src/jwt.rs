pub use jsonwebtoken::Validation;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header};
use serde::{de::DeserializeOwned, Serialize};

pub fn create_token<T: Serialize>(claims: &T, secret: &[u8]) -> Result<String, String> {
	encode(&Header::default(), claims, &EncodingKey::from_secret(secret)).map_err(|e| {
		log::error!("Failed to encode token: {:?}", e);
		e.to_string()
	})
}

pub trait Jwt {
	fn verify<T: DeserializeOwned>(
		&self,
		secret: &[u8],
		validation: &mut Validation,
	) -> Result<T, String>;
}

impl Jwt for String {
	fn verify<T: DeserializeOwned>(
		&self,
		secret: &[u8],
		validation: &mut Validation,
	) -> Result<T, String> {
		decode::<T>(self, &DecodingKey::from_secret(secret), validation)
			.map(|data| data.claims)
			.map_err(|e| e.to_string())
	}
}

impl Jwt for &str {
	fn verify<T: DeserializeOwned>(
		&self,
		secret: &[u8],
		validation: &mut Validation,
	) -> Result<T, String> {
		decode::<T>(self, &DecodingKey::from_secret(secret), validation)
			.map(|data| data.claims)
			.map_err(|e| e.to_string())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use jsonwebtoken::Algorithm;

	#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
	struct JwtClaims {
		pub sub: String,
	}

	#[test]
	fn test_jwt() {
		let secret = b"secret";
		let claims = JwtClaims { sub: "test".to_string() };

		let token = create_token(&claims, secret).unwrap();
		let mut validation = Validation::default();
		validation.sub = Some("test".to_string());
		validation.set_required_spec_claims(&["sub"]);
		let decoded = token.verify::<JwtClaims>(secret, &mut validation).unwrap();

		assert_eq!(claims, decoded);
	}

	#[test]
	fn test_jwt_invalid_algorithm() {
		let secret = b"secret";
		let claims = JwtClaims { sub: "test".to_string() };

		let token = create_token(&claims, secret).unwrap();
		let mut validation = Validation::default();
		validation.algorithms = vec![Algorithm::RS256];
		let decoded = Jwt::verify::<JwtClaims>(&token, secret, &mut validation);

		assert_eq!(decoded, Err("InvalidAlgorithm".to_string()));
	}
}
