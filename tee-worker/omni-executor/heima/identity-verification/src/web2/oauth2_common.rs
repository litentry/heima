use crate::helpers;
use base64::prelude::{Engine, BASE64_URL_SAFE_NO_PAD};
use serde::de::DeserializeOwned;
use url::Url;

pub struct AuthorizeData {
	pub authorize_url: String,
	pub state: String,
}

pub fn get_authorize_data(
	base_url: &str,
	client_id: &str,
	redirect_uri: &str,
	scope: &str,
	include_response_mode: bool,
) -> AuthorizeData {
	let state = helpers::generate_alphanumeric_otp(32);
	let mut authorize_url = Url::parse(base_url).expect("Failed to parse URL");

	let params = vec![
		("response_type", "code"),
		("client_id", client_id),
		("redirect_uri", redirect_uri),
		("scope", scope),
		("state", &state),
	];

	authorize_url.query_pairs_mut().extend_pairs(&params);

	if include_response_mode {
		authorize_url.query_pairs_mut().append_pair("response_mode", "form_post");
	}

	AuthorizeData { authorize_url: authorize_url.into(), state }
}

pub fn decode_id_token<T: DeserializeOwned>(token: &str) -> Result<T, &'static str> {
	let parts: Vec<&str> = token.split('.').collect();
	if parts.len() != 3 {
		return Err("Invalid token format");
	}
	let payload = base64_decode(parts[1])?;
	serde_json::from_str(&payload).map_err(|_| "Failed to parse claims")
}

fn base64_decode(input: &str) -> Result<String, &'static str> {
	let decoded = &BASE64_URL_SAFE_NO_PAD.decode(input).map_err(|_| "Failed to decode base64")?;

	Ok(String::from_utf8_lossy(decoded).to_string())
}
