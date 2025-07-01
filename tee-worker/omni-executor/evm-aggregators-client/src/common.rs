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

use lazy_static::lazy_static;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const WRAPPED_MAINNET_ETH: &[u8] = b"0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
pub const WRAPPPED_BASE_ETH: &[u8] = b"0x4200000000000000000000000000000000000006";
pub const GOERLI_WETH: &[u8] = b"0xB4FBF271143F4FBf7B91A5ded31805e42b2208d6";
pub const WRAPPED_BNB: &[u8] = b"0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c";
pub const NATIVE_TOKEN: &[u8] = b"0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

pub const NATIVE_ADDRESS: &str = "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

pub const SERVICE_FEE_PERCENT: &str = "1";
pub const CROSS_SERVICE_FEE_PERCENT: &str = "1.1";
pub const SERVICE_FEE_BPS: &str = "100";
pub const CROSS_SERVICE_FEE_BPS: &str = "110";

pub const KYBER_SWAP_APPROVE_ADDRESS: &str = "0x6131B5fae19EA4f9D964eAc0408E4408b66337b5";
pub const INCH_SWAP_APPROVE_ADDRESS: &str = "0x111111125421cA6dc452d289314280a0f8842A65";
pub const OKX_SWAP_APPROVE_ADDRESS: &str = "0x2c34A2Fb1d0b4f55de51E1d0bDEfaDDce6b7cDD6";

lazy_static! {
	pub static ref GAS_LIMIT: u64 = 450_000u64;
	pub static ref GWEI_DECIMAL: Decimal = Decimal::from(1_000_000_000u64);
	pub static ref WEI_DECIMAL: Decimal = Decimal::from(1_000_000_000_000_000_000u128);
}

lazy_static! {
	pub static ref KYBER_SWAP_DEX_ID_MAP: HashMap<String, String> = {
		let mut map = HashMap::new();
		map.insert(SWAP_NAME_UNIV2.to_string(), "uniswap".to_string());
		map.insert(SWAP_NAME_UNIV3.to_string(), "uniswapv3".to_string());
		map.insert(SWAP_NAME_PANCAKE_V2.to_string(), "pancake".to_string());
		map.insert(SWAP_NAME_PANCAKE_V3.to_string(), "pancake-v3".to_string());
		map
	};
	pub static ref INCH_DEX_IDS_MAP: HashMap<u64, HashMap<String, String>> = {
		let mut map = HashMap::new();

		let mut eth_map = HashMap::new();
		eth_map.insert("UniswapV2".to_string(), "UNISWAP_V2".to_string());
		eth_map.insert("UniswapV3".to_string(), "UNISWAP_V3".to_string());
		eth_map.insert("PancakeV2".to_string(), "ETHEREUM_PANCAKESWAP_V2".to_string());
		eth_map.insert("PancakeV3".to_string(), "PANCAKESWAP_V3".to_string());
		map.insert(1, eth_map); // ETH Chain ID

		let mut bsc_map = HashMap::new();
		bsc_map.insert("UniswapV2".to_string(), "".to_string());
		bsc_map.insert("UniswapV3".to_string(), "BSC_UNISWAP_V3".to_string());
		bsc_map.insert("PancakeV2".to_string(), "PANCAKESWAP_V2".to_string());
		bsc_map.insert("PancakeV3".to_string(), "BSC_PANCAKESWAP_V3".to_string());
		map.insert(56, bsc_map); // BSC Chain ID

		let mut base_map = HashMap::new();
		base_map.insert("UniswapV2".to_string(), "BASE_UNISWAP_V2".to_string());
		base_map.insert("UniswapV3".to_string(), "BASE_UNISWAP_V3".to_string());
		base_map.insert("PancakeV2".to_string(), "BASE_PANCAKESWAP_V2".to_string());
		base_map.insert("PancakeV3".to_string(), "BASE_PANCAKESWAP_V3".to_string());
		map.insert(8453, base_map); // BASE Chain ID

		map
	};
}

lazy_static! {
	pub static ref DECIMALS_TO_VALUE: HashMap<u8, i64> = {
		let mut map = HashMap::new();
		map.insert(0, 1);
		map.insert(1, 10_i64.pow(1));
		map.insert(2, 10_i64.pow(2));
		map.insert(3, 10_i64.pow(3));
		map.insert(4, 10_i64.pow(4));
		map.insert(5, 10_i64.pow(5));
		map.insert(6, 10_i64.pow(6));
		map.insert(7, 10_i64.pow(7));
		map.insert(8, 10_i64.pow(8));
		map.insert(9, 10_i64.pow(9));
		map.insert(10, 10_i64.pow(10));
		map.insert(11, 10_i64.pow(11));
		map.insert(12, 10_i64.pow(12));
		map.insert(13, 10_i64.pow(13));
		map.insert(14, 10_i64.pow(14));
		map.insert(15, 10_i64.pow(15));
		map.insert(16, 10_i64.pow(16));
		map.insert(17, 10_i64.pow(17));
		map.insert(18, 10_i64.pow(18));
		map
	};
}

pub const SWAP_NAME_UNIV2: &str = "Uniswap V2";
pub const SWAP_NAME_UNIV3: &str = "Uniswap V3";
pub const SWAP_NAME_PANCAKE_V2: &str = "PancakeSwap V2";
pub const SWAP_NAME_PANCAKE_V3: &str = "PancakeSwap V3";
pub const SWAP_NAME_FOUR_MEME_V2: &str = "FourMeme V2";

lazy_static! {
	pub static ref OKX_DEX_IDS_MAP: HashMap<String, String> = {
		let mut map = HashMap::new();
		map.insert(SWAP_NAME_UNIV2.to_string(), "OKX_UNIV2".to_string());
		map.insert(SWAP_NAME_UNIV3.to_string(), "OKX_UNIV3".to_string());
		map.insert(SWAP_NAME_PANCAKE_V2.to_string(), "OKX_PANCAKE_V2".to_string());
		map.insert(SWAP_NAME_PANCAKE_V3.to_string(), "OKX_PANCAKE_V3".to_string());
		map
	};
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMarketTx {
	pub chain_id: u64,
	pub user_wallet_address: String,
	pub amount_in: String,
	pub slippage: u32,
	pub gas_type: i32,
	pub trade_pool_name: String,
	pub in_decimal: u8,
	pub in_token_ca: String,
	pub out_token_ca: String,
	pub is_pre_cross: bool,
}

pub fn is_native_token(token: &[u8]) -> bool {
	if token == WRAPPED_MAINNET_ETH
		|| token == WRAPPED_BNB
		|| token == WRAPPPED_BASE_ETH
		|| token == GOERLI_WETH
		|| token == NATIVE_TOKEN
	{
		return true;
	}
	false
}
