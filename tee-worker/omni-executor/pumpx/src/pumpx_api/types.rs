use parity_scale_codec::{Codec, Decode, Encode};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

// This file contains the type and struct definitions of pumpx-API (backend).
// For each API, we define the struct of:
// - (optional) request body: ...Body
// - response `data` section: ...ResponseData
//
// TODO: split this file to individual file per API

pub type UserConnectResponse = ApiResponse<UserConnectResponseData>;
pub type CreateMarketOrderUnsignedTxResponse = ApiResponse<CreateMarketOrderUnsignedTxResponseData>;
pub type SendOrderTxResponse = ApiResponse<SendOrderTxResponseData>;
pub type UserTradeInfoResponse = ApiResponse<UserTradeInfoResponseData>;
pub type OrderInfoResponse = ApiResponse<OrderInfoResponseData>;
pub type VerifyGoogleCodeResponse = ApiResponse<VerifyGoogleCodeResponseData>;
pub type AddWalletResponse = ApiResponse<EmptyResponse>;
pub type CreateTransferUnsignedTxResponse = ApiResponse<CreateTransferUnsignedTxResponseData>;
pub type SendTransferTxResponse = ApiResponse<SendTransferTxResponseData>;

// new API
pub type CreateMarketOrderTxResponse = ApiResponse<CreateMarketOrderTxResponseData>;
pub type CreateTransferTxResponse = ApiResponse<CreateTransferTxResponseData>;
pub type GetGasInfoResponse = ApiResponse<GetGasInfoResponseData>;

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

// the generic (standard) JSON-RPC response
#[derive(Deserialize, Serialize, Encode, Decode, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T: Codec> {
	code: u32,
	message: String,
	pub data: T,
}

impl<T: Codec> ApiResponse<T> {
	pub fn data(&self) -> &T {
		&self.data
	}
}

#[derive(Deserialize, Serialize, Encode, Decode, PartialEq, Eq, Debug, Clone, Default)]
pub struct EmptyResponse {}

// /v3/trade/create_transfer_unsigned_tx
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransferUnsignedTxBody {
	pub request_id: Option<u32>,
	pub chain_id: u32,
	pub wallet_index: u32,
	pub recipient_address: String,
	pub token_ca: String,
	pub amount: String,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransferUnsignedTxResponseData {
	pub tx_data: Option<Vec<String>>,
	pub transfer_id: u32,
	pub chain_id: u32,
}

// /v3/trade/send_transfer_tx
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendTransferTxBody {
	pub chain_id: u32,
	pub tx_data: Vec<String>,
	pub transfer_id: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendTransferTxResponseData {
	pub tx_hash: Option<Vec<String>>,
}

// /v3/account/user_connect
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserConnectBody {
	pub email: String,
	pub invite_code: Option<String>,
	pub google_code: String,
	pub user_id: String,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserConnectResponseData {
	pub user_id: String,
	pub google_auth_check: bool,
}

// /v3/trade/create_market_order_unsigned_tx
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateMarketOrderUnsignedTxBody {
	pub request_id: u32,
	pub chain_id: u32,
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

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateMarketOrderUnsignedTxResponseData {
	pub chain_id: u32,
	pub order_id: u32,
	pub tx_data: Vec<String>,
}

// /v3/trade/create_market_order_tx
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateMarketOrderTxBody {
	pub request_id: u32,
	pub chain_id: u32,
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

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateMarketOrderTxResponseData {
	pub chain_id: u32,
	pub order_id: u32,
	pub tx_hash: Vec<String>,
}

// /v3/trade/create_transfer_tx
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransferTxBody {
	pub request_id: Option<u32>,
	pub chain_id: u32,
	pub wallet_index: u32,
	pub recipient_address: String,
	pub token_ca: String,
	pub amount: String,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransferTxResponseData {
	pub tx_hash: String,
	pub transfer_id: u32,
	pub chain_id: u32,
}

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

// /v3/account/get_user_trade_info
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserTradeInfoResponseData {
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

// /v3/trade/create_limit_order/
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateLimitOrderBody {
	pub request_id: u32,
	pub chain_id: u32,
	pub token_ca: String,
	pub amount: String,
	pub swap_type: SwapType,
	pub double_out: bool,
	pub token_cap: Option<String>,
	pub price_usd: Option<String>,
	pub trailing_percent: Option<String>,
	pub address: String,
	pub is_anti_mev: bool,
	pub is_auto_slippage: bool,
	pub gas_type: GasType,
	pub slippage: u32,
	pub wallet_index: u32,
}

// Used in
// /v3/trade/create_cross_order
// /v3/trade/cross_fail
// /v3/trade/create_limit_order/
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderInfoResponseData {
	pub order_id: u32,
}

// /v3/account/verify_google_code
#[derive(Serialize)]
pub struct GoogleCode {
	pub google_code: String,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct VerifyGoogleCodeResponseData {
	pub result: bool,
}

// /v3/trade/create_cross_order
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateCrossOrderBody {
	pub request_id: u32,
	pub chain_id: u32,
	pub token_ca: String,
	pub swap_type: SwapType,
	pub is_one_click: bool,
	pub cross_info: Vec<CrossOrderInfo>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CrossOrderInfo {
	pub chain_id: u32,
	pub wallet_index: u32,
	pub address: String,
	pub token_ca: String,
	pub amount: String,
	pub usd: String,
}

// /v3/trade/cross_fail
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossFailBody {
	pub request_id: u32,
	pub fail_reason: String,
}

// /v1/trade/get_gas_info
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetGasInfoParams {
	pub chain_id: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetGasInfoResponseData {
	pub gas_info: GasInfo,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GasInfo {
	pub chain_id: String,
	pub normal: String,
	pub fast: String,
	pub super_fast: String,
	pub normal_usd: String,
	pub fast_usd: String,
	pub super_fast_usd: String,
	pub normal_price: String,
	pub fast_price: String,
	pub super_fast_price: String,
}

// /v3/account/get_account_user_id
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountUserIdParams {
	pub email: String,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountUserIdResponseData {
	pub user_id: String,
}
