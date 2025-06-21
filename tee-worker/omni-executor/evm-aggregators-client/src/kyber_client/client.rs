use log::error;
use reqwest::{Client, Error};
use crate::kyber_client::types::{GetSwapRouteRequest, GetSwapRouteResponse, SwapRequest, SwapResponse};

pub struct KyberClient {
    client: Client,
    endpoint: String,
}

pub(crate) trait KyberSwap {
    async fn get_swap_route(&self, access_token: String, swap_route_request: GetSwapRouteRequest) -> Result<GetSwapRouteResponse, Error>;
    async fn swap(&self, access_token: String, swap_request: SwapRequest) -> Result<SwapResponse, Error>;
}

impl KyberSwap for KyberClient {
    async fn get_swap_route(&self, access_token: String, swap_route_request: GetSwapRouteRequest) -> Result<GetSwapRouteResponse, Error> {
        let query = swap_route_request.convert_to_query_params();

        let response = self
            .client
            .get(self.endpoint.clone())
            .query(&query)
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .bearer_auth(access_token)
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

    async fn swap(&self, access_token: String, swap_request: SwapRequest) -> Result<SwapResponse, Error> {
        let response = self
            .client
            .post(self.endpoint.clone())
            .json(&swap_request) // <-- This serializes the struct to JSON and sets content-type
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .bearer_auth(access_token)
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
