pub use jsonwebtoken::errors::{Error, ErrorKind};
use jsonwebtoken::{
	decode as decode_jwt, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{de::DeserializeOwned, Serialize};

pub fn create<T: Serialize>(claims: &T, private_key: &[u8]) -> Result<String, String> {
	let encoding_key = EncodingKey::from_rsa_der(private_key);
	let header = Header::new(Algorithm::RS256);
	encode(&header, claims, &encoding_key).map_err(|e| {
		log::error!("Failed to encode token: {:?}", e);
		e.to_string()
	})
}

pub fn decode<T: DeserializeOwned>(token: &str, public_key: &[u8]) -> Result<T, Error> {
	let validation = Validation::new(Algorithm::RS256);
	let decoding_key = DecodingKey::from_rsa_der(public_key);
	decode_jwt::<T>(token, &decoding_key, &validation).map(|data| data.claims)
}

#[cfg(test)]
mod tests {
	use super::*;
	use chrono::{Days, Utc};
	use rsa::{
		pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey},
		RsaPrivateKey,
	};

	#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
	struct JwtClaims {
		pub sub: String,
		pub exp: i64,
	}

	#[test]
	fn test_jwt() {
		let mut rng = rand::thread_rng();
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let private_key = rsa_private_key.to_pkcs1_der().unwrap();
		let public_key = rsa_private_key.to_public_key().to_pkcs1_der().unwrap();

		let exp = Utc::now()
			.checked_add_days(Days::new(1))
			.expect("Failed to calculate expiration")
			.timestamp();
		let claims = JwtClaims { sub: "test".to_string(), exp };

		let token = create(&claims, private_key.as_bytes()).unwrap();
		let decoded = decode::<JwtClaims>(&token, public_key.as_bytes()).unwrap();

		assert_eq!(claims, decoded);
	}
}
