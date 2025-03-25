use jsonwebtoken::{
	decode as decode_jwt, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{de::DeserializeOwned, Serialize};

pub fn create<T: Serialize>(claims: &T, private_key: &[u8]) -> Result<String, String> {
	let encoding_key = EncodingKey::from_rsa_pem(private_key).map_err(|e| {
		log::error!("Failed to create encoding key: {:?}", e);
		e.to_string()
	})?;
	let header = Header::new(Algorithm::RS256);
	encode(&header, claims, &encoding_key).map_err(|e| {
		log::error!("Failed to encode token: {:?}", e);
		e.to_string()
	})
}

pub fn decode<T: DeserializeOwned>(token: &str, public_key: &[u8]) -> Result<T, String> {
	let mut validation = Validation::new(Algorithm::RS256);
	validation.set_required_spec_claims(&["sub"]);
	validation.validate_exp = false;
	let decoding_key = DecodingKey::from_rsa_pem(public_key).map_err(|e| {
		log::error!("Failed to create decoding key: {:?}", e);
		e.to_string()
	})?;
	decode_jwt::<T>(token, &decoding_key, &validation)
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
		let private_key = include_bytes!("../test_private_key.pem");
		let claims = JwtClaims { sub: "test".to_string() };

		let token = create(&claims, private_key).unwrap();
		let public_key = include_bytes!("../test_public_key.pem");
		let decoded = decode::<JwtClaims>(&token, public_key).unwrap();

		assert_eq!(claims, decoded);
	}
}
