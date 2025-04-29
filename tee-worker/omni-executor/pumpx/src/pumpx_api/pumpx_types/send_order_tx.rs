use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::pumpx_types::common::ApiResponse;

// /v3/trade/send_order_tx
#[derive(Deserialize, Serialize, Encode, Decode, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SendOrderTxBody {
	pub chain_id: u32,
	pub order_id: u32,
	pub tx_data: Vec<String>,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendOrderTxResponseData {
	pub tx_hash: Option<Vec<String>>,
}

pub type SendOrderTxResponse = ApiResponse<SendOrderTxResponseData>;
