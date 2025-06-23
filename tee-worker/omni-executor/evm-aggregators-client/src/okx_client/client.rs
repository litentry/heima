use crate::okx_client::types::{GetGasPriceResp, SwapRequest, SwapResponse};
use log::error;
use reqwest::{Client, Error};
use async_trait::async_trait;

pub const BASIC_ENDPOINT: &str = "https://www.okx.com";
pub const GET_SWAP_PATH: &str = "/api/v5/dex/aggregator/swap";
pub const GET_GAS_PATH: &str = "/api/v5/wallet/pre-transaction/gas-price?chainIndex=56";

pub struct OkxClient {
	client: Client,
	access_token: String,
}

impl OkxClient {
	pub fn new(access_token: impl Into<String>) -> Self {
		OkxClient { client: Client::new(), access_token: access_token.into() }
	}
}

#[async_trait]
pub trait OkxSwap: Send + Sync {
	async fn swap(&self, swap_request: SwapRequest) -> Result<SwapResponse, Error>;
	async fn get_gas_price(&self, chain_id: u64) -> Result<GetGasPriceResp, Error>;
}

#[async_trait]
impl OkxSwap for OkxClient {
	async fn swap(&self, swap_request: SwapRequest) -> Result<SwapResponse, Error> {
		let query_params = swap_request.convert_to_query_params();

		let response = self
			.client
			.get(BASIC_ENDPOINT.to_owned() + GET_SWAP_PATH)
			.query(&query_params)
			.header("accept", "application/json")
			.header("content-type", "application/json")
			.bearer_auth(self.access_token.clone())
			.send()
			.await
			.map_err(|e| {
				error!("Failed to send add_wallet request: {:?}", e);
				e
			})?;

		let status = response.status();
		let response = response.error_for_status().map_err(|e| {
			error!("okx_swap request failed with status: {}, error: {:?}", status, e);
			e
		})?;
		response.json().await.map_err(|e| {
			error!("Failed to parse 1inch swap response: {:?}", e);
			e
		})
	}

	async fn get_gas_price(&self, chain_id: u64) -> Result<GetGasPriceResp, Error> {
		// TODO: Check endpoint for gas fees and all okx endpoints in general
		let response = self
			.client
			.get(BASIC_ENDPOINT.to_owned() + GET_GAS_PATH)
			.query(&[("gasPrice", format!("{}", chain_id))])
			.header("accept", "application/json")
			.header("content-type", "application/json")
			.bearer_auth(self.access_token.clone())
			.send()
			.await
			.map_err(|e| {
				error!("Failed to send add_wallet request: {:?}", e);
				e
			})?;

		let status = response.status();
		let response = response.error_for_status().map_err(|e| {
			error!("okx_swap request failed with status: {}, error: {:?}", status, e);
			e
		})?;
		response.json().await.map_err(|e| {
			error!("Failed to parse 1inch swap response: {:?}", e);
			e
		})
	}
}
