use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Deserialize_repr, Serialize_repr)]
#[allow(clippy::upper_case_acronyms)]
#[repr(u8)]
pub enum ChainType {
	EVM = 1,
	Solana = 2,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Wallet {
	pub user_wallet_address: String,
	pub chain_type: ChainType,
	pub wallet_index: u32,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewUser {
	pub invite_code: String,
	pub wallet_list: Vec<Wallet>,
	pub email: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
	code: u32,
	message: String,
	data: T,
}

#[derive(Deserialize)]
pub struct EmptyData {}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
	pub user_id: u32,
}
