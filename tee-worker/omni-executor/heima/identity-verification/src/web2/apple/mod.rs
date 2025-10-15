use crate::helpers;
use base64::prelude::{Engine, BASE64_URL_SAFE_NO_PAD};
use serde::Deserialize;
use url::Url;

const BASE_URL: &str = "https://appleid.apple.com/auth/authorize";
const SCOPES: &str = "email";

#[derive(Deserialize)]
pub struct IdToken {
	pub iss: String,
	pub aud: String,
	pub exp: u64,
	pub iat: u64,
	pub sub: String,
	pub email: String,
	pub email_verified: Option<String>,
}

pub struct AuthorizeData {
	pub authorize_url: String,
	pub state: String,
}

pub fn get_authorize_data(client_id: &str, redirect_uri: &str) -> AuthorizeData {
	let state = helpers::generate_alphanumeric_otp(32);
	let mut authorize_url = Url::parse(BASE_URL).expect("Failed to parse URL");
	authorize_url.query_pairs_mut().extend_pairs(&[
		("response_type", "code"),
		("client_id", client_id),
		("redirect_uri", redirect_uri),
		("scope", SCOPES),
		("state", &state),
		("response_mode", "form_post"),
	]);

	AuthorizeData { authorize_url: authorize_url.into(), state }
}

pub fn decode_id_token(token: &str) -> Result<IdToken, &'static str> {
	let parts: Vec<&str> = token.split('.').collect();
	if parts.len() != 3 {
		return Err("Invalid token format");
	}
	let payload = base64_decode(parts[1])?;
	let claims: IdToken = serde_json::from_str(&payload).map_err(|_| "Failed to parse claims")?;
	Ok(claims)
}

fn base64_decode(input: &str) -> Result<String, &'static str> {
	let decoded = &BASE64_URL_SAFE_NO_PAD.decode(input).map_err(|_| "Failed to decode base64")?;

	Ok(String::from_utf8_lossy(decoded).to_string())
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::format;
	use url::Url;

	#[test]
	fn test_get_authorize_data() {
		let client_id = "com.example.app";
		let redirect_uri = "https://example.com/callback";
		let authorize_data = get_authorize_data(client_id, redirect_uri);

		let authorize_url = Url::parse(&authorize_data.authorize_url).unwrap();
		std::println!("{:?}", authorize_url.as_str());
		let expected_url = format!("https://appleid.apple.com/auth/authorize?response_type=code&client_id=com.example.app&redirect_uri=https%3A%2F%2Fexample.com%2Fcallback&scope=email&state={}&response_mode=form_post", authorize_data.state);

		assert_eq!(authorize_url.as_str(), expected_url);
	}
}
