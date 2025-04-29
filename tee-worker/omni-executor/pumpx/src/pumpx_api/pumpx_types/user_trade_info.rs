use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::pumpx_types::common::ApiResponse;
use crate::pumpx_types::types::GasType;

// /v3/account/get_user_trade_info
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserTradeInfoResponseData {
	pub gas_type_base: Option<GasType>,
	pub gas_type_bsc: Option<GasType>,
	pub gas_type_eth: Option<GasType>,
	pub gas_type_sol: Option<GasType>,
	pub gas_type_omni: Option<GasType>,
	pub is_anti_mev: Option<bool>,
	pub is_auto_slippage: Option<bool>,
	pub slippage: Option<u32>,
	pub slippage_display: Option<String>,
}

pub type UserTradeInfoResponse = ApiResponse<UserTradeInfoResponseData>;
