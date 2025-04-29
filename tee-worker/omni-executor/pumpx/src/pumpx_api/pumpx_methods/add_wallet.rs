use crate::pumpx_api::pumpx_types::add_wallet::AddWalletResponse;
use reqwest::Error;

pub async fn add_wallet(
	&self,
	access_token: &str,
	language: Option<String>,
) -> Result<AddWalletResponse, Error> {
	let endpoint = self.base_url.join("v3/account/add_wallet").unwrap();
	let response = self
		.http_client
		.post(endpoint)
		.header("Content-Length", 0)
		.header("X-Language", language.unwrap_or("en".to_string()))
		.bearer_auth(access_token)
		.send()
		.await
		.map_err(|e| {
			log::error!("Failed to send add_wallet request: {:?}", e);
			e
		})?;
	let status = response.status();
	let response = response.error_for_status().map_err(|e| {
		log::error!("add_wallet request failed with status: {}, error: {:?}", status, e);
		e
	})?;
	response.json().await.map_err(|e| {
		log::error!("Failed to parse add_wallet response: {:?}", e);
		e
	})
}
