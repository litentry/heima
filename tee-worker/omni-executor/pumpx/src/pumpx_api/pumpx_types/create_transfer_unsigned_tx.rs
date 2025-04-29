use crate::pumpx_types::common::ApiResponse;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

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
	pub transfer_id: Option<u32>,
	pub chain_id: Option<u32>,
}

pub type CreateTransferUnsignedTxResponse = ApiResponse<CreateTransferUnsignedTxResponseData>;
