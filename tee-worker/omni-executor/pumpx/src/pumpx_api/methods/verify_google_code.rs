use crate::methods::common::ApiResponse;
use crate::pumpx_api::PumpxApiClient;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

// v3/account/verify_google_code
#[derive(Serialize)]
pub struct GoogleCode {
	pub google_code: String,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct VerifyGoogleCodeResponseData {
	pub result: Option<bool>,
}

pub type VerifyGoogleCodeResponse = ApiResponse<VerifyGoogleCodeResponseData>;

pub async fn verify_google_code_impl(
	client: &PumpxApiClient,
	access_token: &str,
	google_code: String,
	language: Option<String>,
) -> Result<VerifyGoogleCodeResponse, Error> {
	let endpoint = client.base_url.join("v3/account/verify_google_code").unwrap();
	let response = client
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
