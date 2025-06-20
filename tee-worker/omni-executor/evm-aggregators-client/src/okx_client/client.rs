use log::error;
use reqwest::{Client, Error};
use crate::okx_client::types::{SwapRequest, SwapResponse};

pub struct OkxClient {
    client: Client,
    endpoint: String,
}

pub trait OkxSwap {
    async fn swap(&self, swap_request: SwapRequest, access_token: String) -> Result<SwapResponse, Error>;
}

impl OkxSwap for OkxClient {
    async fn swap(&self, swap_request: SwapRequest, access_token: String) -> Result<SwapResponse, Error> {
        let query_params = swap_request.convert_to_query_params();

        let response = self.client
            .get(self.endpoint.as_str())
            .query(&query_params)
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