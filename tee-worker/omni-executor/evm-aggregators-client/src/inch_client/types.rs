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
use rust_decimal::prelude::{Decimal, FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::ops::Mul;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapResponse {
	pub dst_amount: String,
	pub tx: TransactionData,
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

		let gas = Decimal::from_u64(self.tx.gas).ok_or_else(|| {
			error!("Failed to deserialize to decimal");
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

pub fn convert_slippage_to_inch(slippage: u32) -> String {
	// inch max 50% slippage
	if slippage > 5000 {
		return "50".to_string();
	}

	Decimal::from(slippage)
		.checked_div(Decimal::from(100))
		.map(|d| d.normalize().to_string())
		.unwrap_or_else(|| "0".to_string())
}
