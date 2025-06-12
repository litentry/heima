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
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::HashMap;
use tracing::info;

const DEFAULT_PUMPX_API_BASE_URL: &str = "https://api.pumpx.ai";
const DEFAULT_BINANCE_API_BASE_URL: &str = "https://api.binance.com";

/// Configuration struct for environment variables used by the executor worker
pub struct EnvConfig {
	pub parentchain_url: String,
	pub ethereum_url: String,
	pub solana_url: String,
	pub bsc_url: String,
	pub bsc_testnet_url: Option<String>,
	pub pumpx_signer_url: String,
	pub pumpx_api_base_url: String,
	pub binance_api_key: String,
	pub binance_api_secret: String,
	pub binance_api_base_url: String,
	pub worker_base_url: String,
}

impl EnvConfig {
	/// Creates a new EnvConfig by loading values from environment variables
	pub fn from_env() -> Self {
		let env_map = Self::load_env_vars();

		Self::log_env_vars(&env_map);

		// Convert from HashMap to struct for better type safety and usage
		Self::from_map(env_map)
	}

	/// Loads environment variables into a HashMap
	fn load_env_vars() -> HashMap<&'static str, Option<String>> {
		let vars = [
			("OE_PARENTCHAIN_URL", "parentchain_url"),
			("OE_ETHERUM_URL", "ethereum_url"),
			("OE_SOLANA_URL", "solana_url"),
			("OE_BSC_URL", "bsc_url"),
			("OE_BSC_TESTNET_URL", "bsc_testnet_url"),
			("OE_ALCHEMY_KEY", "alchemy_key"),
			("OE_PUMPX_SIGNER_URL", "pumpx_signer_url"),
			("OE_PUMPX_API_BASE_URL", "pumpx_api_base_url"),
			("OE_BINANCE_API_KEY", "binance_api_key"),
			("OE_BINANCE_API_SECRET", "binance_api_secret"),
			("OE_BINANCE_API_BASE_URL", "binance_api_base_url"),
			("OE_WORKER_BASE_URL", "worker_base_url"),
		];

		let mut env_map = HashMap::new();

		// Load environment variables
		for (env_name, var_name) in vars {
			let value = Some(std::env::var(env_name).unwrap_or_default());
			env_map.insert(var_name, value);
		}

		// Set default values for certain variables if they're empty
		let defaults = [
			("binance_api_base_url", DEFAULT_BINANCE_API_BASE_URL),
			("pumpx_api_base_url", DEFAULT_PUMPX_API_BASE_URL),
		];

		for (key, default_value) in defaults {
			if env_map.get(key).and_then(|v| v.as_ref()).map_or(true, |v| v.is_empty()) {
				env_map.insert(key, Some(default_value.to_string()));
			}
		}

		env_map
	}

	/// Logs non-sensitive environment variables
	fn log_env_vars(env_map: &HashMap<&'static str, Option<String>>) {
		// Log command line arguments
		let args_string = std::env::args().collect::<Vec<String>>().join(" ");
		info!("Executing: {}", args_string);

		for (name, value) in env_map {
			if *name != "alchemy_key" && *name != "binance_api_key" && *name != "binance_api_secret"
			{
				if let Some(val) = value {
					if !val.is_empty() {
						info!("Environment variable {}: {}", name, val);
					}
				}
			}
		}
	}

	/// Converts a HashMap of environment variables to an EnvConfig struct
	fn from_map(env_map: HashMap<&'static str, Option<String>>) -> Self {
		let get_string = |key: &str| -> String {
			env_map
				.get(key)
				.and_then(|v| v.as_ref())
				.map_or_else(String::new, |v| v.clone())
		};

		let ethereum_url = get_string("ethereum_url");
		let solana_url = get_string("solana_url");
		let bsc_url = get_string("bsc_url");
		let bsc_testnet_url =
			env_map.get("bsc_testnet_url").and_then(|v| v.clone()).filter(|v| !v.is_empty());
		let alchemy_key = get_string("alchemy_key");

		// Append alchemy_key to URLs in the format: https://base-url/v2/{alchemy_key}
		let append_key = |url: &str| -> String {
			if !alchemy_key.is_empty() && !url.is_empty() {
				format!("{}/v2/{}", url.trim_end_matches('/'), alchemy_key)
			} else {
				url.to_string()
			}
		};

		EnvConfig {
			parentchain_url: get_string("parentchain_url"),
			ethereum_url: append_key(&ethereum_url),
			solana_url: append_key(&solana_url),
			bsc_url: append_key(&bsc_url),
			bsc_testnet_url: bsc_testnet_url.as_ref().map(|url| append_key(url)),
			pumpx_signer_url: get_string("pumpx_signer_url"),
			pumpx_api_base_url: get_string("pumpx_api_base_url"),
			binance_api_key: get_string("binance_api_key"),
			binance_api_secret: get_string("binance_api_secret"),
			binance_api_base_url: get_string("binance_api_base_url"),
			worker_base_url: get_string("worker_base_url"),
		}
	}
}
