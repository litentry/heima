// Copyright 2020-2025 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

use crate::common::GAS_LIMIT;
use alloy::primitives::{Address, U256};
use hex::FromHex;
use log::error;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::ops::Mul;
use std::str::FromStr;

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
#[serde(rename_all = "camelCase")]
pub struct GetSwapRouteResponse {
	pub route_summary: RouteSummary,
	pub router_address: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RouteSummary {
	pub token_in: String,
	pub amount_in: String,
	pub amount_in_usd: String,
	pub token_in_market_price_available: bool,
	pub token_out: String,
	pub amount_out: String,
	pub amount_out_usd: String,
	pub token_out_market_price_available: bool,
	pub gas: String,
	pub gas_price: String,
	pub gas_usd: String,
	pub l1_fee_usd: String,
	pub extra_fee: ExtraFee,
	pub route: Vec<Vec<RouteItem>>,
	pub route_id: String,
	pub checksum: String,
	pub timestamp: i32,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExtraFee {
	pub fee_amount: String,
	pub charge_fee_by: String,
	pub is_in_bps: bool,
	pub fee_receiver: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteItem {
	pub pool: String,
	pub token_in: String,
	pub token_out: String,
	pub limit_return_amount: String,
	pub swap_amount: String,
	pub amount_out: String,
	pub exchange: String,
	pub pool_length: i32,
	pub pool_type: String,
	pub pool_extra: PoolExtra,
	pub extra: Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolExtra {
	pub block_number: i32,
	pub price_limit: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SwapRequest {
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
	#[serde(skip_serializing_if = "Option::is_none")]
	pub enable_gas_estimation: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub permit: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ignore_capped_slippage: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapResponse {
	pub amount_in: String,
	pub amount_in_usd: String,
	pub amount_out: String,
	pub amount_out_usd: String,
	pub gas: String,
	pub gas_usd: String,
	pub additional_cost_usd: String,
	pub additional_cost_message: String,
	pub output_change: OutputChange,
	pub data: String,
	pub router_address: String,
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
			final_gas = *GAS_LIMIT;
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
