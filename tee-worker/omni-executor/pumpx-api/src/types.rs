use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Deserialize_repr, Serialize_repr)]
#[allow(clippy::upper_case_acronyms)]
#[repr(u8)]
pub enum ChainId {
	EVM = 1,
	Solana = 2,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Wallet {
	pub user_wallet_address: String,
	pub chain_type: ChainId,
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

#[derive(Deserialize_repr, Serialize_repr)]
#[allow(clippy::upper_case_acronyms)]
#[repr(u8)]
pub enum SwapType {
	Buy = 1,
	Sell = 2,
}

#[derive(Deserialize_repr, Serialize_repr)]
#[allow(clippy::upper_case_acronyms)]
#[repr(u8)]
pub enum GasType {
	Slow = 1,
	Medium = 2,
	Fast = 3,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewMarketOrder {
	pub chain_id: ChainId,
	pub token_ca: String,
	pub swap_type: SwapType,
	pub amount_in: String,
	pub double_out: bool,
	pub is_one_click: bool,
	pub address: String,
	pub is_anti_mev: i32,
	pub is_auto_slippage: i32,
	pub gas_type: GasType,
	pub slippage: u32,
	pub wallet_index: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketOrderUnsignedTx {
	pub order_id: String,
	pub tx_data: Vec<String>,
}
