use std::ops::Mul;
use std::str::FromStr;
use alloy::primitives::{Address, U256};
use hex::FromHex;
use log::error;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::common::GAS_LIMIT;

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicResp {
	pub code: i32,
	pub data: Value,
	pub msg: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
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
		let charge_fee_by =
			if self.is_from_token_referrer { "currency_in" } else { "currency_out" };

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

#[derive(Debug, Serialize, Deserialize, Default)]
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
#[derive(Default)]
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

#[derive(Debug, Serialize, Deserialize, Default)]
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

impl SwapResponse {
	pub fn get_transaction_data(self) -> Result<(Vec<u8>, Address, U256, u64), ()> {
		let data = hex::decode(&self.data).map_err(|e| {
			error!("Failed to decode transaction data: {}", e);
		})?;
		let value: U256 = U256::from_str(&self.transaction_value).map_err(|e| {
			error!("Failed to deserialize to u256: {}", e);
		})?;
		let to = Address::from_hex(&self.router_address)
			.map_err(|e| error!("Failed to decode hex to address: {}", e))?;

		let gas = Decimal::from_str(&self.gas).map_err(|e| {
			error!("Failed to deserialize to decimal: {}", e);
		})?;

		let adjusted_gas = gas.mul(Decimal::new(1, 5)).to_u64().ok_or_else(|| {
			error!("Failed to convert gas to u128: {}", self.gas);
		})?;
		let mut final_gas = adjusted_gas;
		if final_gas > *GAS_LIMIT {
			final_gas = GAS_LIMIT.clone();
		}

		Ok((data, to, value, final_gas))
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputChange {
	pub amount: String,
	pub percent: i32,
	pub level: i32,
}