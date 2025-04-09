use parity_scale_codec::{Codec, Decode, Encode};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub type UserConnectResponse = ApiResponse<ConnectedUser>;

pub type MarketOrderUnsignedTxResponse = ApiResponse<MarketOrderUnsignedTx>;

pub type MarketOrderTxResponse = ApiResponse<TxData>;

pub type UserTradeInfoResponse = ApiResponse<UserTradeInfo>;

pub type OrderInfoResponse = ApiResponse<OrderInfo>;

pub type VerifyGoogleCodeResponse = ApiResponse<DataResult>;

pub type AddWalletResponse = ApiResponse<EmptyResponse>;

#[derive(Deserialize, Serialize, Encode, Decode, PartialEq, Eq, Debug, Clone, Default)]
pub struct EmptyResponse {}

#[derive(Deserialize_repr, Serialize_repr, Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
#[repr(u8)]
pub enum ChainId {
	EVM = 1,
	Solana = 2,
}

impl ChainId {
	pub fn to_number(&self) -> u8 {
		match self {
			ChainId::EVM => 1,
			ChainId::Solana => 2,
		}
	}
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectUser {
	pub email: String,
	pub invite_code: Option<String>,
	pub google_code: Option<String>,
}

#[derive(Deserialize, Serialize, Encode, Decode, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedUser {
	pub user_id: String,
	pub google_auth_check: bool,
}

#[derive(Deserialize, Serialize, Encode, Decode, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T: Encode> {
	code: u32,
	message: String,
	pub data: T,
}

impl<T: Codec> ApiResponse<T> {
	pub fn data(&self) -> &T {
		&self.data
	}
}

#[derive(Deserialize_repr, Serialize_repr, Debug)]
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

#[derive(Deserialize_repr, Serialize_repr, Encode)]
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewMarketOrder {
	pub request_id: u32,
	pub chain_id: ChainId,
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

#[derive(Deserialize, Encode)]
#[serde(rename_all = "camelCase")]
pub struct MarketOrderUnsignedTx {
	pub order_id: String,
	pub tx_data: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketOrderTx {
	pub order_id: String,
	pub tx_data: Vec<String>,
	pub chain_id: ChainId,
}

#[derive(Deserialize, Encode)]
#[serde(rename_all = "camelCase")]
pub struct TxData {
	pub tx_hash: String,
}

#[derive(Deserialize, Encode)]
#[serde(rename_all = "camelCase")]
pub struct UserTradeInfo {
	pub gas_type_base: GasType,
	pub gas_type_bsc: GasType,
	pub gas_type_eth: GasType,
	pub gas_type_sol: GasType,
	pub gas_type_omni: GasType,
	pub is_anti_mev: bool,
	pub is_auto_slippage: bool,
	pub slippage: u32,
	pub slippage_display: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewLimitOrder {
	pub request_id: u32,
	pub chain_id: ChainId,
	pub token_ca: String,
	pub amount: String,
	pub swap_type: SwapType,
	pub double_out: bool,
	pub token_cap: String,
	pub price_usd: String,
	pub trailing_percent: String,
	pub address: String,
	pub is_anti_mev: bool,
	pub is_auto_slippage: bool,
	pub gas_type: GasType,
	pub slippage: u32,
	pub wallet_index: u32,
}

#[derive(Deserialize, Encode)]
#[serde(rename_all = "camelCase")]
pub struct OrderInfo {
	pub order_id: String,
}

#[derive(Serialize)]
pub struct GoogleCode {
	pub google_code: String,
}

#[derive(Deserialize, Encode)]
pub struct DataResult {
	pub result: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCrossOrderData {
	pub request_id: u32,
	pub chain_id: ChainId,
	pub info: CrossOrderInfo,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossOrderInfo {
	pub chain_id: ChainId,
	pub wallet_index: u32,
	pub address: String,
	pub token_ca: String,
	pub amount: String,
	pub usd: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossOrderFailData {
	pub request_id: u32,
	pub fail_reason: String,
}
