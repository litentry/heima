mod oauth_store;
pub use oauth_store::*;

use crate::{
	alloc::{format, string::String, vec::Vec},
	helpers, Error, Result,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use lc_data_providers::twitter_official::Tweet;
use litentry_primitives::{self, ErrorDetail};
use sp_core::hashing::sha2_256;

#[derive(Debug)]
pub struct AuthorizeData {
	pub authorize_url: String,
	pub code_verifier: String,
	pub state: String,
}

const BASE_URL: &str = "https://twitter.com/i/oauth2/authorize";
const SCOPES: &str = "tweet.read%20users.read";

pub fn get_authorize_data(client_id: &str, redirect_uri: &str) -> AuthorizeData {
	let state = get_state_verifier();
	let code_verifier = get_code_verifier();
	let code_verifier_hash = sha2_256(code_verifier.as_bytes());
	let code_challenge = URL_SAFE_NO_PAD.encode(code_verifier_hash);

	let authorize_url = format!(
		"{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
		BASE_URL,
		client_id,
		redirect_uri,
		SCOPES,
		state,
		code_challenge
	);

	AuthorizeData { authorize_url, code_verifier, state }
}

pub fn payload_from_tweet(tweet: &Tweet) -> Result<Vec<u8>> {
	hex::decode(tweet.text.strip_prefix("0x").unwrap_or(tweet.text.as_str()))
		.map_err(|_| Error::LinkIdentityFailed(ErrorDetail::ParseError))
}

fn get_code_verifier() -> String {
	helpers::get_random_string(128)
}

fn get_state_verifier() -> String {
	helpers::get_random_string(32)
}

#[cfg(test)]
mod tests {
	use super::*;
	use url::Url;

	#[test]
	fn test_get_authorize_data() {
		let client_id = "client_id";
		let redirect_uri = "http://localhost:8080";
		let authorize_data = get_authorize_data(client_id, redirect_uri);

		let authorize_url = Url::parse(&authorize_data.authorize_url).unwrap();
		assert_eq!(authorize_url.query_pairs().count(), 7);
		assert_eq!(
			authorize_url.query_pairs().find(|(key, _)| key == "response_type").unwrap().1,
			"code"
		);
		assert_eq!(
			authorize_url.query_pairs().find(|(key, _)| key == "client_id").unwrap().1,
			client_id
		);
		assert_eq!(
			authorize_url.query_pairs().find(|(key, _)| key == "redirect_uri").unwrap().1,
			redirect_uri
		);
		assert_eq!(
			authorize_url.query_pairs().find(|(key, _)| key == "scope").unwrap().1,
			SCOPES.replace("%20", " ")
		);
		assert_eq!(
			authorize_url
				.query_pairs()
				.find(|(key, _)| key == "code_challenge_method")
				.unwrap()
				.1,
			"S256"
		);
		assert_eq!(authorize_data.code_verifier.len(), 128);
		assert_eq!(authorize_data.state.len(), 32);
	}
}
