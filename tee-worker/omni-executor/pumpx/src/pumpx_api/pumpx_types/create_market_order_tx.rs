use crate::pumpx_types::common::{ApiResponse, GasType, SwapType};
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

// /v3/trade/create_market_order_tx
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateMarketOrderTxBody {
	pub request_id: u32,
	pub chain_id: u32,
	pub token_ca: String,
	pub swap_type: SwapType,
	pub amount_in: String,
	pub double_out: bool,
	pub is_one_click: bool,
	pub address: String,
	pub is_anti_mev: bool,
	pub is_auto_slippage: bool,
	pub gas_type: GasType,
	pub slippage: u32,
	pub wallet_index: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateMarketOrderTxResponseData {
	pub chain_id: Option<u32>,
	pub order_id: Option<u32>,
	pub tx_hash: Option<Vec<String>>,
}

pub type CreateMarketOrderTxResponse = ApiResponse<CreateMarketOrderTxResponseData>;
