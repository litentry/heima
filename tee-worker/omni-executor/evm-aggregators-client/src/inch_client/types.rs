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

use crate::errors::{ClientError, ClientResult};
use crate::transaction_extractor::{
	extract_transaction_data, TransactionDataExtractor, TransactionGas,
};
use alloy::primitives::{Address, U256};
use rust_decimal::prelude::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapResponse {
	pub dst_amount: String,
	pub tx: TransactionData,
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
		TransactionGas::AsU64(self.tx.gas)
	}
}

impl SwapResponse {
	pub fn get_transaction_data(self) -> ClientResult<(Vec<u8>, Address, U256, u64)> {
		extract_transaction_data(self)
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionData {
	pub from: String,
	pub to: String,
	pub data: String,
	pub value: String,
	pub gas: u64,
	pub gas_price: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
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
	pub dex_ids: String,
}

impl SwapRequest {
	pub fn convert_to_query_params(&self) -> Vec<(&str, String)> {
		let mut params = vec![
			("amount", self.amount.clone()),
			("src", self.from_token_address.clone()),
			("dst", self.to_token_address.clone()),
			("slippage", self.slippage.clone()),
			("from", self.user_wallet_address.clone()),
			("origin", self.user_wallet_address.clone()),
			("fee", self.fee_percent.clone()),
			("referrer", self.referrer.clone()),
		];
		if !self.dex_ids.is_empty() {
			params.push(("protocols", self.dex_ids.clone()));
		}
		params
	}
}

/// Converts slippage from basis points (bps) to percentage format expected by 1inch API.
///
/// This function takes slippage tolerance in basis points (where 100 bps = 1%) and converts
/// it to the decimal percentage format that the 1inch aggregator API expects.
///
/// # Arguments
/// * `slippage` - Slippage tolerance in basis points (e.g., 50 = 0.5%, 100 = 1%, 500 = 5%)
///
/// # Returns
/// String representation of slippage percentage (e.g., "0.5", "1", "5")
///
/// # Examples
/// * 50 bps → "0.5" (0.5%)
/// * 100 bps → "1" (1%)  
/// * 500 bps → "5" (5%)
/// * 5000 bps → "50" (50% - maximum allowed)
/// * 6000 bps → "50" (capped at 50% maximum)
pub fn convert_slippage_to_inch(slippage: u32) -> String {
	// 1inch API has a maximum slippage tolerance of 50%
	if slippage > 5000 {
		return "50".to_string();
	}

	// Convert basis points to percentage: divide by 100
	// e.g., 150 bps ÷ 100 = 1.5%
	Decimal::from(slippage)
		.checked_div(Decimal::from(100))
		.map(|d| d.normalize().to_string())
		.unwrap_or_else(|| "0".to_string())
}

pub struct SwapRequestBuilder {
	chain_id: Option<u64>,
	amount: Option<String>,
	from_token_address: Option<String>,
	to_token_address: Option<String>,
	slippage: Option<String>,
	user_wallet_address: Option<String>,
	fee_percent: Option<String>,
	referrer: Option<String>,
	dex_ids: Option<String>,
}

impl Default for SwapRequestBuilder {
	fn default() -> Self {
		Self::new()
	}
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
			referrer: None,
			dex_ids: None,
		}
	}

	pub fn chain_id(mut self, chain_id: u64) -> Self {
		self.chain_id = Some(chain_id);
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

	pub fn referrer(mut self, referrer: impl Into<String>) -> Self {
		self.referrer = Some(referrer.into());
		self
	}

	pub fn dex_ids(mut self, dex_ids: impl Into<String>) -> Self {
		self.dex_ids = Some(dex_ids.into());
		self
	}

	pub fn build(self) -> ClientResult<SwapRequest> {
		let chain_id = self
			.chain_id
			.ok_or_else(|| ClientError::MissingRequiredField { field: "chain_id".to_string() })?;

		let amount = self
			.amount
			.ok_or_else(|| ClientError::MissingRequiredField { field: "amount".to_string() })?;

		let from_token_address = self.from_token_address.ok_or_else(|| {
			ClientError::MissingRequiredField { field: "from_token_address".to_string() }
		})?;

		let to_token_address = self.to_token_address.ok_or_else(|| {
			ClientError::MissingRequiredField { field: "to_token_address".to_string() }
		})?;

		let slippage = self
			.slippage
			.ok_or_else(|| ClientError::MissingRequiredField { field: "slippage".to_string() })?;

		let user_wallet_address = self.user_wallet_address.ok_or_else(|| {
			ClientError::MissingRequiredField { field: "user_wallet_address".to_string() }
		})?;

		Ok(SwapRequest {
			chain_id,
			amount,
			from_token_address,
			to_token_address,
			slippage,
			user_wallet_address,
			fee_percent: self.fee_percent.unwrap_or_default(),
			referrer: self.referrer.unwrap_or_default(),
			dex_ids: self.dex_ids.unwrap_or_default(),
			gas_level: Default::default(),
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_convert_slippage_to_inch_normal_values() {
		// Test common slippage values
		assert_eq!(convert_slippage_to_inch(50), "0.5", "50 bps should convert to 0.5%");
		assert_eq!(convert_slippage_to_inch(100), "1", "100 bps should convert to 1%");
		assert_eq!(convert_slippage_to_inch(150), "1.5", "150 bps should convert to 1.5%");
		assert_eq!(convert_slippage_to_inch(300), "3", "300 bps should convert to 3%");
		assert_eq!(convert_slippage_to_inch(500), "5", "500 bps should convert to 5%");
	}

	#[test]
	fn test_convert_slippage_to_inch_edge_cases() {
		// Test edge cases
		assert_eq!(convert_slippage_to_inch(0), "0", "0 bps should convert to 0%");
		assert_eq!(convert_slippage_to_inch(1), "0.01", "1 bps should convert to 0.01%");
		assert_eq!(convert_slippage_to_inch(25), "0.25", "25 bps should convert to 0.25%");
		assert_eq!(convert_slippage_to_inch(1000), "10", "1000 bps should convert to 10%");
	}

	#[test]
	fn test_convert_slippage_to_inch_maximum_cap() {
		// Test maximum slippage cap (50%)
		assert_eq!(
			convert_slippage_to_inch(5000),
			"50",
			"5000 bps should convert to 50% (at limit)"
		);
		assert_eq!(convert_slippage_to_inch(5001), "50", "5001 bps should be capped at 50%");
		assert_eq!(convert_slippage_to_inch(6000), "50", "6000 bps should be capped at 50%");
		assert_eq!(convert_slippage_to_inch(10000), "50", "10000 bps should be capped at 50%");
		assert_eq!(convert_slippage_to_inch(u32::MAX), "50", "Maximum u32 should be capped at 50%");
	}

	#[test]
	fn test_convert_slippage_to_inch_precision() {
		// Test decimal precision handling
		assert_eq!(convert_slippage_to_inch(75), "0.75", "75 bps should convert to 0.75%");
		assert_eq!(convert_slippage_to_inch(125), "1.25", "125 bps should convert to 1.25%");
		assert_eq!(convert_slippage_to_inch(333), "3.33", "333 bps should convert to 3.33%");
		assert_eq!(convert_slippage_to_inch(1234), "12.34", "1234 bps should convert to 12.34%");
	}

	#[test]
	fn test_convert_slippage_to_inch_real_world_scenarios() {
		// Test realistic trading scenarios
		assert_eq!(convert_slippage_to_inch(30), "0.3", "Low slippage: 30 bps = 0.3%");
		assert_eq!(convert_slippage_to_inch(200), "2", "Medium slippage: 200 bps = 2%");
		assert_eq!(convert_slippage_to_inch(800), "8", "High slippage: 800 bps = 8%");
		assert_eq!(convert_slippage_to_inch(2500), "25", "Very high slippage: 2500 bps = 25%");
	}
}
