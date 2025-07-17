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
use std::str::FromStr;
use tracing::info;

#[derive(Debug, Clone, PartialEq)]
pub enum MailerType {
	Sendgrid,
	Console,
}

impl FromStr for MailerType {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"sendgrid" => Ok(MailerType::Sendgrid),
			"console" => Ok(MailerType::Console),
			_ => Err(format!("Invalid mailer type: {}", s)),
		}
	}
}

const DEFAULT_MAILER_TYPE: &str = "sendgrid";
const DEFAULT_MAILER_API_HOST: &str = ""; // Optional
const DEFAULT_MAILER_API_KEY: &str = "";
const DEFAULT_MAILER_FROM_EMAIL: &str = "no-reply@example.com";
const DEFAULT_MAILER_FROM_NAME: &str = "Heima Verify";
const DEFAULT_GOOGLE_CLIENT_ID: &str = "";
const DEFAULT_GOOGLE_CLIENT_SECRET: &str = "";
const DEFAULT_PARENTCHAIN_URL: &str = "wss://rpc.paseo-parachain.heima.network";
const DEFAULT_ETHEREUM_URL: &str = "https://eth-mainnet.g.alchemy.com/v2/";
const DEFAULT_SOLANA_URL: &str = "https://solana-mainnet.g.alchemy.com/v2/";
const DEFAULT_BSC_URL: &str = "https://bnb-mainnet.g.alchemy.com/v2/";
const DEFAULT_BSC_TESTNET_URL: &str = "https://bnb-testnet.g.alchemy.com/v2/"; // Optional
const DEFAULT_ARBITRUM_URL: &str = "https://arb-mainnet.g.alchemy.com/v2/";
const DEFAULT_ARBITRUM_TESTNET_URL: &str = "https://arb-sepolia.g.alchemy.com/v2/"; // Optional
const DEFAULT_HYPEREVM_URL: &str = "https://rpc.hyperevm.org";
const DEFAULT_HYPEREVM_TESTNET_URL: &str = "https://testnet-rpc.hyperevm.org"; // Optional
const DEFAULT_PUMPX_API_BASE_URL: &str = "https://test-dex-api.heima.network";
const DEFAULT_PUMPX_SIGNER_URL: &str = "https://dev-dex-signer.heima.network";
const DEFAULT_PUMPX_WORKER_URL: &str = "wss://dev-dex-worker.heima.network";
const DEFAULT_BINANCE_API_KEY: &str = "";
const DEFAULT_BINANCE_API_SECRET: &str = "";
const DEFAULT_BINANCE_API_BASE_URL: &str = "https://api.binance.com";
const DEFAULT_OMNI_FACTORY_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
const DEFAULT_ENTRY_POINT_ADDRESS: &str = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789";

#[derive(Debug, Clone)]
pub struct ConfigLoader {
	pub mailer_type: MailerType,
	pub mailer_api_host: Option<String>,
	pub mailer_api_key: String,
	pub mailer_from_email: String,
	pub mailer_from_name: String,
	pub google_client_id: String,
	pub google_client_secret: String,
	pub parentchain_url: String,
	pub ethereum_url: String,
	pub solana_url: String,
	pub bsc_url: String,
	pub bsc_testnet_url: Option<String>,
	pub arbitrum_url: String,
	pub arbitrum_testnet_url: Option<String>,
	pub hyperevm_url: String,
	pub hyperevm_testnet_url: Option<String>,
	pub pumpx_signer_url: String,
	pub pumpx_api_base_url: String,
	pub pumpx_worker_url: String,
	pub binance_api_key: String,
	pub binance_api_secret: String,
	pub binance_api_base_url: String,
	pub omni_factory_address: String,
	pub entry_point_address: String,
}

struct EnvVar {
	env_key: &'static str,
	default: &'static str,
	sensitive: bool,
	optional: bool,
}

fn get_env_value(var: &EnvVar) -> Option<String> {
	let val = std::env::var(var.env_key).unwrap_or_else(|_| var.default.to_string());
	if !var.sensitive && (!val.is_empty() || !var.optional) {
		info!("Env {}: {}", var.env_key, val);
	}
	if var.optional && val.is_empty() {
		None
	} else {
		Some(val)
	}
}

impl ConfigLoader {
	pub fn from_env() -> Self {
		info!("Executing: {}", std::env::args().collect::<Vec<_>>().join(" "));

		let vars: HashMap<&str, EnvVar> = HashMap::from([
			(
				"mailer_type",
				EnvVar {
					env_key: "OE_MAILER_TYPE",
					default: DEFAULT_MAILER_TYPE,
					sensitive: false,
					optional: false,
				},
			),
			(
				"mailer_api_host",
				EnvVar {
					env_key: "OE_SENDGRID_API_HOST",
					default: DEFAULT_MAILER_API_HOST,
					sensitive: false,
					optional: true,
				},
			),
			(
				"mailer_api_key",
				EnvVar {
					env_key: "OE_SENDGRID_API_KEY",
					default: DEFAULT_MAILER_API_KEY,
					sensitive: true,
					optional: false,
				},
			),
			(
				"mailer_from_email",
				EnvVar {
					env_key: "OE_SENDGRID_FROM_EMAIL",
					default: DEFAULT_MAILER_FROM_EMAIL,
					sensitive: false,
					optional: false,
				},
			),
			(
				"mailer_from_name",
				EnvVar {
					env_key: "OE_SENDGRID_FROM_NAME",
					default: DEFAULT_MAILER_FROM_NAME,
					sensitive: false,
					optional: false,
				},
			),
			(
				"google_client_id",
				EnvVar {
					env_key: "OE_GOOGLE_CLIENT_ID",
					default: DEFAULT_GOOGLE_CLIENT_ID,
					sensitive: false,
					optional: false,
				},
			),
			(
				"google_client_secret",
				EnvVar {
					env_key: "OE_GOOGLE_CLIENT_SECRET",
					default: DEFAULT_GOOGLE_CLIENT_SECRET,
					sensitive: true,
					optional: false,
				},
			),
			(
				"parentchain_url",
				EnvVar {
					env_key: "OE_PARENTCHAIN_URL",
					default: DEFAULT_PARENTCHAIN_URL,
					sensitive: false,
					optional: false,
				},
			),
			(
				"ethereum_url",
				EnvVar {
					env_key: "OE_ETHEREUM_URL",
					default: DEFAULT_ETHEREUM_URL,
					sensitive: false,
					optional: false,
				},
			),
			(
				"solana_url",
				EnvVar {
					env_key: "OE_SOLANA_URL",
					default: DEFAULT_SOLANA_URL,
					sensitive: false,
					optional: false,
				},
			),
			(
				"bsc_url",
				EnvVar {
					env_key: "OE_BSC_URL",
					default: DEFAULT_BSC_URL,
					sensitive: false,
					optional: false,
				},
			),
			(
				"bsc_testnet_url",
				EnvVar {
					env_key: "OE_BSC_TESTNET_URL",
					default: DEFAULT_BSC_TESTNET_URL,
					sensitive: false,
					optional: true,
				},
			),
			(
				"arbitrum_url",
				EnvVar {
					env_key: "OE_ARBITRUM_URL",
					default: DEFAULT_ARBITRUM_URL,
					sensitive: false,
					optional: false,
				},
			),
			(
				"arbitrum_testnet_url",
				EnvVar {
					env_key: "OE_ARBITRUM_TESTNET_URL",
					default: DEFAULT_ARBITRUM_TESTNET_URL,
					sensitive: false,
					optional: true,
				},
			),
			(
				"hyperevm_url",
				EnvVar {
					env_key: "OE_HYPEREVM_URL",
					default: DEFAULT_HYPEREVM_URL,
					sensitive: false,
					optional: false,
				},
			),
			(
				"hyperevm_testnet_url",
				EnvVar {
					env_key: "OE_HYPEREVM_TESTNET_URL",
					default: DEFAULT_HYPEREVM_TESTNET_URL,
					sensitive: false,
					optional: true,
				},
			),
			(
				"pumpx_signer_url",
				EnvVar {
					env_key: "OE_PUMPX_SIGNER_URL",
					default: DEFAULT_PUMPX_SIGNER_URL,
					sensitive: false,
					optional: false,
				},
			),
			(
				"pumpx_api_base_url",
				EnvVar {
					env_key: "OE_PUMPX_API_BASE_URL",
					default: DEFAULT_PUMPX_API_BASE_URL,
					sensitive: false,
					optional: false,
				},
			),
			(
				"pumpx_worker_url",
				EnvVar {
					env_key: "OE_PUMPX_WORKER_URL",
					default: DEFAULT_PUMPX_WORKER_URL,
					sensitive: false,
					optional: false,
				},
			),
			(
				"binance_api_key",
				EnvVar {
					env_key: "OE_BINANCE_API_KEY",
					default: DEFAULT_BINANCE_API_KEY,
					sensitive: true,
					optional: false,
				},
			),
			(
				"binance_api_secret",
				EnvVar {
					env_key: "OE_BINANCE_API_SECRET",
					default: DEFAULT_BINANCE_API_SECRET,
					sensitive: true,
					optional: false,
				},
			),
			(
				"binance_api_base_url",
				EnvVar {
					env_key: "OE_BINANCE_API_BASE_URL",
					default: DEFAULT_BINANCE_API_BASE_URL,
					sensitive: false,
					optional: false,
				},
			),
			(
				"omni_factory_address",
				EnvVar {
					env_key: "OE_OMNI_FACTORY_ADDRESS",
					default: DEFAULT_OMNI_FACTORY_ADDRESS,
					sensitive: false,
					optional: false,
				},
			),
			(
				"entry_point_address",
				EnvVar {
					env_key: "OE_ENTRY_POINT_ADDRESS",
					default: DEFAULT_ENTRY_POINT_ADDRESS,
					sensitive: false,
					optional: false,
				},
			),
		]);

		let alchemy_key = std::env::var("OE_ALCHEMY_KEY").unwrap_or_default();
		let append_key = |url: &str| {
			if !alchemy_key.is_empty() && url.contains("alchemy") {
				format!("{}{}", url, alchemy_key)
			} else {
				url.to_string()
			}
		};

		let get = |key: &str| get_env_value(&vars[key]).unwrap_or_default();
		let get_opt = |key: &str| get_env_value(&vars[key]);

		ConfigLoader {
			mailer_type: MailerType::from_str(&get("mailer_type")).unwrap_or(MailerType::Sendgrid),
			mailer_api_host: get_opt("mailer_api_host"),
			mailer_api_key: get("mailer_api_key"),
			mailer_from_email: get("mailer_from_email"),
			mailer_from_name: get("mailer_from_name"),
			google_client_id: get("google_client_id"),
			google_client_secret: get("google_client_secret"),
			parentchain_url: get("parentchain_url"),
			ethereum_url: append_key(&get("ethereum_url")),
			solana_url: append_key(&get("solana_url")),
			bsc_url: append_key(&get("bsc_url")),
			bsc_testnet_url: get_opt("bsc_testnet_url").map(|v| append_key(&v)),
			arbitrum_url: append_key(&get("arbitrum_url")),
			arbitrum_testnet_url: get_opt("arbitrum_testnet_url").map(|v| append_key(&v)),
			hyperevm_url: get("hyperevm_url"),
			hyperevm_testnet_url: get_opt("hyperevm_testnet_url"),
			pumpx_signer_url: get("pumpx_signer_url"),
			pumpx_api_base_url: get("pumpx_api_base_url"),
			pumpx_worker_url: get("pumpx_worker_url"),
			binance_api_key: get("binance_api_key"),
			binance_api_secret: get("binance_api_secret"),
			binance_api_base_url: get("binance_api_base_url"),
			omni_factory_address: get("omni_factory_address"),
			entry_point_address: get("entry_point_address"),
		}
	}
}
