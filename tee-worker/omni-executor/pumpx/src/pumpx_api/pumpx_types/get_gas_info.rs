use crate::pumpx_types::common::ApiResponse;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

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
