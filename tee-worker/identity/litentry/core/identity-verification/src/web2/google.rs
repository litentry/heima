use crate::helpers;
use alloc::string::String;
use url::Url;

pub struct AuthorizeData {
	pub authorize_url: String,
	pub state: String,
}

const BASE_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const SCOPES: &str = "openid email";

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
}
