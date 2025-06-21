// This is where the client code of 1inch will reside
// It's main object will be to provide swapping
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use log::error;
use crate::inch_client::types::{SwapResponse, SwapRequest};

pub struct InchClient {
    client: Client,
    base_url: String,
    access_token: String,
}

pub trait InchSwap {
    async fn swap(&self, swap_request: SwapRequest) -> Result<SwapResponse, Error>;
}

impl InchSwap for InchClient {
    async fn swap(&self, swap_request: SwapRequest) -> Result<SwapResponse, Error> {
        let query_params = swap_request.convert_to_query_params();

        let response = self.client
            .get(&self.base_url)
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