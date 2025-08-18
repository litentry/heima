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

use alloy::primitives::{Address, U256};
use crate::errors::{ClientError, ClientResult};
use crate::transaction_extractor::{TransactionDataExtractor, TransactionGas, extract_transaction_data};
use rust_decimal::prelude::Decimal;
use serde::{Deserialize, Serialize};

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

impl TransactionDataExtractor for SwapResponse {
	fn get_data(&self) -> &str {
		&self.tx.data
	}

	fn get_value(&self) -> &str {
		&self.tx.value
	}

	fn get_to_address(&self) -> &str {
		&self.tx.to
	}

	fn get_gas(&self) -> TransactionGas {
		TransactionGas::AsString(self.tx.gas.clone())
	}
}

impl SwapResponse {
	pub fn get_transaction_data(self) -> ClientResult<(Vec<u8>, Address, U256, u64)> {
		extract_transaction_data(self)
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
	slippage_decimal.checked_div(all_bp_decimal)
		.unwrap_or(Decimal::ZERO)
		.to_string()
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

pub struct SwapRequestBuilder {
	chain_id: Option<String>,
	amount: Option<String>,
	from_token_address: Option<String>,
	to_token_address: Option<String>,
	slippage: Option<String>,
	user_wallet_address: Option<String>,
	fee_percent: Option<String>,
	from_token_referrer_wallet_address: Option<String>,
	to_token_referrer_wallet_address: Option<String>,
	gas_level: Option<String>,
	dex_ids: Option<String>,
	auto_slippage: Option<bool>,
}

impl SwapRequestBuilder {
	pub fn new() -> Self {
		Self {
			chain_id: None,
			amount: None,
			from_token_address: None,
			to_token_address: None,
			slippage: None,
			user_wallet_address: None,
			fee_percent: None,
			from_token_referrer_wallet_address: None,
			to_token_referrer_wallet_address: None,
			gas_level: None,
			dex_ids: None,
			auto_slippage: None,
		}
	}

	pub fn chain_id(mut self, chain_id: impl Into<String>) -> Self {
		self.chain_id = Some(chain_id.into());
		self
	}

	pub fn amount(mut self, amount: impl Into<String>) -> Self {
		self.amount = Some(amount.into());
		self
	}

	pub fn from_token_address(mut self, address: impl Into<String>) -> Self {
		self.from_token_address = Some(address.into());
		self
	}

	pub fn to_token_address(mut self, address: impl Into<String>) -> Self {
		self.to_token_address = Some(address.into());
		self
	}

	pub fn slippage(mut self, slippage: impl Into<String>) -> Self {
		self.slippage = Some(slippage.into());
		self
	}

	pub fn user_wallet_address(mut self, address: impl Into<String>) -> Self {
		self.user_wallet_address = Some(address.into());
		self
	}

	pub fn fee_percent(mut self, fee: impl Into<String>) -> Self {
		self.fee_percent = Some(fee.into());
		self
	}

	pub fn gas_level(mut self, level: impl Into<String>) -> Self {
		self.gas_level = Some(level.into());
		self
	}

	pub fn dex_ids(mut self, ids: impl Into<String>) -> Self {
		self.dex_ids = Some(ids.into());
		self
	}

	pub fn auto_slippage(mut self, auto: bool) -> Self {
		self.auto_slippage = Some(auto);
		self
	}

	pub fn build(self) -> ClientResult<SwapRequest> {
		let chain_id = self.chain_id
			.ok_or_else(|| ClientError::MissingRequiredField { field: "chain_id".to_string() })?;
		
		let amount = self.amount
			.ok_or_else(|| ClientError::MissingRequiredField { field: "amount".to_string() })?;
		
		let from_token_address = self.from_token_address
			.ok_or_else(|| ClientError::MissingRequiredField { field: "from_token_address".to_string() })?;
		
		let to_token_address = self.to_token_address
			.ok_or_else(|| ClientError::MissingRequiredField { field: "to_token_address".to_string() })?;
		
		let slippage = self.slippage
			.ok_or_else(|| ClientError::MissingRequiredField { field: "slippage".to_string() })?;
		
		let user_wallet_address = self.user_wallet_address
			.ok_or_else(|| ClientError::MissingRequiredField { field: "user_wallet_address".to_string() })?;
		
		let fee_percent = self.fee_percent
			.ok_or_else(|| ClientError::MissingRequiredField { field: "fee_percent".to_string() })?;

		Ok(SwapRequest {
			chain_id,
			amount,
			from_token_address,
			to_token_address,
			slippage,
			user_wallet_address,
			fee_percent,
			from_token_referrer_wallet_address: self.from_token_referrer_wallet_address.unwrap_or_default(),
			to_token_referrer_wallet_address: self.to_token_referrer_wallet_address.unwrap_or_default(),
			gas_level: self.gas_level.unwrap_or_default(),
			dex_ids: self.dex_ids.unwrap_or_default(),
			auto_slippage: self.auto_slippage.unwrap_or(false),
		})
	}
}
