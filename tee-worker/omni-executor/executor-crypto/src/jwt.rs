use jsonwebtoken::{decode as decode_jwt, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Serialize};

pub fn create<T: Serialize>(claims: &T, secret: &[u8]) -> Result<String, String> {
	encode(&Header::default(), claims, &EncodingKey::from_secret(secret)).map_err(|e| {
		log::error!("Failed to encode token: {:?}", e);
		e.to_string()
	})
}

pub fn decode<T: DeserializeOwned>(token: &str, secret: &[u8]) -> Result<T, String> {
	let mut validation = Validation::default();
	validation.set_required_spec_claims(&["sub"]);
	validation.validate_exp = false;
	decode_jwt::<T>(token, &DecodingKey::from_secret(secret), &validation)
		.map(|data| data.claims)
		.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
	struct JwtClaims {
		pub sub: String,
	}

	#[test]
	fn test_jwt() {
		let secret = b"secret";
		let claims = JwtClaims { sub: "test".to_string() };

		let token = create(&claims, secret).unwrap();
		let decoded = decode::<JwtClaims>(&token, secret).unwrap();

		assert_eq!(claims, decoded);
	}
}
