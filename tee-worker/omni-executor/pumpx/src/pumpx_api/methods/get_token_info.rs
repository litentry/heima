use crate::methods::common::ApiResponse;
use crate::PumpxApiClient;
use reqwest::Error;
use serde::{Deserialize, Serialize};
use sp_core::{Decode, Encode};
use tracing::error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct GetTokenInfoBody {
	pub chain_id: i64,
	pub token_address: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct GetTokenInfo {
	pub chain_id: i64,
	pub address: String,
	pub name: String,
	pub symbol: String,
	pub decimals: i64,
	pub total_supply: f64,
	pub icon: String,
	pub hold_count: i64,
	pub is_ca_drop_owner: i64,
	pub is_ca_verify: i64,
	pub is_honey_scam: i64,
	pub is_liquid_lock: i64,
	pub is_can_pause_trade: i64,
	pub is_can_change_tax: i64,
	pub is_have_black_list: i64,
	pub is_can_all_sell: i64,
	pub is_have_proxy: i64,
	pub is_can_external_call: i64,
	pub is_can_add_token: i64,
	pub is_can_change_token: i64,
	pub sell_tax: f64,
	pub buy_tax: f64,
	pub twitter_username: String,
	pub website: String,
	pub telegram: String,
	pub is_check_ca: i64,
	pub check_ca_at: i64,
	pub program: String,
}

pub type GetTokenInfoResponse = ApiResponse<GetTokenInfo>;

pub async fn get_token_info_impl(
	client: &PumpxApiClient,
	access_token: &str,
	body: GetTokenInfoBody,
) -> Result<GetTokenInfoResponse, Error> {
	let endpoint = client.base_url.join("v3/trade/get_token_info").unwrap();
	let response = client
		.http_client
		.post(endpoint)
		.bearer_auth(access_token)
		.json(&body)
		.send()
		.await
		.map_err(|e| {
			error!("Failed to send get_token_info request: {:?}", e);
			e
		})?;

	let status = response.status();
	let response = response.error_for_status().map_err(|e| {
		error!("get_token_info failed with status: {}, error: {:?}", status, e);
		e
	})?;

	response.json().await.map_err(|e| {
		error!("Failed to parse get_token_info response: {:?}", e);
		e
	})
}
