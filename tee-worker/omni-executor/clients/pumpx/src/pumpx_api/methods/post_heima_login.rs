use oe_primitives::{ClientAuth, UserId};
use parity_scale_codec::{Decode, Encode};
use reqwest::Error;
use serde::{Deserialize, Serialize};

use crate::methods::common::ApiResponse;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostHeimaLoginBody {
	pub user_id: UserId,
	pub client_id: String,
	pub client_auth: Option<ClientAuth>,
	// TODO:
	// here's a tricky part: if heima_login fails, we still call `post_heima_login` to notify backend
	// to allow backend to handle it accordingly.
	// Should backend return Ok or Error here? If backend returns Ok (that indicates the call itself is Ok),
	// we'll still need to return error to F/E, as the whole login process failed.
	pub heima_login_success: bool,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct PostHeimaLoginResponseData {
	pub user_id: Option<String>,
}

pub type PostHeimaLoginResponse = ApiResponse<PostHeimaLoginResponseData>;

pub async fn post_heima_login_impl(
	client: &crate::pumpx_api::PumpxApiClient,
	access_token: &str,
	body: PostHeimaLoginBody,
) -> Result<PostHeimaLoginResponse, Error> {
	let endpoint = client.base_url.join("v3/account/post_heima_login").unwrap();
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
