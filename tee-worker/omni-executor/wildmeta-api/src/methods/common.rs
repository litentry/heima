use parity_scale_codec::{Codec, Decode, Encode};
use serde::{Deserialize, Serialize};

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
