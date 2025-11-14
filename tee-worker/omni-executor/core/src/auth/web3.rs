use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct HeimaMessagePayload {
	pub client_id: String,
	pub omni_account: String,
	pub message_code: String,
}
