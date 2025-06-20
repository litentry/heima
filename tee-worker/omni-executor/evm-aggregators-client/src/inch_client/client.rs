// This is where the client code of 1inch will reside
// It's main object will be to provide swapping
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use log::error;

pub struct InchClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapResponse {
    #[serde(rename = "dstAmount")]
    pub dst_amount: String,
    pub tx: TransactionData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionData {
    pub from: String,
    pub to: String,
    pub data: String,
    pub value: String,
    pub gas: u64,
    #[serde(rename = "gasPrice")]
    pub gas_price: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapRequest {
    pub chain_id: u64,
    pub amount: String,
    pub from_token_address: String,
    pub to_token_address: String,
    pub slippage: String,
    pub user_wallet_address: String,
    pub fee_percent: String,
    pub referrer: String,
    pub gas_level: u8,
    pub dex_ids: String
}

impl SwapRequest {

    // TODO, adding protocols is conditional
    pub fn convert_to_query_params(&self) -> Vec<(&'static str, &'_ str)> {
        vec![
            ("amount", &self.amount),
            ("src", &self.from_token_address),
            ("dst", &self.to_token_address),
            ("slippage", &self.slippage),
            ("from", &self.user_wallet_address),
            ("origin", &self.user_wallet_address),
            ("fee", &self.fee_percent),
            ("referrer", &self.referrer),
            ("protocols", &self.dex_ids),
        ]
    }
}

trait Swap {
    async fn swap(self, access_token: String, swap_request: SwapRequest) -> Result<SwapResponse, Error>;
}

impl Swap for InchClient {
    async fn swap(self, access_token: String, swap_request: SwapRequest) -> Result<SwapResponse, Error> {
        let query_params = swap_request.convert_to_query_params();

        let response = self.client
            .get(&self.base_url)
            .query(&query_params)
            .header("Content-Length", "0")
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