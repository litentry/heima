use log::error;
use reqwest::{Client, Error};
use crate::okx_client::types::{SwapRequest, SwapResponse, GetGasPriceResp};

pub struct OkxClient {
    client: Client,
    endpoint: String,
    access_token: String,
}

pub trait OkxSwap {
    async fn swap(&self, swap_request: SwapRequest) -> Result<SwapResponse, Error>;
    async fn get_gas_price(&self, chain_id: u64) -> Result<GetGasPriceResp, Error>;
}

impl OkxSwap for OkxClient {
    async fn swap(&self, swap_request: SwapRequest) -> Result<SwapResponse, Error> {
        let query_params = swap_request.convert_to_query_params();

        let response = self.client
            .get(self.endpoint.as_str())
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
        let response = self.client
            .get(self.endpoint.as_str())
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