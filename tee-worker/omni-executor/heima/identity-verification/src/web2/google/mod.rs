mod oauth2_client;
pub use oauth2_client::GoogleOAuth2Client;

use crate::helpers;
use url::Url;

const BASE_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const SCOPES: &str = "openid email";

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
	]);

	AuthorizeData { authorize_url: authorize_url.into(), state }
}
