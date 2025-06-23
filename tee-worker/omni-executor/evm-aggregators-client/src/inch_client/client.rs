// This is where the client code of 1inch will reside
// It's main object will be to provide swapping
use crate::inch_client::types::{SwapRequest, SwapResponse};
use log::error;
use reqwest::{Client, Error};
use async_trait::async_trait;

pub const BASIC_ENDPOINT: &str = "https://api.1inch.dev";
pub const GET_SWAP_PATH: &str = "/swap/v6.0/56/swap";

pub struct InchClient {
	client: Client,
	access_token: String,
}

impl InchClient {
	pub fn new(access_token: impl Into<String>) -> Self {
		InchClient { client: Client::new(), access_token: access_token.into() }
	}
}

#[async_trait]
pub trait InchSwap: Send + Sync {
	async fn swap(&self, swap_request: SwapRequest) -> Result<SwapResponse, Error>;
}

#[async_trait]
impl InchSwap for InchClient {
	async fn swap(&self, swap_request: SwapRequest) -> Result<SwapResponse, Error> {
		let query_params = swap_request.convert_to_query_params();

		let response = self
			.client
			.get(BASIC_ENDPOINT.to_owned() + GET_SWAP_PATH)
			.query(&query_params)
			.header("Content-Length", "0")
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
			error!("add_wallet request failed with status: {}, error: {:?}", status, e);
			e
		})?;
		response.json().await.map_err(|e| {
			error!("Failed to parse 1inch swap response: {:?}", e);
			e
		})
	}
}
