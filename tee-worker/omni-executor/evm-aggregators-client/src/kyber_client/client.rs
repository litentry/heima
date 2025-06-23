use crate::kyber_client::types::{
	GetSwapRouteRequest, GetSwapRouteResponse, SwapRequest, SwapResponse,
};
use async_trait::async_trait;
use log::error;
use reqwest::{Client, Error};

pub const BASIC_ENDPOINT: &str = "https://aggregator-api.kyberswap.com";
pub const GET_SWAP_ROUTE_PATH: &str = "/56/api/v1/routes";
pub const GET_SWAP_PATH: &str = "/56/api/v1/route/build";

pub struct KyberClient {
	client: Client,
	access_token: String,
}

impl KyberClient {
	pub fn new(access_token: impl Into<String>) -> Self {
		KyberClient { client: Client::new(), access_token: access_token.into() }
	}
}

#[async_trait]
pub trait KyberSwap: Send + Sync {
	async fn get_swap_route(
		&self,
		swap_route_request: GetSwapRouteRequest,
	) -> Result<GetSwapRouteResponse, Error>;
	async fn swap(&self, swap_request: SwapRequest) -> Result<SwapResponse, Error>;
}

#[async_trait]
impl KyberSwap for KyberClient {
	async fn get_swap_route(
		&self,
		swap_route_request: GetSwapRouteRequest,
	) -> Result<GetSwapRouteResponse, Error> {
		let query = swap_route_request.convert_to_query_params();

		let response = self
			.client
			.get(BASIC_ENDPOINT.to_owned() + GET_SWAP_ROUTE_PATH)
			.query(&query)
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

	async fn swap(&self, swap_request: SwapRequest) -> Result<SwapResponse, Error> {
		let response = self
			.client
			.post(BASIC_ENDPOINT.to_owned() + GET_SWAP_PATH)
			.json(&swap_request)
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
			error!("swap request failed with status: {}, error: {:?}", status, e);
			e
		})?;
		response.json().await.map_err(|e| {
			error!("Failed to parse 1inch swap response: {:?}", e);
			e
		})
	}
}
