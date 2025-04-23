use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct HeimaMessagePayload {
	pub message_code: String,
}
