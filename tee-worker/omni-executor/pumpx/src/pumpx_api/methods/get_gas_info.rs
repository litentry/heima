use super::common::ApiResponse;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::pumpx_api::PumpxApiClient;

// /v1/trade/get_gas_info
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetGasInfoParams {
	pub chain_id: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetGasInfoResponseData {
	pub gas_info: Option<Vec<GasInfo>>,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GasInfo {
	pub chain_id: String,
	pub normal: String,
	pub fast: String,
	pub super_fast: String,
	pub normal_usd: String,
	pub fast_usd: String,
	pub super_fast_usd: String,
	pub normal_price: String,
	pub fast_price: String,
	pub super_fast_price: String,
}
pub type GetGasInfoResponse = ApiResponse<GetGasInfoResponseData>;

pub async fn get_gas_info_impl(
	client: &PumpxApiClient,

	access_token: &str,
	chain_id: u32,
) -> Result<GetGasInfoResponse, Error> {
	let endpoint = client.base_url.join("v1/trade/get_gas_info").unwrap();
	let params = GetGasInfoParams { chain_id };
	client
		.http_client
		.get(endpoint)
		.bearer_auth(access_token)
		.query(&params)
		.send()
		.await?
		.json()
		.await
}
