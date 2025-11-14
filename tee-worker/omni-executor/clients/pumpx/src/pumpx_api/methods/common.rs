use parity_scale_codec::{Codec, Decode, Encode};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Deserialize_repr, Serialize_repr, Debug, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
#[repr(u8)]
pub enum SwapType {
	Buy = 1,
	Sell = 2,
}

impl SwapType {
	pub fn to_number(&self) -> u8 {
		match self {
			SwapType::Buy => 1,
			SwapType::Sell => 2,
		}
	}
}

#[derive(Deserialize_repr, Serialize_repr, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[allow(clippy::upper_case_acronyms)]
#[repr(u8)]
pub enum GasType {
	Slow = 1,
	Medium = 2,
	Fast = 3,
}

impl GasType {
	pub fn to_number(&self) -> u8 {
		match self {
			GasType::Slow => 1,
			GasType::Medium => 2,
			GasType::Fast => 3,
		}
	}
}

#[derive(Deserialize, Serialize, Encode, Decode, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T: Codec> {
	pub code: u32,
	pub message: String,
	pub data: T,
}

impl<T: Codec> ApiResponse<T> {
	pub fn data(&self) -> &T {
		&self.data
	}
}

#[derive(Deserialize, Serialize, Encode, Decode, PartialEq, Eq, Debug, Clone, Default)]
pub struct EmptyResponse {}

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
