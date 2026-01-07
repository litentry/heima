use parity_scale_codec::{Decode, Encode};
use reqwest::Error;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::methods::common::ApiResponse;
use crate::WildmetaApiClient;

#[derive(Serialize)]
pub struct HyperliquidLinkRequest {
	pub main_address: String,
	pub agent_address: String,
	pub login_type: i32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HyperliquidLinkResponseData {
	pub is_bound: bool,
}

pub type HyperliquidLinkResponse = ApiResponse<HyperliquidLinkResponseData>;

pub async fn verify_hyperliquid_link_impl(
	client: &WildmetaApiClient,
	main_address: String,
	agent_address: String,
	login_type: i32,
) -> Result<HyperliquidLinkResponse, Error> {
	let endpoint = client.base_url.join("/v1/account/check_hyper_agent_address").unwrap();
	let response = client
		.http_client
		.post(endpoint)
		.json(&HyperliquidLinkRequest { main_address, agent_address, login_type })
		.send()
		.await
		.map_err(|e| {
			error!("Failed to send hyperliquid link verification request: {:?}", e);
			e
		})?;
	let status = response.status();
	let response = response.error_for_status().map_err(|e| {
		error!("Hyperliquid link verification failed with status: {}, error: {:?}", status, e);
		e
	})?;
	response.json().await.map_err(|e| {
		error!("Failed to parse hyperliquid link verification response: {:?}", e);
		e
	})
}
