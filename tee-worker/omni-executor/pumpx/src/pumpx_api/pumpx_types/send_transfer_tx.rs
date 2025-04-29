use crate::pumpx_types::common::ApiResponse;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

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

pub type SendTransferTxResponse = ApiResponse<SendTransferTxResponseData>;
