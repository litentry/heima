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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::result_large_err)]

extern crate alloc;
extern crate core;
#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

// re-export module to properly feature gate sgx and regular std environment
#[cfg(all(not(feature = "std"), feature = "sgx"))]
pub mod sgx_reexport_prelude {
	pub use chrono_sgx as chrono;
	pub use http_req_sgx as http_req;
	pub use http_sgx as http;
	pub use thiserror_sgx as thiserror;
}

#[cfg(all(not(feature = "std"), feature = "sgx"))]
use crate::sgx_reexport_prelude::*;

use alloc::{
	format,
	string::{String, ToString},
	vec,
	vec::Vec,
};
use codec::{Decode, Encode};
use core::time::Duration;
use http_req::response::Headers;
use itc_rest_client::{
	error::Error as HttpError,
	http_client::{HttpClient, Send, SendWithCertificateVerification},
	rest_client::RestClient,
	Query, RestGet, RestPath, RestPost,
};
use litentry_primitives::{
	AchainableParams, Assertion, ErrorDetail, ErrorString, IntoErrorDetail, ParameterString,
	VCMPError,
};
use log::debug;
use serde::{Deserialize, Serialize};
use std::{env, thread};
use url::Url;

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

pub mod achainable;
pub mod achainable_names;
pub mod blockchain_info;
pub mod daren_market;
pub mod discord_litentry;
pub mod discord_official;
pub mod geniidata;
pub mod google;
pub mod karat_dao;
pub mod magic_craft;
pub mod moralis;
pub mod nodereal;
pub mod nodereal_jsonrpc;
pub mod twitter_official;
pub mod vip3;

const TIMEOUT: Duration = Duration::from_secs(3u64);

pub const WBTC_TOKEN_ADDRESS: &str = "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599";
pub const WETH_TOKEN_ADDRESS: &str = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";
pub const USDT_TOKEN_ADDRESS: &str = "0xdac17f958d2ee523a2206206994597c13d831ec7";
pub const USDC_TOKEN_ADDRESS: &str = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
pub const LIT_TOKEN_ADDRESS: &str = "0xb59490ab09a0f526cc7305822ac65f2ab12f9723";
pub const CRV_TOKEN_ADDRESS: &str = "0xd533a949740bb3306d119cc777fa900ba034cd52";
pub const MATIC_TOKEN_ADDRESS: &str = "0x7d1afa7b718fb893db30a3abc0cfc608aacfebb0";
pub const DYDX_TOKEN_ADDRESS: &str = "0x92d6c1e31e14520e676a687f0a93788b716beff5";
pub const AMP_TOKEN_ADDRESS: &str = "0xff20817765cb7f73d4bde2e66e067e58d11095c2";
pub const CVX_TOKEN_ADDRESS: &str = "0x4e3fbd56cd56c3e72c1403e103b45db9da5b9d2b";
pub const TUSD_TOKEN_ADDRESS: &str = "0x0000000000085d4780b73119b644ae5ecd22b376";
pub const USDD_TOKEN_ADDRESS: &str = "0x0c10bf8fcb7bf5412187a595ab97a3609160b5c6";
pub const GUSD_TOKEN_ADDRESS: &str = "0x056fd409e1d7a124bd7017459dfea2f387b6d5cd";
pub const LINK_TOKEN_ADDRESS: &str = "0x514910771af9ca656af840dff83e8264ecf986ca";
pub const GRT_TOKEN_ADDRESS: &str = "0xc944e90c64b2c07662a292be6244bdf05cda44a7";
pub const COMP_TOKEN_ADDRESS: &str = "0xc00e94cb662c3520282e6f5717214004a7f26888";
pub const PEOPLE_TOKEN_ADDRESS: &str = "0x7a58c0be72be218b41c608b7fe7c5bb630736c71";
pub const GTC_TOKEN_ADDRESS: &str = "0xde30da39c46104798bb5aa3fe8b9e0e1f348163f";

#[derive(Debug, PartialEq, PartialOrd)]
pub enum ETokenAddress {
	WBTC,
	LIT,
	USDC,
	USDT,
	CRV,
	MATIC,
	DYDX,
	AMP,
	CVX,
	TUSD,
	USDD,
	GUSD,
	LINK,
	GRT,
	COMP,
	PEOPLE,
	GTC,
	Unknown,
}

pub trait TokenFromString {
	fn from_vec(vec: ParameterString) -> ETokenAddress;
}
impl TokenFromString for ETokenAddress {
	fn from_vec(vec: ParameterString) -> ETokenAddress {
		let address = vec_to_string(vec.to_vec()).unwrap_or_default();
		if address == WBTC_TOKEN_ADDRESS {
			ETokenAddress::WBTC
		} else if address == LIT_TOKEN_ADDRESS {
			ETokenAddress::LIT
		} else if address == USDC_TOKEN_ADDRESS {
			ETokenAddress::USDC
		} else if address == USDT_TOKEN_ADDRESS {
			ETokenAddress::USDT
		} else if address == CRV_TOKEN_ADDRESS {
			ETokenAddress::CRV
		} else if address == MATIC_TOKEN_ADDRESS {
			ETokenAddress::MATIC
		} else if address == DYDX_TOKEN_ADDRESS {
			ETokenAddress::DYDX
		} else if address == AMP_TOKEN_ADDRESS {
			ETokenAddress::AMP
		} else if address == CVX_TOKEN_ADDRESS {
			ETokenAddress::CVX
		} else if address == TUSD_TOKEN_ADDRESS {
			ETokenAddress::TUSD
		} else if address == USDD_TOKEN_ADDRESS {
			ETokenAddress::USDD
		} else if address == GUSD_TOKEN_ADDRESS {
			ETokenAddress::GUSD
		} else if address == LINK_TOKEN_ADDRESS {
			ETokenAddress::LINK
		} else if address == GRT_TOKEN_ADDRESS {
			ETokenAddress::GRT
		} else if address == COMP_TOKEN_ADDRESS {
			ETokenAddress::COMP
		} else if address == PEOPLE_TOKEN_ADDRESS {
			ETokenAddress::PEOPLE
		} else if address == GTC_TOKEN_ADDRESS {
			ETokenAddress::GTC
		} else {
			ETokenAddress::Unknown
		}
	}
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Serialize, Deserialize, Debug)]
pub struct DataProviderConfig {
	pub twitter_official_url: String,
	pub twitter_auth_token_v2: String,
	pub twitter_client_id: String,
	pub twitter_client_secret: String,
	pub discord_official_url: String,
	pub discord_client_id: String,
	pub discord_client_secret: String,
	pub google_client_id: String,
	pub google_client_secret: String,
	pub litentry_discord_microservice_url: String,
	pub discord_auth_token: String,
	pub achainable_url: String,
	pub achainable_auth_key: String,
	pub credential_endpoint: String,
	pub oneblock_notion_key: String,
	pub oneblock_notion_url: String,
	pub sora_quiz_master_id: String,
	pub sora_quiz_attendee_id: String,
	pub nodereal_api_key: String,
	pub nodereal_api_retry_delay: u64,
	pub nodereal_api_retry_times: u16,
	pub nodereal_api_url: String,
	pub nodereal_api_chain_network_url: String,
	pub contest_legend_discord_role_id: String,
	pub contest_popularity_discord_role_id: String,
	pub contest_participant_discord_role_id: String,
	pub vip3_url: String,
	pub geniidata_url: String,
	pub geniidata_api_key: String,
	pub litentry_archive_url: String,
	pub karat_dao_api_retry_delay: u64,
	pub karat_dao_api_retry_times: u16,
	pub karat_dao_api_url: String,
	pub magic_craft_api_retry_delay: u64,
	pub magic_craft_api_retry_times: u16,
	pub magic_craft_api_url: String,
	pub magic_craft_api_key: String,
	pub daren_market_api_retry_delay: u64,
	pub daren_market_api_retry_times: u16,
	pub daren_market_api_url: String,
	pub moralis_api_url: String,
	pub moralis_solana_api_url: String,
	pub moralis_api_retry_delay: u64,
	pub moralis_api_retry_times: u16,
	pub moralis_api_key: String,
	pub blockchain_info_api_retry_delay: u64,
	pub blockchain_info_api_retry_times: u16,
	pub blockchain_info_api_url: String,
	pub sendgrid_api_key: String,
	pub sendgrid_from_email: String,
}

impl DataProviderConfig {
	pub fn new() -> Result<Self, Error> {
		log::debug!("Initializing data providers config");

		// default prod config
		let mut config = DataProviderConfig {
			twitter_official_url: "https://api.twitter.com".to_string(),
			twitter_auth_token_v2: "".to_string(),
			twitter_client_id: "".to_string(),
			twitter_client_secret: "".to_string(),
			discord_official_url: "https://discordapp.com".to_string(),
			discord_client_id: "".to_string(),
			discord_client_secret: "".to_string(),
			google_client_id: "".to_string(),
			google_client_secret: "".to_string(),
			litentry_discord_microservice_url: "https://tee-microservice.litentry.io:9528"
				.to_string(),
			discord_auth_token: "".to_string(),
			achainable_url: "https://label-production.litentry.io/".to_string(),
			achainable_auth_key: "".to_string(),
			credential_endpoint: "wss://rpc.litentry-parachain.litentry.io".to_string(),
			oneblock_notion_key: "".to_string(),
			oneblock_notion_url: "https://api.notion.com/".to_string(),
			sora_quiz_master_id: "1164463721989554218".to_string(),
			sora_quiz_attendee_id: "1166941149219532800".to_string(),
			nodereal_api_key: "".to_string(),
			nodereal_api_retry_delay: 5000,
			nodereal_api_retry_times: 2,
			nodereal_api_url: "https://open-platform.nodereal.io/".to_string(),
			nodereal_api_chain_network_url: "https://{chain}-mainnet.nodereal.io/".to_string(),
			contest_legend_discord_role_id: "1172576273063739462".to_string(),
			contest_popularity_discord_role_id: "1172576681119195208".to_string(),
			contest_participant_discord_role_id: "1172576734135210104".to_string(),
			vip3_url: "https://dappapi.vip3.io/".to_string(),
			geniidata_url: "https://api.geniidata.com/".to_string(),
			geniidata_api_key: "".to_string(),
			litentry_archive_url: "https://archive-test.litentry.io".to_string(),
			karat_dao_api_retry_delay: 5000,
			karat_dao_api_retry_times: 2,
			karat_dao_api_url: "https://api.karatdao.com/".to_string(),
			magic_craft_api_retry_delay: 5000,
			magic_craft_api_retry_times: 2,
			magic_craft_api_url: "https://lobby-api-prod.magiccraft.io/".to_string(),
			magic_craft_api_key: "".to_string(),
			daren_market_api_retry_delay: 5000,
			daren_market_api_retry_times: 2,
			daren_market_api_url: "https://daren.market/".to_string(),
			moralis_api_key: "".to_string(),
			moralis_api_retry_delay: 5000,
			moralis_api_retry_times: 2,
			moralis_api_url: "https://deep-index.moralis.io/api/v2.2/".to_string(),
			moralis_solana_api_url: "https://solana-gateway.moralis.io/".to_string(),
			blockchain_info_api_retry_delay: 5000,
			blockchain_info_api_retry_times: 2,
			blockchain_info_api_url: "https://blockchain.info/".to_string(),
			sendgrid_api_key: "".to_string(),
			sendgrid_from_email: "".to_string(),
		};

		// we allow to override following config properties for non prod dev
		#[cfg(any(feature = "env-data-providers-config", feature = "development"))]
		{
			if let Ok(v) = env::var("TWITTER_OFFICIAL_URL") {
				config.set_twitter_official_url(v)?;
			}
			if let Ok(v) = env::var("DISCORD_OFFICIAL_URL") {
				config.set_discord_official_url(v)?;
			}
			if let Ok(v) = env::var("LITENTRY_DISCORD_MICROSERVICE_URL") {
				config.set_litentry_discord_microservice_url(v)?;
			}
			if let Ok(v) = env::var("ACHAINABLE_URL") {
				config.set_achainable_url(v)?;
			}
			if let Ok(v) = env::var("CREDENTIAL_ENDPOINT") {
				config.set_credential_endpoint(v);
			}
			if let Ok(v) = env::var("ONEBLOCK_NOTION_URL") {
				config.set_oneblock_notion_url(v)?;
			}
			if let Ok(v) = env::var("SORA_QUIZ_MASTER_ID") {
				config.set_sora_quiz_master_id(v);
			}
			if let Ok(v) = env::var("SORA_QUIZ_ATTENDEE_ID") {
				config.set_sora_quiz_attendee_id(v);
			}
			if let Ok(v) = env::var("NODEREAL_API_URL") {
				config.set_nodereal_api_url(v)?;
			}
			if let Ok(v) = env::var("NODEREAL_API_RETRY_DELAY") {
				config.set_nodereal_api_retry_delay(v.parse::<u64>().unwrap());
			}
			if let Ok(v) = env::var("NODEREAL_API_RETRY_TIMES") {
				config.set_nodereal_api_retry_times(v.parse::<u16>().unwrap());
			}
			if let Ok(v) = env::var("NODEREAL_API_CHAIN_NETWORK_URL") {
				config.set_nodereal_api_chain_network_url(v)?;
			}
			if let Ok(v) = env::var("CONTEST_LEGEND_DISCORD_ROLE_ID") {
				config.set_contest_legend_discord_role_id(v);
			}
			if let Ok(v) = env::var("CONTEST_POPULARITY_DISCORD_ROLE_ID") {
				config.set_contest_popularity_discord_role_id(v);
			}
			if let Ok(v) = env::var("CONTEST_PARTICIPANT_DISCORD_ROLE_ID") {
				config.set_contest_participant_discord_role_id(v);
			}
			if let Ok(v) = env::var("VIP3_URL") {
				config.set_vip3_url(v)?;
			}
			if let Ok(v) = env::var("GENIIDATA_URL") {
				config.set_geniidata_url(v)?;
			}
			if let Ok(v) = env::var("LITENTRY_ARCHIVE_URL") {
				config.set_litentry_archive_url(v)?;
			}
			if let Ok(v) = env::var("KARAT_DAO_API_RETRY_DELAY") {
				config.set_karat_dao_api_retry_delay(v.parse::<u64>().unwrap());
			}
			if let Ok(v) = env::var("KARAT_DAO_API_RETRY_TIMES") {
				config.set_karat_dao_api_retry_times(v.parse::<u16>().unwrap());
			}
			if let Ok(v) = env::var("KARAT_DAO_API_URL") {
				config.set_karat_dao_api_url(v)?;
			}
			if let Ok(v) = env::var("MAGIC_CRAFT_API_RETRY_DELAY") {
				config.set_magic_craft_api_retry_delay(v.parse::<u64>().unwrap());
			}
			if let Ok(v) = env::var("MAGIC_CRAFT_API_RETRY_TIMES") {
				config.set_magic_craft_api_retry_times(v.parse::<u16>().unwrap());
			}
			if let Ok(v) = env::var("MAGIC_CRAFT_API_URL") {
				config.set_magic_craft_api_url(v)?;
			}
			if let Ok(v) = env::var("DAREN_MARKET_API_RETRY_DELAY") {
				config.set_daren_market_api_retry_delay(v.parse::<u64>().unwrap());
			}
			if let Ok(v) = env::var("DAREN_MARKET_API_RETRY_TIMES") {
				config.set_daren_market_api_retry_times(v.parse::<u16>().unwrap());
			}
			if let Ok(v) = env::var("DAREN_MARKET_API_URL") {
				config.set_daren_market_api_url(v)?;
			}
			if let Ok(v) = env::var("MORALIS_API_URL") {
				config.set_moralis_api_url(v)?;
			}
			if let Ok(v) = env::var("MORALIS_SOLANA_API_URL") {
				config.set_moralis_solana_api_url(v)?;
			}
			if let Ok(v) = env::var("MORALIS_API_RETRY_DELAY") {
				config.set_moralis_api_retry_delay(v.parse::<u64>().unwrap());
			}
			if let Ok(v) = env::var("MORALIS_API_RETRY_TIMES") {
				config.set_moralis_api_retry_times(v.parse::<u16>().unwrap());
			}
			if let Ok(v) = env::var("BLOCKCHAIN_INFO_API_URL") {
				config.set_blockchain_info_api_url(v)?;
			}
			if let Ok(v) = env::var("BLOCKCHAIN_INFO_API_RETRY_DELAY") {
				config.set_blockchain_info_api_retry_delay(v.parse::<u64>().unwrap());
			}
			if let Ok(v) = env::var("BLOCKCHAIN_INFO_API_RETRY_TIMES") {
				config.set_blockchain_info_api_retry_times(v.parse::<u16>().unwrap());
			}
		};
		// set secrets from env variables
		if let Ok(v) = env::var("TWITTER_AUTH_TOKEN_V2") {
			config.set_twitter_auth_token_v2(v);
		}
		if let Ok(v) = env::var("TWITTER_CLIENT_ID") {
			config.set_twitter_client_id(v);
		}
		if let Ok(v) = env::var("TWITTER_CLIENT_SECRET") {
			config.set_twitter_client_secret(v);
		}
		if let Ok(v) = env::var("DISCORD_AUTH_TOKEN") {
			config.set_discord_auth_token(v);
		}
		if let Ok(v) = env::var("DISCORD_CLIENT_ID") {
			config.set_discord_client_id(v);
		}
		if let Ok(v) = env::var("DISCORD_CLIENT_SECRET") {
			config.set_discord_client_secret(v);
		}
		if let Ok(v) = env::var("GOOGLE_CLIENT_ID") {
			config.set_google_client_id(v);
		}
		if let Ok(v) = env::var("GOOGLE_CLIENT_SECRET") {
			config.set_google_client_secret(v);
		}
		if let Ok(v) = env::var("ACHAINABLE_AUTH_KEY") {
			config.set_achainable_auth_key(v);
		}
		if let Ok(v) = env::var("ONEBLOCK_NOTION_KEY") {
			config.set_oneblock_notion_key(v);
		}
		if let Ok(v) = env::var("NODEREAL_API_KEY") {
			config.set_nodereal_api_key(v);
		}
		if let Ok(v) = env::var("GENIIDATA_API_KEY") {
			config.set_geniidata_api_key(v);
		}
		if let Ok(v) = env::var("MORALIS_API_KEY") {
			config.set_moralis_api_key(v);
		}
		if let Ok(v) = env::var("MAGIC_CRAFT_API_KEY") {
			config.set_magic_craft_api_key(v);
		}
		if let Ok(v) = env::var("SENDGRID_API_KEY") {
			config.set_sendgrid_api_key(v);
		}
		if let Ok(v) = env::var("SENDGRID_FROM_EMAIL") {
			config.set_sendgrid_from_email(v);
		}

		Ok(config)
	}
	pub fn set_twitter_official_url(&mut self, v: String) -> Result<(), Error> {
		check_url(&v)?;
		debug!("set_twitter_official_url: {:?}", v);
		self.twitter_official_url = v;
		Ok(())
	}
	pub fn set_twitter_auth_token_v2(&mut self, v: String) {
		debug!("set_twitter_auth_token_v2: {:?}", v);
		self.twitter_auth_token_v2 = v;
	}
	pub fn set_twitter_client_id(&mut self, v: String) {
		debug!("set_twitter_client_id: {:?}", v);
		self.twitter_client_id = v;
	}
	pub fn set_twitter_client_secret(&mut self, v: String) {
		debug!("set_twitter_client_secret: {:?}", v);
		self.twitter_client_secret = v;
	}
	pub fn set_discord_official_url(&mut self, v: String) -> Result<(), Error> {
		check_url(&v)?;
		debug!("set_discord_official_url: {:?}", v);
		self.discord_official_url = v;
		Ok(())
	}
	pub fn set_discord_client_id(&mut self, v: String) {
		debug!("set_discord_client_id: {:?}", v);
		self.discord_client_id = v;
	}
	pub fn set_discord_client_secret(&mut self, v: String) {
		debug!("set_discord_client_secret: {:?}", v);
		self.discord_client_secret = v;
	}
	pub fn set_google_client_id(&mut self, v: String) {
		debug!("set_google_client_id: {:?}", v);
		self.google_client_id = v;
	}
	pub fn set_google_client_secret(&mut self, v: String) {
		debug!("set_google_client_secret: {:?}", v);
		self.google_client_secret = v;
	}
	pub fn set_litentry_discord_microservice_url(&mut self, v: String) -> Result<(), Error> {
		check_url(&v)?;
		debug!("set_litentry_discord_microservice_url: {:?}", v);
		self.litentry_discord_microservice_url = v;
		Ok(())
	}
	pub fn set_discord_auth_token(&mut self, v: String) {
		debug!("set_discord_auth_token: {:?}", v);
		self.discord_auth_token = v;
	}
	pub fn set_achainable_url(&mut self, v: String) -> Result<(), Error> {
		check_url(&v)?;
		debug!("set_achainable_url: {:?}", v);
		self.achainable_url = v;
		Ok(())
	}
	pub fn set_achainable_auth_key(&mut self, v: String) {
		debug!("set_achainable_auth_key: {:?}", v);
		self.achainable_auth_key = v;
	}
	pub fn set_credential_endpoint(&mut self, v: String) {
		debug!("set_credential_endpoint: {:?}", v);
		self.credential_endpoint = v;
	}
	pub fn set_oneblock_notion_key(&mut self, v: String) {
		debug!("set_oneblock_notion_key: {:?}", v);
		self.oneblock_notion_key = v;
	}
	pub fn set_oneblock_notion_url(&mut self, v: String) -> Result<(), Error> {
		check_url(&v)?;
		debug!("set_oneblock_notion_url: {:?}", v);
		self.oneblock_notion_url = v;
		Ok(())
	}
	pub fn set_sora_quiz_master_id(&mut self, v: String) {
		debug!("set_sora_quiz_master_id: {:?}", v);
		self.sora_quiz_master_id = v;
	}
	pub fn set_sora_quiz_attendee_id(&mut self, v: String) {
		debug!("set_sora_quiz_attendee_id: {:?}", v);
		self.sora_quiz_attendee_id = v;
	}
	pub fn set_nodereal_api_key(&mut self, v: String) {
		debug!("set_nodereal_api_key: {:?}", v);
		self.nodereal_api_key = v;
	}
	pub fn set_nodereal_api_retry_delay(&mut self, v: u64) {
		debug!("set_nodereal_api_retry_delay: {:?}", v);
		self.nodereal_api_retry_delay = v;
	}
	pub fn set_nodereal_api_retry_times(&mut self, v: u16) {
		debug!("set_nodereal_api_retry_times: {:?}", v);
		self.nodereal_api_retry_times = v;
	}
	pub fn set_nodereal_api_url(&mut self, v: String) -> Result<(), Error> {
		check_url(&v)?;
		debug!("set_nodereal_api_url: {:?}", v);
		self.nodereal_api_url = v;
		Ok(())
	}
	pub fn set_nodereal_api_chain_network_url(&mut self, v: String) -> Result<(), Error> {
		check_url(&v)?;
		debug!("set_nodereal_api_chain_network_url: {:?}", v);
		self.nodereal_api_chain_network_url = v;
		Ok(())
	}
	pub fn set_contest_legend_discord_role_id(&mut self, v: String) {
		debug!("set_contest_legend_discord_role_id: {:?}", v);
		self.contest_legend_discord_role_id = v;
	}
	pub fn set_contest_popularity_discord_role_id(&mut self, v: String) {
		debug!("set_contest_popularity_discord_role_id: {:?}", v);
		self.contest_popularity_discord_role_id = v;
	}
	pub fn set_contest_participant_discord_role_id(&mut self, v: String) {
		debug!("set_contest_participant_discord_role_id: {:?}", v);
		self.contest_participant_discord_role_id = v;
	}
	pub fn set_vip3_url(&mut self, v: String) -> Result<(), Error> {
		check_url(&v)?;
		debug!("set_vip3_url: {:?}", v);
		self.vip3_url = v;
		Ok(())
	}
	pub fn set_geniidata_url(&mut self, v: String) -> Result<(), Error> {
		check_url(&v)?;
		debug!("set_geniidata_url: {:?}", v);
		self.geniidata_url = v;
		Ok(())
	}
	pub fn set_geniidata_api_key(&mut self, v: String) {
		debug!("set_geniidata_api_key: {:?}", v);
		self.geniidata_api_key = v;
	}
	pub fn set_litentry_archive_url(&mut self, v: String) -> Result<(), Error> {
		check_url(&v)?;
		debug!("set_litentry_archive_url: {:?}", v);
		self.litentry_archive_url = v;
		Ok(())
	}
	pub fn set_karat_dao_api_retry_delay(&mut self, v: u64) {
		debug!("set_karat_dao_api_retry_delay: {:?}", v);
		self.karat_dao_api_retry_delay = v;
	}
	pub fn set_karat_dao_api_retry_times(&mut self, v: u16) {
		debug!("set_karat_dao_api_retry_times: {:?}", v);
		self.karat_dao_api_retry_times = v;
	}
	pub fn set_karat_dao_api_url(&mut self, v: String) -> Result<(), Error> {
		check_url(&v)?;
		debug!("set_karat_dao_api_url: {:?}", v);
		self.karat_dao_api_url = v;
		Ok(())
	}
	pub fn set_magic_craft_api_retry_delay(&mut self, v: u64) {
		debug!("set_magic_craft_api_retry_delay: {:?}", v);
		self.magic_craft_api_retry_delay = v;
	}
	pub fn set_magic_craft_api_retry_times(&mut self, v: u16) {
		debug!("set_magic_craft_api_retry_times: {:?}", v);
		self.magic_craft_api_retry_times = v;
	}
	pub fn set_magic_craft_api_url(&mut self, v: String) -> Result<(), Error> {
		check_url(&v)?;
		debug!("set_magic_craft_api_url: {:?}", v);
		self.magic_craft_api_url = v;
		Ok(())
	}
	pub fn set_magic_craft_api_key(&mut self, v: String) {
		debug!("set_magic_craft_api_key: {:?}", v);
		self.magic_craft_api_key = v;
	}
	pub fn set_daren_market_api_retry_delay(&mut self, v: u64) {
		debug!("set_daren_market_api_retry_delay: {:?}", v);
		self.daren_market_api_retry_delay = v;
	}
	pub fn set_daren_market_api_retry_times(&mut self, v: u16) {
		debug!("set_daren_market_api_retry_times: {:?}", v);
		self.daren_market_api_retry_times = v;
	}
	pub fn set_daren_market_api_url(&mut self, v: String) -> Result<(), Error> {
		check_url(&v)?;
		debug!("set_daren_market_api_url: {:?}", v);
		self.daren_market_api_url = v;
		Ok(())
	}
	pub fn set_moralis_api_key(&mut self, v: String) {
		debug!("set_moralis_api_key: {:?}", v);
		self.moralis_api_key = v;
	}
	pub fn set_moralis_api_retry_delay(&mut self, v: u64) {
		debug!("set_moralis_api_retry_delay: {:?}", v);
		self.moralis_api_retry_delay = v;
	}
	pub fn set_moralis_api_retry_times(&mut self, v: u16) {
		debug!("set_moralis_api_retry_times: {:?}", v);
		self.moralis_api_retry_times = v;
	}
	pub fn set_moralis_api_url(&mut self, v: String) -> Result<(), Error> {
		check_url(&v)?;
		debug!("set_moralis_api_url: {:?}", v);
		self.moralis_api_url = v;
		Ok(())
	}
	pub fn set_moralis_solana_api_url(&mut self, v: String) -> Result<(), Error> {
		check_url(&v)?;
		debug!("set_moralis_solana_api_url: {:?}", v);
		self.moralis_solana_api_url = v;
		Ok(())
	}
	pub fn set_blockchain_info_api_retry_delay(&mut self, v: u64) {
		debug!("set_blockchain_info_api_retry_delay: {:?}", v);
		self.blockchain_info_api_retry_delay = v;
	}
	pub fn set_blockchain_info_api_retry_times(&mut self, v: u16) {
		debug!("set_blockchain_info_api_retry_times: {:?}", v);
		self.blockchain_info_api_retry_times = v;
	}
	pub fn set_blockchain_info_api_url(&mut self, v: String) -> Result<(), Error> {
		check_url(&v)?;
		debug!("set_blockchain_info_api_url: {:?}", v);
		self.blockchain_info_api_url = v;
		Ok(())
	}
	pub fn set_sendgrid_api_key(&mut self, v: String) {
		debug!("set_sendgrid_api_key: {:?}", v);
		self.sendgrid_api_key = v;
	}
	pub fn set_sendgrid_from_email(&mut self, v: String) {
		debug!("set_sendgrid_from_email: {:?}", v);
		self.sendgrid_from_email = v;
	}
}

fn check_url(v: &String) -> Result<(), Error> {
	match Url::parse(v) {
		Ok(_) => Ok(()),
		Err(err) => Err(Error::Utf8Error(format!("Input url: {:?}, parse error: {:?}", v, err))),
	}
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum Error {
	#[error("Request error: {0}")]
	RequestError(String),

	#[error("UTF8 error: {0}")]
	Utf8Error(String),

	#[error("Achainable error: {0}")]
	AchainableError(String),

	#[error("Nodereal error: {0}")]
	NoderealError(String),

	#[error("GeniiData error: {0}")]
	GeniiDataError(String),

	#[error("Retryable error: {0}")]
	RetryableError(String),
}

impl IntoErrorDetail for Error {
	fn into_error_detail(self) -> ErrorDetail {
		ErrorDetail::DataProviderError(ErrorString::truncate_from(
			format!("{self:?}").as_bytes().to_vec(),
		))
	}
}

pub trait UserInfo {
	fn get_user_id(&self) -> Option<String>;
}

pub fn vec_to_string(vec: Vec<u8>) -> Result<String, Error> {
	let tmp = String::from_utf8(vec.to_vec()).map_err(|e| Error::Utf8Error(e.to_string()))?;
	let tmp = tmp.trim();
	if tmp.is_empty() {
		return Err(Error::Utf8Error("empty string".to_string()))
	}
	Ok(tmp.to_string())
}

pub fn build_client_with_cert(
	base_url: &str,
	headers: Headers,
) -> RestClient<HttpClient<SendWithCertificateVerification>> {
	debug!("base_url: {}", base_url);
	let base_url = Url::parse(base_url).unwrap();
	let http_client = HttpClient::new(
		SendWithCertificateVerification::new(vec![]),
		true,
		Some(TIMEOUT),
		Some(headers),
		None,
	);
	RestClient::new(http_client, base_url)
}

#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct ReqPath<'a> {
	path: &'a str,
}

impl<'a> ReqPath<'a> {
	pub fn new(path: &'a str) -> Self {
		Self { path }
	}
}

#[derive(Debug, Clone)]
pub struct RetryOption {
	// default delay is 0
	retry_delay: Option<u64>,
	// set retry_times to 0 or None for fail fast
	retry_times: Option<u16>,
}

const DEFAULT_RETRY_OPTION: RetryOption = RetryOption { retry_delay: None, retry_times: None };

pub type RestHttpClient<T> = RestClient<HttpClient<T>>;

pub trait RetryableRestClient<T> {
	fn retry<A, R>(&mut self, action: A, retry_option: RetryOption) -> Result<R, Error>
	where
		A: Fn(&mut RestHttpClient<T>) -> Result<R, HttpError>;
}

impl<T> RetryableRestClient<T> for RestHttpClient<T>
where
	T: Send,
{
	fn retry<A, R>(&mut self, action: A, retry_option: RetryOption) -> Result<R, Error>
	where
		A: Fn(&mut RestHttpClient<T>) -> Result<R, HttpError>,
	{
		let mut retries = 0;
		let base_delay = Duration::from_millis(retry_option.retry_delay.unwrap_or_default());
		let maximum_retries = retry_option.retry_times.unwrap_or_default();

		loop {
			if retries > 0 {
				debug!("Fail to call rest api, begin retry: {}", retries);
			}

			if retries > maximum_retries {
				return Err(Error::RetryableError(format!(
					"Fail to call rest api within {} retries",
					maximum_retries
				)))
			}

			match action(self) {
				Ok(response) => return Ok(response),
				Err(err) => {
					let req_err: Error =
						Error::RequestError(format!("call rest api error: {}", err));
					match err {
						HttpError::HttpError(code, _) =>
							if code == 429 {
								// Too Many Requests
								// exponential back off
								thread::sleep(base_delay * 2u32.pow(retries as u32));
								retries += 1;
							} else {
								return Err(req_err)
							},
						_ => return Err(req_err),
					}
				},
			}
		}
	}
}

pub trait RetryableRestGet {
	// set retry_option to None for fail fast
	fn get_retry<U, R>(&mut self, params: U, retry_option: Option<RetryOption>) -> Result<R, Error>
	where
		U: Copy,
		R: serde::de::DeserializeOwned + RestPath<U>;

	// set retry_option to None for fail fast
	fn get_with_retry<U, R>(
		&mut self,
		params: U,
		query: &Query<'_>,
		retry_option: Option<RetryOption>,
	) -> Result<R, Error>
	where
		U: Copy,
		R: serde::de::DeserializeOwned + RestPath<U>;
}

impl<T> RetryableRestGet for RestHttpClient<T>
where
	T: Send,
{
	fn get_retry<U, R>(&mut self, params: U, retry_option: Option<RetryOption>) -> Result<R, Error>
	where
		U: Copy,
		R: serde::de::DeserializeOwned + RestPath<U>,
	{
		self.retry(|c| c.get(params), retry_option.unwrap_or(DEFAULT_RETRY_OPTION))
	}

	fn get_with_retry<U, R>(
		&mut self,
		params: U,
		query: &Query<'_>,
		retry_option: Option<RetryOption>,
	) -> Result<R, Error>
	where
		U: Copy,
		R: serde::de::DeserializeOwned + RestPath<U>,
	{
		self.retry(|c| c.get_with(params, query), retry_option.unwrap_or(DEFAULT_RETRY_OPTION))
	}
}

pub trait RetryableRestPost {
	// set retry_option to None for fail fast
	fn post_capture_retry<U, D, K>(
		&mut self,
		params: U,
		data: &D,
		retry_option: Option<RetryOption>,
	) -> Result<K, Error>
	where
		U: Copy,
		D: serde::Serialize + RestPath<U>,
		K: serde::de::DeserializeOwned;
}

impl<T> RetryableRestPost for RestHttpClient<T>
where
	T: Send,
{
	fn post_capture_retry<U, D, K>(
		&mut self,
		params: U,
		data: &D,
		retry_option: Option<RetryOption>,
	) -> Result<K, Error>
	where
		U: Copy,
		D: serde::Serialize + RestPath<U>,
		K: serde::de::DeserializeOwned,
	{
		self.retry(|c| c.post_capture(params, data), retry_option.unwrap_or(DEFAULT_RETRY_OPTION))
	}
}

pub trait ConvertParameterString {
	fn to_string(&self, field: &ParameterString) -> Result<String, VCMPError>;
}

impl ConvertParameterString for AchainableParams {
	fn to_string(&self, field: &ParameterString) -> Result<String, VCMPError> {
		vec_to_string(field.to_vec()).map_err(|_| {
			VCMPError::RequestVCFailed(Assertion::Achainable(self.clone()), ErrorDetail::ParseError)
		})
	}
}

pub fn convert_balance_hex_to_u128(hex: &str) -> Result<u128, String> {
	match u128::from_str_radix(&hex[2..], 16) {
		Ok(balance) => Ok(balance),
		Err(_) => Err(format!("Cannot parse hex {:?} to u128", hex)),
	}
}

pub fn convert_balance_hex_json_value_to_u128(value: serde_json::Value) -> Result<u128, Error> {
	match value.as_str() {
		Some(str) => match convert_balance_hex_to_u128(str) {
			Ok(balance) => Ok(balance),
			Err(err) => Err(Error::RequestError(err)),
		},
		None => Err(Error::RequestError(format!("Cannot transform value {:?} to &str", value))),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_return_correct_value_when_param_is_valid() {
		assert_eq!(
			convert_balance_hex_json_value_to_u128(serde_json::Value::String("0x0".into()))
				.unwrap(),
			0_u128
		);

		assert_eq!(
			convert_balance_hex_json_value_to_u128(serde_json::Value::String("0x320".into()))
				.unwrap(),
			800_u128
		);

		assert_eq!(
			convert_balance_hex_json_value_to_u128(serde_json::Value::String(
				"0x2b5e3af16b1880000".into()
			))
			.unwrap(),
			50_000_000_000_000_000_000_u128
		);
	}

	#[test]
	fn should_return_error_when_param_is_not_a_str() {
		match convert_balance_hex_json_value_to_u128(serde_json::Value::Bool(true)) {
			Ok(_) => panic!("Expected an error, but got Ok"),
			Err(err) => assert_eq!(
				err.to_string(),
				"Request error: Cannot transform value Bool(true) to &str"
			),
		}
	}

	#[test]
	fn should_return_error_when_param_is_not_a_hex_str() {
		match convert_balance_hex_json_value_to_u128(serde_json::Value::String("qwexyz".into())) {
			Ok(_) => panic!("Expected an error, but got Ok"),
			Err(err) =>
				assert_eq!(err.to_string(), "Request error: Cannot parse hex \"qwexyz\" to u128"),
		}
	}
}
