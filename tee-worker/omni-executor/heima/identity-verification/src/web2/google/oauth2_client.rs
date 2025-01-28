use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};

const OAUTH2_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoogleOAuth2TokenResponse {
	access_token: String,
	expires_in: u64,
	id_token: String,
	scope: String,
	token_type: String,
}

pub struct GoogleOAuth2Client {
	client: Client,
	client_id: String,
	client_secret: String,
}

impl GoogleOAuth2Client {
	pub fn new(client_id: String, client_secret: String) -> Self {
		let client = ClientBuilder::new()
			.redirect(reqwest::redirect::Policy::none())
			.build()
			.unwrap();

		Self { client, client_id, client_secret }
	}

	pub async fn exchange_code_for_token(
		&self,
		code: String,
		redirect_uri: String,
	) -> Result<String, String> {
		let response = self
			.client
			.post(OAUTH2_TOKEN_ENDPOINT)
			.form(&[
				("code", code),
				("client_id", self.client_id.clone()),
				("client_secret", self.client_secret.clone()),
				("redirect_uri", redirect_uri),
				("grant_type", "authorization_code".to_string()),
			])
			.send()
			.await
			.map_err(|e| e.to_string())?
			.json::<GoogleOAuth2TokenResponse>()
			.await
			.map_err(|e| e.to_string())?;

		Ok(response.id_token)
	}
}
