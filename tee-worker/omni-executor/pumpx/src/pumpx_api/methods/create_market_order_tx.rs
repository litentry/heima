use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

use super::common::{ApiResponse, GasType, SwapType};
use crate::pumpx_api::PumpxApiClient;
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
	pub chain_id: Option<u32>,
	pub order_id: Option<u32>,
	pub tx_hash: Option<Vec<String>>,
}

pub type CreateMarketOrderTxResponse = ApiResponse<CreateMarketOrderTxResponseData>;

pub async fn create_market_order_tx_impl(
	client: &PumpxApiClient,

	access_token: &str,
	body: CreateMarketOrderTxBody,
) -> Result<CreateMarketOrderTxResponse, Error> {
	let endpoint = client.base_url.join("v3/trade/create_market_order_tx").unwrap();
	let response = client
		.http_client
		.post(endpoint)
		.bearer_auth(access_token)
		.json(&body)
		.send()
		.await
		.map_err(|e| {
			log::error!("Failed to send create_market_order_tx request: {:?}", e);
			e
		})?;

	let status = response.status();
	let response = response.error_for_status().map_err(|e| {
		log::error!("create_market_order_tx failed with status: {}, error: {:?}", status, e);
		e
	})?;

	response.json().await.map_err(|e| {
		log::error!("Failed to parse create_market_order_tx response: {:?}", e);
		e
	})
}
