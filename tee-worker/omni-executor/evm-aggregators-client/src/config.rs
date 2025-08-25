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
use lazy_static::lazy_static;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ================================
// Chain Configuration
// ================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
	pub chain_id: u64,
	pub name: String,
	pub native_token_address: String,
	pub dex_mappings: DexMappings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexMappings {
	pub inch: HashMap<String, String>,
	pub kyber: HashMap<String, String>,
	pub okx: HashMap<String, String>,
}

// ================================
// Token Configuration
// ================================

pub const NATIVE_TOKEN_ADDRESS: &str = "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

// Supported native token addresses
pub const WRAPPED_MAINNET_ETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
pub const WRAPPED_BASE_ETH: &str = "0x4200000000000000000000000000000000000006";
pub const GOERLI_WETH: &str = "0xB4FBF271143F4FBf7B91A5ded31805e42b2208d6";
pub const WRAPPED_BNB: &str = "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c";

// ================================
// Fee Configuration
// ================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeConfig {
	pub service_fee_percent: String,
	pub cross_service_fee_percent: String,
	pub service_fee_bps: String,
	pub cross_service_fee_bps: String,
}

impl Default for FeeConfig {
	fn default() -> Self {
		Self {
			service_fee_percent: "1".to_string(),
			cross_service_fee_percent: "1.1".to_string(),
			service_fee_bps: "100".to_string(),
			cross_service_fee_bps: "110".to_string(),
		}
	}
}

// ================================
// DEX Configuration
// ================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexConfig {
	pub approve_addresses: ApproveAddresses,
	pub gas_limit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveAddresses {
	pub kyber_swap: String,
	pub inch: String,
	pub okx: String,
}

impl Default for DexConfig {
	fn default() -> Self {
		Self {
			approve_addresses: ApproveAddresses {
				kyber_swap: "0x6131B5fae19EA4f9D964eAc0408E4408b66337b5".to_string(),
				inch: "0x111111125421cA6dc452d289314280a0f8842A65".to_string(),
				okx: "0x2c34A2Fb1d0b4f55de51E1d0bDEfaDDce6b7cDD6".to_string(),
			},
			gas_limit: 450_000,
		}
	}
}

// ================================
// Main Configuration
// ================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmConfig {
	pub chains: HashMap<u64, ChainConfig>,
	pub dex: DexConfig,
	pub fees: FeeConfig,
	pub decimal_multipliers: HashMap<u8, i64>,
}

impl EvmConfig {
	pub fn validate(&self) -> ClientResult<()> {
		// Validate chain configurations
		for (chain_id, config) in &self.chains {
			if *chain_id != config.chain_id {
				return Err(ClientError::Internal {
					message: format!(
						"Chain ID mismatch: key {} != config.chain_id {}",
						chain_id, config.chain_id
					),
				});
			}

			// Validate DEX mappings are not empty
			for (pool_name, dex_id) in &config.dex_mappings.inch {
				if dex_id.is_empty() {
					return Err(ClientError::Internal {
						message: format!(
							"Empty DEX ID for 1inch pool '{}' on chain {}",
							pool_name, chain_id
						),
					});
				}
			}
		}

		// Validate decimal multipliers
		for (decimals, multiplier) in &self.decimal_multipliers {
			if *decimals > 18 {
				return Err(ClientError::Internal {
					message: format!("Decimals {} exceeds maximum of 18", decimals),
				});
			}
			if *multiplier <= 0 {
				return Err(ClientError::Internal {
					message: format!("Invalid multiplier {} for decimals {}", multiplier, decimals),
				});
			}
		}

		Ok(())
	}

	pub fn get_chain_config(&self, chain_id: u64) -> ClientResult<&ChainConfig> {
		self.chains.get(&chain_id).ok_or(ClientError::UnsupportedChainId { chain_id })
	}

	pub fn get_decimal_multiplier(&self, decimals: u8) -> ClientResult<i64> {
		self.decimal_multipliers
			.get(&decimals)
			.copied()
			.ok_or(ClientError::UnsupportedDecimals { decimals })
	}

	pub fn is_supported_chain(&self, chain_id: u64) -> bool {
		self.chains.contains_key(&chain_id)
	}
}

impl Default for EvmConfig {
	fn default() -> Self {
		let mut chains = HashMap::new();

		// Ethereum configuration
		let mut ethereum_inch = HashMap::new();
		ethereum_inch.insert("UniswapV2".to_string(), "UNISWAP_V2".to_string());
		ethereum_inch.insert("UniswapV3".to_string(), "UNISWAP_V3".to_string());
		ethereum_inch.insert("PancakeV2".to_string(), "ETHEREUM_PANCAKESWAP_V2".to_string());
		ethereum_inch.insert("PancakeV3".to_string(), "PANCAKESWAP_V3".to_string());

		let mut ethereum_kyber = HashMap::new();
		ethereum_kyber.insert("Uniswap V2".to_string(), "uniswap".to_string());
		ethereum_kyber.insert("Uniswap V3".to_string(), "uniswapv3".to_string());
		ethereum_kyber.insert("PancakeSwap V2".to_string(), "pancake".to_string());
		ethereum_kyber.insert("PancakeSwap V3".to_string(), "pancake-v3".to_string());

		let mut ethereum_okx = HashMap::new();
		ethereum_okx.insert("Uniswap V2".to_string(), "OKX_UNIV2".to_string());
		ethereum_okx.insert("Uniswap V3".to_string(), "OKX_UNIV3".to_string());
		ethereum_okx.insert("PancakeSwap V2".to_string(), "OKX_PANCAKE_V2".to_string());
		ethereum_okx.insert("PancakeSwap V3".to_string(), "OKX_PANCAKE_V3".to_string());

		chains.insert(
			1,
			ChainConfig {
				chain_id: 1,
				name: "Ethereum".to_string(),
				native_token_address: WRAPPED_MAINNET_ETH.to_string(),
				dex_mappings: DexMappings {
					inch: ethereum_inch,
					kyber: ethereum_kyber,
					okx: ethereum_okx,
				},
			},
		);

		// BSC configuration
		let mut bsc_inch = HashMap::new();
		bsc_inch.insert("UniswapV2".to_string(), "BSC_UNISWAP_V2".to_string());
		bsc_inch.insert("UniswapV3".to_string(), "BSC_UNISWAP_V3".to_string());
		bsc_inch.insert("PancakeV2".to_string(), "PANCAKESWAP_V2".to_string());
		bsc_inch.insert("PancakeV3".to_string(), "BSC_PANCAKESWAP_V3".to_string());

		let mut bsc_kyber = HashMap::new();
		bsc_kyber.insert("Uniswap V2".to_string(), "uniswap".to_string());
		bsc_kyber.insert("Uniswap V3".to_string(), "uniswapv3".to_string());
		bsc_kyber.insert("PancakeSwap V2".to_string(), "pancake".to_string());
		bsc_kyber.insert("PancakeSwap V3".to_string(), "pancake-v3".to_string());

		let mut bsc_okx = HashMap::new();
		bsc_okx.insert("Uniswap V2".to_string(), "OKX_UNIV2".to_string());
		bsc_okx.insert("Uniswap V3".to_string(), "OKX_UNIV3".to_string());
		bsc_okx.insert("PancakeSwap V2".to_string(), "OKX_PANCAKE_V2".to_string());
		bsc_okx.insert("PancakeSwap V3".to_string(), "OKX_PANCAKE_V3".to_string());

		chains.insert(
			56,
			ChainConfig {
				chain_id: 56,
				name: "BSC".to_string(),
				native_token_address: WRAPPED_BNB.to_string(),
				dex_mappings: DexMappings { inch: bsc_inch, kyber: bsc_kyber, okx: bsc_okx },
			},
		);

		// Base configuration
		let mut base_inch = HashMap::new();
		base_inch.insert("UniswapV2".to_string(), "BASE_UNISWAP_V2".to_string());
		base_inch.insert("UniswapV3".to_string(), "BASE_UNISWAP_V3".to_string());
		base_inch.insert("PancakeV2".to_string(), "BASE_PANCAKESWAP_V2".to_string());
		base_inch.insert("PancakeV3".to_string(), "BASE_PANCAKESWAP_V3".to_string());

		let mut base_kyber = HashMap::new();
		base_kyber.insert("Uniswap V2".to_string(), "uniswap".to_string());
		base_kyber.insert("Uniswap V3".to_string(), "uniswapv3".to_string());
		base_kyber.insert("PancakeSwap V2".to_string(), "pancake".to_string());
		base_kyber.insert("PancakeSwap V3".to_string(), "pancake-v3".to_string());

		let mut base_okx = HashMap::new();
		base_okx.insert("Uniswap V2".to_string(), "OKX_UNIV2".to_string());
		base_okx.insert("Uniswap V3".to_string(), "OKX_UNIV3".to_string());
		base_okx.insert("PancakeSwap V2".to_string(), "OKX_PANCAKE_V2".to_string());
		base_okx.insert("PancakeSwap V3".to_string(), "OKX_PANCAKE_V3".to_string());

		chains.insert(
			8453,
			ChainConfig {
				chain_id: 8453,
				name: "Base".to_string(),
				native_token_address: WRAPPED_BASE_ETH.to_string(),
				dex_mappings: DexMappings { inch: base_inch, kyber: base_kyber, okx: base_okx },
			},
		);

		// Decimal multipliers
		let mut decimal_multipliers = HashMap::new();
		for i in 0..=18 {
			decimal_multipliers.insert(i, 10_i64.pow(i as u32));
		}

		Self { chains, dex: DexConfig::default(), fees: FeeConfig::default(), decimal_multipliers }
	}
}

// Global configuration instance
lazy_static! {
	pub static ref GLOBAL_CONFIG: EvmConfig = EvmConfig::default();
	pub static ref GWEI_DECIMAL: Decimal = Decimal::from(1_000_000_000u64);
	pub static ref WEI_DECIMAL: Decimal = Decimal::from(1_000_000_000_000_000_000u128);
}

// Utility function to check if a token is native
pub fn is_native_token(token: &str) -> bool {
	token == WRAPPED_MAINNET_ETH
		|| token == WRAPPED_BNB
		|| token == WRAPPED_BASE_ETH
		|| token == GOERLI_WETH
		|| token == NATIVE_TOKEN_ADDRESS
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_config_validation() {
		let config = EvmConfig::default();
		assert!(config.validate().is_ok());
	}

	#[test]
	fn test_chain_config_lookup() {
		let config = EvmConfig::default();

		// Test valid chain
		let ethereum_config = config.get_chain_config(1).unwrap();
		assert_eq!(ethereum_config.name, "Ethereum");

		// Test invalid chain
		assert!(config.get_chain_config(999).is_err());
	}

	#[test]
	fn test_decimal_multiplier_lookup() {
		let config = EvmConfig::default();

		// Test valid decimals
		assert_eq!(config.get_decimal_multiplier(0).unwrap(), 1);
		assert_eq!(config.get_decimal_multiplier(6).unwrap(), 1_000_000);
		assert_eq!(config.get_decimal_multiplier(18).unwrap(), 1_000_000_000_000_000_000);

		// Test invalid decimals
		assert!(config.get_decimal_multiplier(19).is_err());
	}

	#[test]
	fn test_native_token_detection() {
		assert!(is_native_token(WRAPPED_MAINNET_ETH));
		assert!(is_native_token(WRAPPED_BNB));
		assert!(is_native_token(WRAPPED_BASE_ETH));
		assert!(is_native_token(NATIVE_TOKEN_ADDRESS));

		assert!(!is_native_token("0xA0b86a33E6417c8f7851efA37A9f7F1A5d8C8f6E")); // Random address
	}

	#[test]
	fn test_no_empty_dex_mappings() {
		let config = EvmConfig::default();

		for (chain_id, chain_config) in &config.chains {
			for (pool_name, dex_id) in &chain_config.dex_mappings.inch {
				assert!(
					!dex_id.is_empty(),
					"Empty DEX ID for 1inch pool '{}' on chain {}",
					pool_name,
					chain_id
				);
			}
		}
	}

	#[test]
	fn test_bsc_uniswap_v2_mapping() {
		let config = EvmConfig::default();
		let bsc_config = config.get_chain_config(56).unwrap();

		let uniswap_v2_id = bsc_config.dex_mappings.inch.get("UniswapV2").unwrap();
		assert_eq!(uniswap_v2_id, "BSC_UNISWAP_V2", "BSC UniswapV2 should have proper mapping");
	}
}
