use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OAuth2TokenResponse {
	pub access_token: String,
	pub expires_in: u64,
	pub id_token: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub refresh_token: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub scope: Option<String>,
	pub token_type: String,
}

pub struct OAuth2Client {
	client: Client,
	client_id: String,
	client_secret: String,
	token_endpoint: String,
}

impl OAuth2Client {
	pub fn new(client_id: String, client_secret: String, token_endpoint: String) -> Self {
		let client = ClientBuilder::new()
			.redirect(reqwest::redirect::Policy::none())
			.build()
			.unwrap();

		Self { client, client_id, client_secret, token_endpoint }
	}

	pub async fn exchange_code_for_token(
		&self,
		code: String,
		redirect_uri: Option<String>,
	) -> Result<String, String> {
		let mut params = vec![
			("code", code),
			("client_id", self.client_id.clone()),
			("client_secret", self.client_secret.clone()),
			("grant_type", "authorization_code".to_string()),
		];

		if let Some(uri) = redirect_uri {
			params.push(("redirect_uri", uri));
		}

		let response = self
			.client
			.post(&self.token_endpoint)
			.form(&params)
			.send()
			.await
			.map_err(|e| e.to_string())?
			.json::<OAuth2TokenResponse>()
			.await
			.map_err(|e| e.to_string())?;

		Ok(response.id_token)
	}
}
