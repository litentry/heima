use crate::pumpx_types::common::{GasType, SwapType};
use serde::Serialize;

// /v3/trade/create_limit_order/
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateLimitOrderBody {
	pub request_id: u32,
	pub chain_id: u32,
	pub token_ca: String,
	pub amount: String,
	pub swap_type: SwapType,
	pub double_out: bool,
	pub token_cap: Option<String>,
	pub price_usd: Option<String>,
	pub trailing_percent: Option<String>,
	pub address: String,
	pub is_anti_mev: bool,
	pub is_auto_slippage: bool,
	pub gas_type: GasType,
	pub slippage: u32,
	pub wallet_index: u32,
}
