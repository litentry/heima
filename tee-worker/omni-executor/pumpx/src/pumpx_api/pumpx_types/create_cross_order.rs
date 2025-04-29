use crate::pumpx_types::common::SwapType;

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
