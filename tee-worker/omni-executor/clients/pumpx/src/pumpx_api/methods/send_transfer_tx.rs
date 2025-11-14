use parity_scale_codec::{Decode, Encode};
use reqwest::Error;
use serde::{Deserialize, Serialize};
use tracing::error;

use super::common::ApiResponse;
use crate::pumpx_api::PumpxApiClient;

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

pub async fn send_transfer_tx_impl(
	client: &PumpxApiClient,

	access_token: &str,
	body: SendTransferTxBody,
	language: Option<String>,
) -> Result<SendTransferTxResponse, Error> {
	let endpoint = client.base_url.join("v3/trade/send_transfer_tx").unwrap();
	let response = client
		.http_client
		.post(endpoint)
		.header("X-Language", language.unwrap_or("en".to_string()))
		.bearer_auth(access_token)
		.json(&body)
		.send()
		.await
		.map_err(|e| {
			error!("Failed to send send_transfer_tx request: {:?}", e);
			e
		})?;

	let status = response.status();
	let response = response.error_for_status().map_err(|e| {
		error!("send_transfer_tx request failed with status: {}, error: {:?}", status, e);
		e
	})?;

	response.json().await.map_err(|e| {
		error!("Failed to parse send_transfer_tx response: {:?}", e);
		e
	})
}
