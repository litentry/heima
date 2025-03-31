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
	web3_nft::NftAddress,
};
use lc_data_providers::{
	moralis::{
		GetNftsByWalletParam, MoralisChainParam, MoralisClient, NftApiList as MoralisNftApiList,
	},
	nodereal_jsonrpc::{
		GetTokenBalance721Param, NftApiList as NoderealNftApiList, Web3NetworkNoderealJsonrpcClient,
	},
};
use litentry_primitives::ErrorDetail;

use crate::*;

// support ERC721/BEP721 nft token
pub fn has_nft_721(
	addresses: Vec<(Web3Network, String)>,
	nft_type: Web3NftType,
	data_provider_config: &DataProviderConfig,
) -> Result<bool, Error> {
	let mut result = false;

	loop_with_abort_strategy(
		addresses,
		|address| {
			let network = address.0;
			let token_address = nft_type.get_nft_address(network).unwrap_or_default();

			match network {
				Web3Network::Bsc | Web3Network::Ethereum => {
					match network.create_nodereal_jsonrpc_client(data_provider_config) {
						Some(mut client) => {
							let param = GetTokenBalance721Param {
								token_address: token_address.into(),
								account_address: address.1.clone(),
								block_number: "latest".into(),
							};
							match client.get_token_balance_721(&param, false) {
								Ok(balance) => {
									if balance > 0 {
										result = true;
										Ok(LoopControls::Break)
									} else {
										Ok(LoopControls::Continue)
									}
								},
								Err(err) => Err(err.into_error_detail()),
							}
						},
						None => Ok(LoopControls::Continue),
					}
				},
				Web3Network::Polygon => {
					match check_nft_via_moralis(
						network,
						address.1.clone(),
						token_address.into(),
						data_provider_config,
					) {
						Ok(r) => {
							if r {
								result = true;
								return Ok(LoopControls::Break);
							}
							Ok(LoopControls::Continue)
						},
						Err(err) => Err(err),
					}
				},
				_ => Ok(LoopControls::Continue),
			}
		},
		AbortStrategy::ContinueUntilEnd::<fn(&_) -> bool>,
	)
	.map_err(|errors| errors[0].clone())?;

	Ok(result)
}

pub fn check_nft_via_moralis(
	network: Web3Network,
	address: String,
	token_address: String,
	data_provider_config: &DataProviderConfig,
) -> Result<bool, Error> {
	let mut client = MoralisClient::new(data_provider_config);

	match network {
		Web3Network::Bsc | Web3Network::Ethereum | Web3Network::Polygon | Web3Network::Arbitrum => {
			let mut cursor: Option<String> = None;
			'inner: loop {
				let param = GetNftsByWalletParam {
					address: address.clone(),
					chain: MoralisChainParam::new(&network),
					token_addresses: Some(vec![token_address.clone()]),
					limit: None,
					cursor,
				};
				match client.get_nfts_by_wallet(&param, false) {
					Ok(resp) => {
						cursor = resp.cursor;
						for item in &resp.result {
							match item.amount.parse::<u32>() {
								Ok(balance) => {
									if balance > 0 {
										return Ok(true);
									}
								},
								Err(_) => return Err(ErrorDetail::ParseError),
							}
						}
					},
					Err(err) => return Err(err.into_error_detail()),
				}
				if cursor.is_none() {
					break 'inner;
				}
			}
			Ok(false)
		},
		_ => Ok(false),
	}
}

// support ERC1155/BEP1155 nft token
pub fn has_nft_1155(
	addresses: Vec<(Web3Network, String)>,
	nft_type: Web3NftType,
	data_provider_config: &DataProviderConfig,
) -> Result<bool, Error> {
	let mut result = false;

	loop_with_abort_strategy(
		addresses,
		|address| {
			let network = address.0;
			let token_address = nft_type.get_nft_address(network).unwrap_or_default();

			match network {
				Web3Network::Bsc
				| Web3Network::Ethereum
				| Web3Network::Polygon
				| Web3Network::Arbitrum => {
					match check_nft_via_moralis(
						network,
						address.1.clone(),
						token_address.into(),
						data_provider_config,
					) {
						Ok(r) => {
							if r {
								result = true;
								return Ok(LoopControls::Break);
							}
							Ok(LoopControls::Continue)
						},
						Err(err) => Err(err),
					}
				},
				_ => Ok(LoopControls::Continue),
			}
		},
		AbortStrategy::ContinueUntilEnd::<fn(&_) -> bool>,
	)
	.map_err(|errors| errors[0].clone())?;

	Ok(result)
}
