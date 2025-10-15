use crate::helpers;
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
