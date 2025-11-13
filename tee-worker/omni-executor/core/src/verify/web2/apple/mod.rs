use serde::Deserialize;

pub use super::oauth2_common::AuthorizeData;

pub const BASE_URL: &str = "https://appleid.apple.com/auth/authorize";
pub const SCOPES: &str = "email";

#[derive(Deserialize)]
pub struct IdToken {
	pub iss: String,
	pub aud: String,
	pub sub: String,
	pub email: String,
	pub email_verified: bool,
	#[serde(default)]
	pub nonce: Option<String>,
	pub iat: u64,
	pub exp: u64,
	#[serde(default)]
	pub is_private_email: Option<bool>,
	#[serde(default)]
	pub c_hash: Option<String>,
	#[serde(default)]
	pub auth_time: Option<u64>,
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
		let authorize_data = super::super::oauth2_common::get_authorize_data(
			BASE_URL,
			client_id,
			Some(redirect_uri),
			SCOPES,
			true,
		);

		let authorize_url = Url::parse(&authorize_data.authorize_url).unwrap();
		std::println!("{:?}", authorize_url.as_str());
		let expected_url = format!("https://appleid.apple.com/auth/authorize?response_type=code&client_id=com.example.app&scope=email&state={}&nonce={}&redirect_uri=https%3A%2F%2Fexample.com%2Fcallback&response_mode=form_post", authorize_data.state, authorize_data.nonce);

		assert_eq!(authorize_url.as_str(), expected_url);
	}

	#[test]
	fn decode_id_token_works() {
		let token = "eyJraWQiOiJZUXJxZE1ENGJxIiwiYWxnIjoiUlMyNTYifQ.eyJpc3MiOiJodHRwczovL2FwcGxlaWQuYXBwbGUuY29tIiwiYXVkIjoiaW8ud2lsZG1ldGEuYXBwIiwiZXhwIjoxNzYwNjcxMjA3LCJpYXQiOjE3NjA1ODQ4MDcsInN1YiI6IjAwMDE3OS5kMmJjM2EyYzQ3Njg0YmQ5OTY0NGE0ZDU3MWVjM2IzNy4wMzE4IiwiY19oYXNoIjoiUjA0M2lDWGF5c213am5welQ3MmtzZyIsImVtYWlsIjoiaG1iaHJ0NW05dEBwcml2YXRlcmVsYXkuYXBwbGVpZC5jb20iLCJlbWFpbF92ZXJpZmllZCI6dHJ1ZSwiaXNfcHJpdmF0ZV9lbWFpbCI6dHJ1ZSwiYXV0aF90aW1lIjoxNzYwNTg0ODA3LCJub25jZV9zdXBwb3J0ZWQiOnRydWV9.cLUNFwx9TdCw-mH2QIMqJdnq1zzT-ODlV5saNQyjWlOYomxyRAEJKBF7F-3D-a3EJN9p1rLZ_LOeg6zB3OlAys4VgaSpg-YCqDs49a3hNrdh5NS5Lo6mwDW9qqGY6MdTUUGZiOe9ciYw76HatZsDRwiUDl22vZJHLFEhkXXZ7FDzHTAJs08THIWZZB7lXB6uEdbHiRUFrKlajV36C_PMJnHRtrgF3Tm_TElHZpFiir_47qNqfaX536D7cm3F-Z0-9nonQOHGb_swbMfS-3eGQueAPhIG5_xAR5MPMEySlUVbRu0WD0UqPYk66NzX_6jIowbHKzh-horyIODhFYqPjA";
		let id_token: IdToken = super::super::oauth2_common::decode_id_token(token).unwrap();

		assert_eq!(id_token.iss, "https://appleid.apple.com");
		assert_eq!(id_token.aud, "io.wildmeta.app");
		assert_eq!(id_token.exp, 1760671207);
		assert_eq!(id_token.iat, 1760584807);
		assert_eq!(id_token.sub, "000179.d2bc3a2c47684bd99644a4d571ec3b37.0318");
		assert_eq!(id_token.email, "hmbhrt5m9t@privaterelay.appleid.com");
		assert_eq!(id_token.email_verified, true);
		assert_eq!(id_token.is_private_email, Some(true));
		assert_eq!(id_token.c_hash, Some("R043iCXaysmwjnpzT72ksg".to_string()));
		assert_eq!(id_token.auth_time, Some(1760584807));
		assert_eq!(id_token.nonce, None);
	}
}
