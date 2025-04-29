use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::pumpx_types::common::ApiResponse;

// Used in
// /v3/trade/create_cross_order
// /v3/trade/cross_fail
// /v3/trade/create_limit_order/
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderInfoResponseData {
	pub order_id: Option<u32>,
}

pub type OrderInfoResponse = ApiResponse<OrderInfoResponseData>;
