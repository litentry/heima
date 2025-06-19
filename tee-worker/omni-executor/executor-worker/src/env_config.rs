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
	pub pumpx_worker_url: String,
	pub binance_api_key: String,
	pub binance_api_secret: String,
	pub binance_api_base_url: String,
}

impl EnvConfig {
	/// Creates a new EnvConfig by loading values from environment variables
	pub fn from_env() -> Self {
		let (env_map, sensitive_map) = Self::load_env_vars();

		Self::log_env_vars(&env_map, &sensitive_map);

		// Convert from HashMap to struct for better type safety and usage
		Self::from_map(env_map)
	}

	/// Loads environment variables into a HashMap
	fn load_env_vars() -> (HashMap<&'static str, Option<String>>, HashMap<&'static str, bool>) {
		const SENSITIVE: bool = true;
		const NOT_SENSITIVE: bool = false;

		let vars = [
			("OE_PARENTCHAIN_URL", "parentchain_url", NOT_SENSITIVE),
			("OE_ETHEREUM_URL", "ethereum_url", NOT_SENSITIVE),
			("OE_SOLANA_URL", "solana_url", NOT_SENSITIVE),
			("OE_BSC_URL", "bsc_url", NOT_SENSITIVE),
			("OE_BSC_TESTNET_URL", "bsc_testnet_url", NOT_SENSITIVE),
			("OE_ALCHEMY_KEY", "alchemy_key", SENSITIVE),
			("OE_PUMPX_SIGNER_URL", "pumpx_signer_url", NOT_SENSITIVE),
			("OE_PUMPX_API_BASE_URL", "pumpx_api_base_url", NOT_SENSITIVE),
			("OE_PUMPX_WORKER_URL", "pumpx_worker_url", NOT_SENSITIVE),
			("OE_BINANCE_API_KEY", "binance_api_key", SENSITIVE),
			("OE_BINANCE_API_SECRET", "binance_api_secret", SENSITIVE),
			("OE_BINANCE_API_BASE_URL", "binance_api_base_url", NOT_SENSITIVE),
		];

		let mut env_map = HashMap::new();
		let mut sensitive_map = HashMap::new();

		// Load environment variables
		for (env_name, var_name, is_sensitive) in vars {
			let value = Some(std::env::var(env_name).unwrap_or_default());
			env_map.insert(var_name, value);
			sensitive_map.insert(var_name, is_sensitive);
		}

		// Set default values for certain variables if they're empty
		let defaults = [
			("binance_api_base_url", DEFAULT_BINANCE_API_BASE_URL),
			("pumpx_api_base_url", DEFAULT_PUMPX_API_BASE_URL),
		];

		for (key, default_value) in defaults {
			if env_map.get(key).and_then(|v| v.as_ref()).is_none_or(|v| v.is_empty()) {
				env_map.insert(key, Some(default_value.to_string()));
			}
		}

		(env_map, sensitive_map)
	}

	/// Logs non-sensitive environment variables
	fn log_env_vars(
		env_map: &HashMap<&'static str, Option<String>>,
		sensitive_map: &HashMap<&'static str, bool>,
	) {
		// Log command line arguments
		let args_string = std::env::args().collect::<Vec<String>>().join(" ");

		for (name, value) in env_map {
			if let Some(is_sensitive) = sensitive_map.get(name) {
				if !*is_sensitive {
					if let Some(val) = value {
						info!("Environment variable {}: {}", name, val);
					}
				}
			}
		}

		info!("Executing: {}", args_string);
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
			if !alchemy_key.is_empty() && url.contains("alchemy") {
				format!("{}{}", url, alchemy_key)
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
			pumpx_worker_url: get_string("pumpx_worker_url"),
			binance_api_key: get_string("binance_api_key"),
			binance_api_secret: get_string("binance_api_secret"),
			binance_api_base_url: get_string("binance_api_base_url"),
		}
	}
}
