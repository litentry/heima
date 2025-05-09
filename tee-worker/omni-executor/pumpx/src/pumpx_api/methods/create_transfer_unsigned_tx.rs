use parity_scale_codec::{Decode, Encode};
use reqwest::Error;
use serde::{Deserialize, Serialize};

use crate::pumpx_api::PumpxApiClient;

use super::common::ApiResponse;

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

#[allow(clippy::too_many_arguments)]
pub async fn create_transfer_unsigned_tx_impl(
	client: &PumpxApiClient,

	access_token: &str,
	body: CreateTransferUnsignedTxBody,
	language: Option<String>,
) -> Result<CreateTransferUnsignedTxResponse, Error> {
	let endpoint = client.base_url.join("v3/trade/create_transfer_unsigned_tx").unwrap();
	let response = client
		.http_client
		.post(endpoint)
		.header("X-Language", language.unwrap_or("en".to_string()))
		.bearer_auth(access_token)
		.json(&body)
		.send()
		.await
		.map_err(|e| {
			log::error!("Failed to send create_transfer_unsigned_tx request: {:?}", e);
			e
		})?;

	let status = response.status();
	let response = response.error_for_status().map_err(|e| {
		log::error!(
			"create_transfer_unsigned_tx request failed with status: {}, error: {:?}",
			status,
			e
		);
		e
	})?;

	response.json().await.map_err(|e| {
		log::error!("Failed to parse create_transfer_unsigned_tx response: {:?}", e);
		e
	})
}
