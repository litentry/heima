use crate::okx_client::types::{GetGasPriceResp, SwapRequest, SwapResponse};
use async_trait::async_trait;
use log::error;
use reqwest::{Client, Error};

use std::sync::Arc;

pub struct OkxClient {
	client: Client,
	access_token: Arc<str>,
	base_url: Arc<str>,
}

impl OkxClient {
	pub fn new(access_token: impl Into<String>, base_url: impl Into<String>) -> Self {
		OkxClient {
			client: Client::new(),
			access_token: Arc::from(access_token.into()),
			base_url: Arc::from(base_url.into()),
		}
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

		let path = format!("{}/api/v5/dex/aggregator/swap", self.base_url);
		let response = self
			.client
			.get(&path)
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
		let path = format!(
			"{}/api/v5/wallet/pre-transaction/gas-price?chainIndex={}",
			self.base_url, chain_id
		);
		let response = self
			.client
			.get(&path)
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
