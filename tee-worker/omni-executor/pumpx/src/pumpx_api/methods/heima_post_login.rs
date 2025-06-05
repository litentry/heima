use executor_primitives::{UserAuth, UserId};
use parity_scale_codec::{Decode, Encode};
use reqwest::Error;
use serde::{Deserialize, Serialize};

use crate::methods::common::ApiResponse;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeimaPostLoginBody {
	pub user_id: UserId,
	pub client_id: String,
	pub user_auth: UserAuth,
	pub heima_login_success: bool,
	pub access_token: String,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct HeimaPostLoginResponseData {
	pub user_id: Option<String>,
}

pub type HeimaPostLoginResponse = ApiResponse<HeimaPostLoginResponseData>;

pub async fn heima_post_login_impl(
	client: &crate::pumpx_api::PumpxApiClient,
	access_token: &str,
	body: HeimaPostLoginBody,
) -> Result<HeimaPostLoginResponse, Error> {
	// TODO: figure out the path
	let endpoint = client.base_url.join("/heima_post_login").unwrap();

	let response = client
		.http_client
		.post(endpoint)
		.bearer_auth(access_token)
		.json(&body)
		.send()
		.await
		.map_err(|e| {
			tracing::error!("Failed to send Heima post login request: {:?}", e);
			e
		})?;

	let status = response.status();
	let response = response.error_for_status().map_err(|e| {
		tracing::error!("Heima post login request failed with status: {}, error: {:?}", status, e);
		e
	})?;

	response.json().await.map_err(|e| {
		tracing::error!("Failed to parse Heima post login response: {:?}", e);
		e
	})
}
