// Copyright 2020-2024 Trust Computing GmbH.
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
// along with Litentry. If not, see <https://www.gnu.org/licenses/>.

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

use core::result::Result;

use lc_common::{
	abort_strategy::{loop_with_abort_strategy, AbortStrategy, LoopControls},
	web3_token::{TokenAddress, TokenDecimals},
};
use lc_data_providers::{
	moralis::{BalanceApiList, MoralisClient},
	nodereal_jsonrpc::{FungibleApiList, GetTokenBalance20Param, Web3NetworkNoderealJsonrpcClient},
};
use log::error;

use crate::*;

// only support to get balance for non-native token
pub fn get_balance(
	addresses: Vec<(Web3Network, String)>,
	token_type: Web3TokenType,
	data_provider_config: &DataProviderConfig,
) -> Result<f64, Error> {
	let mut total_balance = 0_f64;

	loop_with_abort_strategy(
		addresses,
		|address| {
			let network = address.0;
			let token_address = token_type.get_token_address(network).unwrap_or_default();

			match network {
				Web3Network::Bsc | Web3Network::Ethereum | Web3Network::Combo => {
					let decimals = token_type.get_decimals(network);
					match network.create_nodereal_jsonrpc_client(data_provider_config) {
						Some(mut client) => {
							let param = GetTokenBalance20Param {
								contract_address: token_address.into(),
								address: address.1.clone(),
								block_number: "latest".into(),
							};
							let result = client.get_token_balance_20(&param, false);
							match result {
								Ok(balance) => {
									total_balance +=
										calculate_balance_with_decimals(balance, decimals);
									Ok(LoopControls::Continue)
								},
								Err(err) => Err(err.into_error_detail()),
							}
						},
						None => Ok(LoopControls::Continue),
					}
				},
				Web3Network::Solana => {
					let mut client = MoralisClient::new(data_provider_config);
					let result =
						client.get_solana_tokens_balance_by_wallet(address.1.clone(), false);

					match result {
						Ok(items) => match items.iter().find(|&item| item.mint == token_address) {
							Some(item) => match item.amount.parse::<f64>() {
								Ok(balance) => {
									total_balance += balance;
									Ok(LoopControls::Continue)
								},
								Err(err) => {
									error!("Failed to parse {} to f64: {}", item.amount, err);
									Err(Error::ParseError)
								},
							},
							None => Ok(LoopControls::Continue),
						},
						Err(err) => Err(err.into_error_detail()),
					}
				},
				Web3Network::Arbitrum | Web3Network::Polygon => {
					let decimals = token_type.get_decimals(network);

					let mut client = MoralisClient::new(data_provider_config);
					let result = client.get_evm_token_balance_by_wallet(
						address.1.clone(),
						token_address.into(),
						&network,
						false,
					);

					match result {
						Ok(items) => {
							if !items.is_empty() {
								match items[0].balance.parse::<u128>() {
									Ok(balance) => {
										total_balance +=
											calculate_balance_with_decimals(balance, decimals);

										Ok(LoopControls::Continue)
									},
									Err(err) => {
										error!(
											"Failed to parse {} to f64: {}",
											items[0].balance, err
										);
										Err(Error::ParseError)
									},
								}
							} else {
								Ok(LoopControls::Continue)
							}
						},
						Err(err) => Err(err.into_error_detail()),
					}
				},
				_ => Ok(LoopControls::Continue),
			}
		},
		AbortStrategy::FailFast::<fn(&_) -> bool>,
	)
	.map_err(|errors| errors[0].clone())?;

	Ok(total_balance)
}

pub fn calculate_balance_with_decimals(source_balance: u128, decimals: u64) -> f64 {
	let decimals_value = (if decimals == 0 { 1 } else { decimals }) as u128;
	(source_balance / decimals_value) as f64
		+ ((source_balance % decimals_value) as f64 / decimals_value as f64)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_calculate_balance_with_decimals_works() {
		assert_eq!(calculate_balance_with_decimals(100, 0), 100_f64);

		assert_eq!(calculate_balance_with_decimals(123, 100), 1.23_f64);

		assert_eq!(calculate_balance_with_decimals(123, 1000), 0.123_f64);

		assert_eq!(calculate_balance_with_decimals(0, 1000), 0_f64);
	}
}
