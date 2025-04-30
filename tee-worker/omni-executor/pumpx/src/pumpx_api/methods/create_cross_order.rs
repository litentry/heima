use reqwest::Error;
use serde::Serialize;

use super::common::{OrderInfoResponse, SwapType};
use crate::pumpx_api::PumpxApiClient;

// /v3/trade/create_cross_order
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateCrossOrderBody {
	pub request_id: u32,
	pub chain_id: u32,
	pub token_ca: String,
	pub swap_type: SwapType,
	pub is_one_click: bool,
	pub cross_info: Vec<CrossOrderInfo>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CrossOrderInfo {
	pub chain_id: u32,
	pub wallet_index: u32,
	pub address: String,
	pub token_ca: String,
	pub amount: String,
	pub usd: String,
}

pub async fn create_cross_order_impl(
	client: &PumpxApiClient,
	access_token: &str,
	data: CreateCrossOrderBody,
) -> Result<OrderInfoResponse, Error> {
	let endpoint = client.base_url.join("v3/trade/create_cross_order").unwrap();
	let response = client
		.http_client
		.post(endpoint)
		.bearer_auth(access_token)
		.json(&data)
		.send()
		.await?;

	let status = response.status();
	let response = response.error_for_status().map_err(|e| {
		log::error!("Cross order creation failed with status: {}, error: {:?}", status, e);
		e
	})?;

	response.json().await.map_err(|e| {
		log::error!("Failed to parse cross order creation response: {:?}", e);
		e
	})
}
