use parity_scale_codec::{Codec, Decode, Encode};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

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
