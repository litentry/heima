mod oauth2_state_store;
pub use oauth2_state_store::*;

use crate::helpers;
use alloc::{
	string::{String, ToString},
	vec::Vec,
};
use base64::prelude::{Engine, BASE64_URL_SAFE_NO_PAD};
use serde::Deserialize;
use url::Url;

const BASE_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const SCOPES: &str = "openid email";

pub struct AuthorizeData {
	pub authorize_url: String,
	pub state: String,
}

pub fn get_authorize_data(client_id: &str, redirect_uri: &str) -> AuthorizeData {
	let state = helpers::get_random_string(32);

	let mut authorize_url = Url::parse(BASE_URL).unwrap();
	authorize_url.query_pairs_mut().extend_pairs(&[
		("response_type", "code"),
		("client_id", client_id),
		("redirect_uri", redirect_uri),
		("scope", SCOPES),
		("state", &state),
	]);

	AuthorizeData { authorize_url: authorize_url.into(), state }
}

#[derive(Deserialize)]
pub struct GoogleClaims {
	pub iss: String,
	pub azp: String,
	pub email_verified: bool,
	pub at_hash: String,
	pub aud: String,
	pub exp: u64,
	pub iat: u64,
	pub sub: String,
	pub hd: String,
	pub email: String,
}

pub fn decode_jwt(jwt: &str) -> Result<GoogleClaims, &'static str> {
	let parts: Vec<&str> = jwt.split('.').collect();
	if parts.len() != 3 {
		return Err("Invalid JWT")
	}
	let payload = base64_decode(parts[1])?;
	let claims: GoogleClaims =
		serde_json::from_str(&payload).map_err(|_| "Failed to parse claims")?;
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
		let client_id = "client_id";
		let redirect_uri = "http://localhost:8080";
		let authorize_data = get_authorize_data(client_id, redirect_uri);

		let authorize_url = Url::parse(&authorize_data.authorize_url).unwrap();
		std::println!("{:?}", authorize_url.as_str());
		let expected_url = format!("https://accounts.google.com/o/oauth2/v2/auth?response_type=code&client_id=client_id&redirect_uri=http%3A%2F%2Flocalhost%3A8080&scope=openid+email&state={}", authorize_data.state);

		assert_eq!(authorize_url.as_str(), expected_url);
	}

	#[test]
	fn decode_google_jwt_works() {
		let jwt = "eyJhbGciOiJSUzI1NiIsImtpZCI6IjM2MjgyNTg2MDExMTNlNjU3NmE0NTMzNzM2NWZlOGI4OTczZDE2NzEiLCJ0eXAiOiJKV1QifQ.eyJpc3MiOiJodHRwczovL2FjY291bnRzLmdvb2dsZS5jb20iLCJhenAiOiI2ODYyOTM4MTAwNjktbTBhNzVwYm9mMWVwbzJzZzkyYTU3cHRtazg1c2FnbGYuYXBwcy5nb29nbGV1c2VyY29udGVudC5jb20iLCJhdWQiOiI2ODYyOTM4MTAwNjktbTBhNzVwYm9mMWVwbzJzZzkyYTU3cHRtazg1c2FnbGYuYXBwcy5nb29nbGV1c2VyY29udGVudC5jb20iLCJzdWIiOiIxMDE2NTk5MjE1MTM4NzY4MzIwNDgiLCJoZCI6Imthd2FnYXJiby10ZWNoLmlvIiwiZW1haWwiOiJmcmFuY2lzY29Aa2F3YWdhcmJvLXRlY2guaW8iLCJlbWFpbF92ZXJpZmllZCI6dHJ1ZSwiYXRfaGFzaCI6IlBuYndCRVIzTnVBa055dFplR2wzcGciLCJpYXQiOjE3MzMyMzU4NDcsImV4cCI6MTczMzIzOTQ0N30.n4gYeYhp2U1ud4bZNW02xMJadki_93CzlcsJnr8F6eIBXwu4-CbsqToNNn40Kq780Wwz44MqnrEIU8dkBLqBc6MBWkMqzQV-RteEXMiZSOAhkNl8dIzds4vDZUnXunom4y-RYcW7yFMu_Vzpdi9A1NmgMvKVf9wqgfTJrqmPwaUh1GfgV8e7SrqHJiI3XVTE_zIxQVdjybR-7dXGh2B9LaXtA1m8v47tNkvtifa7KUw-miSIVt0of0Dq3keETLyptf8HJ1HouwpACMnxSH-Foq3r5EVp3lfGmkmf5dWMxweagsi7-hMhSKsGY2q2g3gy8xxsCaS1Q3uiB1Htw1Dn7Q";
		let claims = decode_jwt(jwt).unwrap();

		assert_eq!(claims.iss, "https://accounts.google.com");
		assert_eq!(
			claims.azp,
			"686293810069-m0a75pbof1epo2sg92a57ptmk85saglf.apps.googleusercontent.com"
		);
		assert_eq!(claims.email_verified, true);
		assert_eq!(claims.at_hash, "PnbwBER3NuAkNytZeGl3pg");
		assert_eq!(
			claims.aud,
			"686293810069-m0a75pbof1epo2sg92a57ptmk85saglf.apps.googleusercontent.com"
		);
		assert_eq!(claims.exp, 1733239447);
		assert_eq!(claims.iat, 1733235847);
		assert_eq!(claims.sub, "101659921513876832048");
		assert_eq!(claims.hd, "kawagarbo-tech.io");
		assert_eq!(claims.email, "francisco@kawagarbo-tech.io");
	}
}
