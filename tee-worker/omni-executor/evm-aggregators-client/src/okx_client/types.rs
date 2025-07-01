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
use rust_decimal::prelude::{Decimal, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::ops::Mul;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SwapRequest {
	pub chain_id: String,
	pub amount: String,
	pub from_token_address: String,
	pub to_token_address: String,
	pub slippage: String,
	pub user_wallet_address: String,
	pub fee_percent: String,
	pub from_token_referrer_wallet_address: String,
	pub to_token_referrer_wallet_address: String,
	pub gas_level: String,
	pub dex_ids: String,
	pub auto_slippage: bool,
}

impl SwapRequest {
	/// Converts the SwapRequest into a vector of (&str, String) suitable for query params.
	pub fn convert_to_query_params(&self) -> Vec<(&'static str, String)> {
		let mut params = vec![
			("chainId", self.chain_id.clone()),
			("amount", self.amount.clone()),
			("fromTokenAddress", self.from_token_address.clone()),
			("toTokenAddress", self.to_token_address.clone()),
			("slippage", self.slippage.clone()),
			("userWalletAddress", self.user_wallet_address.clone()),
			("feePercent", self.fee_percent.clone()),
		];

		if !self.from_token_referrer_wallet_address.is_empty() {
			params.push((
				"fromTokenReferrerWalletAddress",
				self.from_token_referrer_wallet_address.clone(),
			));
		}
		if !self.to_token_referrer_wallet_address.is_empty() {
			params.push((
				"toTokenReferrerWalletAddress",
				self.to_token_referrer_wallet_address.clone(),
			));
		}
		if !self.gas_level.is_empty() {
			params.push(("gasLevel", self.gas_level.clone()));
		}
		if !self.dex_ids.is_empty() {
			params.push(("dexIds", self.dex_ids.clone()));
		}
		if self.auto_slippage {
			params.push(("autoSlippage", "true".to_string()));
		}

		params
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapResponse {
	pub router_result: RouterResult,
	pub tx: Tx,
}

impl SwapResponse {
	pub fn get_transaction_data(self) -> Result<(Vec<u8>, Address, U256, u64), ()> {
		let data = hex::decode(&self.tx.data).map_err(|e| {
			error!("Failed to decode transaction data: {}", e);
		})?;
		let value: U256 = U256::from_str(&self.tx.value).map_err(|e| {
			error!("Failed to deserialize to u256: {}", e);
		})?;
		let to = Address::from_hex(&self.tx.to)
			.map_err(|e| error!("Failed to decode hex to address: {}", e))?;

		let gas = Decimal::from_str(&self.tx.gas).map_err(|e| {
			error!("Failed to deserialize to decimal: {}", e);
		})?;

		let adjusted_gas = gas.mul(Decimal::new(1, 5)).to_u64().ok_or_else(|| {
			error!("Failed to convert gas to u128");
		})?;
		let mut final_gas = adjusted_gas;
		if final_gas > *GAS_LIMIT {
			final_gas = *GAS_LIMIT;
		}

		Ok((data, to, value, final_gas))
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouterResult {
	pub chain_id: String,
	pub dex_router_list: Vec<DexRouter>,
	pub estimate_gas_fee: String,
	pub from_token: Token,
	pub from_token_amount: String,
	pub price_impact_percentage: String,
	pub quote_compare_list: Vec<QuoteCompare>,
	pub to_token: Token,
	pub to_token_amount: String,
	pub trade_fee: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DexRouter {
	pub router: String,
	pub router_percent: String,
	pub sub_router_list: Vec<SubRouter>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubRouter {
	pub dex_protocol: Vec<DexProtocol>,
	pub from_token: Token,
	pub to_token: Token,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DexProtocol {
	pub dex_name: String,
	pub percent: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
	pub decimal: String,
	pub is_honey_pot: bool,
	pub tax_rate: String,
	pub token_contract_address: String,
	pub token_symbol: String,
	pub token_unit_price: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteCompare {
	pub amount_out: String,
	pub dex_logo: String,
	pub dex_name: String,
	pub trade_fee: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tx {
	pub data: String,
	pub from: String,
	pub gas: String,
	pub gas_price: String,
	pub max_priority_fee_per_gas: String,
	pub min_receive_amount: String,
	pub signature_data: Vec<String>,
	pub slippage: String,
	pub to: String,
	pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetGasPriceResp {
	pub normal: String,
	pub min: String,
	pub max: String,
	pub support_eip1559: bool,
	pub erc1599_protocol: Erc1599Protocol,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Erc1599Protocol {
	pub suggest_base_fee: String,
	pub base_fee: String,
	pub propose_priority_fee: String,
	pub safe_priority_fee: String,
	pub fast_priority_fee: String,
}

/// Converts a slippage value in basis points (u32) to a decimal string representation for OKX.
pub fn convert_slippage_to_okx(slippage: u32) -> String {
	let slippage_decimal = Decimal::from(slippage);
	let all_bp_decimal = Decimal::from(10000u32);
	(slippage_decimal / all_bp_decimal).to_string()
}

/// Maps an integer gas_type to OKX gas level string.
pub fn get_okx_gas_level(gas_type: i32) -> &'static str {
	match gas_type {
		1 => "slow",
		2 => "average",
		3 => "fast",
		_ => "average",
	}
}
