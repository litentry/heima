use crate::methods::common::ApiResponse;
use crate::PumpxApiClient;
use reqwest::Error;
use serde::{Deserialize, Serialize};
use sp_core::{Decode, Encode};
use tracing::error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct GetPairInfoByTokenRequestBody {
	pub chain_id: i64,
	pub token_address: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct GetPairInfoByTokenResponse {
	pub chain_id: i64,
	pub address: String,
	pub name: String,
	pub factory_address: String,
	pub base_token_address: String,
	pub token_address: String,
	pub base_token_symbol: String,
	pub token_symbol: String,
	pub base_token_decimal: i64,
	pub token_decimal: i64,
	pub base_token_is_native_token: bool,
	pub base_token_is_token0: bool,
	pub init_base_token_amount: f64,
	pub init_token_amount: f64,
	pub current_base_token_amount: f64,
	pub current_token_amount: f64,
	pub fdv: f64,
	pub mkt_cap: f64,
	pub base_token_price: f64,
	pub token_price: f64,
	pub block_num: i64,
	pub block_time: i64,
	pub highest_token_price: f64,
	pub latest_trade_time: i64,
}

pub type GetPairInfoByTokenRequestResponse = ApiResponse<GetPairInfoByTokenResponse>;

pub async fn get_pair_info_by_token_impl(
	client: &PumpxApiClient,

	access_token: &str,
	body: GetPairInfoByTokenRequestBody,
) -> Result<GetPairInfoByTokenRequestResponse, Error> {
	let endpoint = client.base_url.join("v3/market/get_pair_info_by_token").unwrap();
	let response = client
		.http_client
		.post(endpoint)
		.bearer_auth(access_token)
		.json(&body)
		.send()
		.await
		.map_err(|e| {
			error!("Failed to send create_market_order_tx request: {:?}", e);
			e
		})?;

	let status = response.status();
	let response = response.error_for_status().map_err(|e| {
		error!("create_market_order_tx failed with status: {}, error: {:?}", status, e);
		e
	})?;

	response.json().await.map_err(|e| {
		error!("Failed to parse create_market_order_tx response: {:?}", e);
		e
	})
}
