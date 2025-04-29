use crate::pumpx_api::pumpx_types::verify_google_code::{GoogleCode, VerifyGoogleCodeResponse};

pub async fn verify_google_code(
	&self,
	access_token: &str,
	google_code: String,
	language: Option<String>,
) -> Result<VerifyGoogleCodeResponse, Error> {
	let endpoint = self.base_url.join("v3/account/verify_google_code").unwrap();
	let response = self
		.http_client
		.post(endpoint)
		.header("X-Language", language.unwrap_or("en".to_string()))
		.bearer_auth(access_token)
		.json(&GoogleCode { google_code })
		.send()
		.await
		.map_err(|e| {
			log::error!("Failed to send Google code verification request: {:?}", e);
			e
		})?;
	let status = response.status();
	let response = response.error_for_status().map_err(|e| {
		log::error!("Google code verification failed with status: {}, error: {:?}", status, e);
		e
	})?;
	response.json().await.map_err(|e| {
		log::error!("Failed to parse Google code verification response: {:?}", e);
		e
	})
}
