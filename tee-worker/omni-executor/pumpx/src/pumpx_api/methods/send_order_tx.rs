use parity_scale_codec::{Decode, Encode};
use reqwest::Error;
use serde::{Deserialize, Serialize};

use super::common::ApiResponse;
use crate::pumpx_api::PumpxApiClient;

// /v3/trade/send_order_tx
#[derive(Deserialize, Serialize, Encode, Decode, Debug, PartialEq)]
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

pub type SendOrderTxResponse = ApiResponse<SendOrderTxResponseData>;

pub async fn send_order_tx_impl(
	client: &PumpxApiClient,
	access_token: &str,
	body: SendOrderTxBody,
) -> Result<SendOrderTxResponse, Error> {
	let endpoint = client.base_url.join("v3/trade/send_order_tx").unwrap();
	let response = client
		.http_client
		.post(endpoint)
		.bearer_auth(access_token)
		.json(&body)
		.send()
		.await
		.map_err(|e| {
			log::error!("Failed to send market order transaction: {:?}", e);
			e
		})?;

	let status = response.status();
	let response = response.error_for_status().map_err(|e| {
		log::error!("Market order transaction failed with status: {}, error: {:?}", status, e);
		e
	})?;

	response.json().await.map_err(|e| {
		log::error!("Failed to parse market order transaction response: {:?}", e);
		e
	})
}
