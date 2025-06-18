use log::error;
use reqwest::{Client, RequestBuilder, Error};
use serde::{Serialize, Deserialize};
use serde_json::Value;

pub struct KyberClient {
    client: Client,
    endpoint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicResp {
    pub code: i32,
    pub data: Value,
    pub msg: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetSwapRouteRequest {
    pub chain_id: u64,
    pub amount: String,
    pub from_token_address: String,
    pub to_token_address: String,
    pub fee_bps: String,
    pub referrer: String,
    pub dex_ids: String,
    pub is_from_token_referrer: bool,
}

impl GetSwapRouteRequest {
    pub fn convert_to_query_params(&self) -> Vec<(&str, String)> {
        let charge_fee_by = if self.is_from_token_referrer {
            "currency_in"
        } else {
            "currency_out"
        };

        let mut params = vec![
            ("amountIn", self.amount.clone()),
            ("tokenIn", self.from_token_address.clone()),
            ("tokenOut", self.to_token_address.clone()),
            ("gasInclude", "true".to_string()),
            ("feeAmount", self.fee_bps.clone()),
            ("chargeFeeBy", charge_fee_by.to_string()),
            ("isInBps", "true".to_string()),
            ("feeReceiver", self.referrer.clone()),
        ];

        if !self.dex_ids.is_empty() {
            params.push(("includedSources", self.dex_ids.clone()));
        }

        params
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetSwapRouteResponse {
    #[serde(rename = "routeSummary")]
    pub route_summary: RouteSummary,

    #[serde(rename = "routerAddress")]
    pub router_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteSummary {
    #[serde(rename = "tokenIn")]
    pub token_in: String,
    #[serde(rename = "amountIn")]
    pub amount_in: String,
    #[serde(rename = "amountInUsd")]
    pub amount_in_usd: String,
    #[serde(rename = "tokenInMarketPriceAvailable")]
    pub token_in_market_price_available: bool,
    #[serde(rename = "tokenOut")]
    pub token_out: String,
    #[serde(rename = "amountOut")]
    pub amount_out: String,
    #[serde(rename = "amountOutUsd")]
    pub amount_out_usd: String,
    #[serde(rename = "tokenOutMarketPriceAvailable")]
    pub token_out_market_price_available: bool,
    #[serde(rename = "gas")]
    pub gas: String,
    #[serde(rename = "gasPrice")]
    pub gas_price: String,
    #[serde(rename = "gasUsd")]
    pub gas_usd: String,
    #[serde(rename = "l1FeeUsd")]
    pub l1_fee_usd: String,
    #[serde(rename = "extraFee")]
    pub extra_fee: ExtraFee,
    pub route: Vec<Vec<RouteItem>>,
    #[serde(rename = "routeID")]
    pub route_id: String,
    pub checksum: String,
    pub timestamp: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtraFee {
    #[serde(rename = "feeAmount")]
    pub fee_amount: String,
    #[serde(rename = "chargeFeeBy")]
    pub charge_fee_by: String,
    #[serde(rename = "isInBps")]
    pub is_in_bps: bool,
    #[serde(rename = "feeReceiver")]
    pub fee_receiver: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RouteItem {
    pub pool: String,
    #[serde(rename = "tokenIn")]
    pub token_in: String,
    #[serde(rename = "tokenOut")]
    pub token_out: String,
    #[serde(rename = "limitReturnAmount")]
    pub limit_return_amount: String,
    #[serde(rename = "swapAmount")]
    pub swap_amount: String,
    #[serde(rename = "amountOut")]
    pub amount_out: String,
    pub exchange: String,
    #[serde(rename = "poolLength")]
    pub pool_length: i32,
    #[serde(rename = "poolType")]
    pub pool_type: String,
    #[serde(rename = "poolExtra")]
    pub pool_extra: PoolExtra,
    pub extra: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PoolExtra {
    #[serde(rename = "blockNumber")]
    pub block_number: i32,
    #[serde(rename = "priceLimit")]
    pub price_limit: String, // or a BigUint if using a crate like `num`
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapRequest {
    #[serde(rename = "routeSummary")]
    pub route_summary: RouteSummary,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<i64>,
    #[serde(rename = "slippageTolerance", skip_serializing_if = "Option::is_none")]
    pub slippage_bps: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referral: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(rename = "enableGasEstimation", skip_serializing_if = "Option::is_none")]
    pub enable_gas_estimation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit: Option<String>,
    #[serde(rename = "ignoreCappedSlippage", skip_serializing_if = "Option::is_none")]
    pub ignore_capped_slippage: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapResponse {
    #[serde(rename = "amountIn")]
    pub amount_in: String,
    #[serde(rename = "amountInUsd")]
    pub amount_in_usd: String,
    #[serde(rename = "amountOut")]
    pub amount_out: String,
    #[serde(rename = "amountOutUsd")]
    pub amount_out_usd: String,
    pub gas: String,
    #[serde(rename = "gasUsd")]
    pub gas_usd: String,
    #[serde(rename = "additionalCostUsd")]
    pub additional_cost_usd: String,
    #[serde(rename = "additionalCostMessage")]
    pub additional_cost_message: String,
    #[serde(rename = "outputChange")]
    pub output_change: OutputChange,
    pub data: String,
    #[serde(rename = "routerAddress")]
    pub router_address: String,
    #[serde(rename = "transactionValue")]
    pub transaction_value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputChange {
    pub amount: String,
    pub percent: i32,
    pub level: i32,
}

trait KyberSwap {
    async fn get_swap_route(&self, access_token: String, swap_route_request: GetSwapRouteRequest) -> Result<GetSwapRouteResponse, Error>;
    async fn swap(&self, access_token: String, swap_request: crate::inch_client::SwapRequest) -> Result<crate::inch_client::SwapResponse, Error>;
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

    async fn swap(&self, access_token: String, swap_request: crate::inch_client::SwapRequest) -> Result<crate::inch_client::SwapResponse, Error> {
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
