mod types;

use crate::{error::Error, BinanceApi};
use std::collections::HashMap;
use types::{ExchangeInfo, Permission, ServerTime};

/// https://developers.binance.com/docs/binance-spot-api-docs/rest-api/general-api-information
const SPOT_TRADING_API: &str = "/api/v3";

pub struct SpotTradingApi<'a> {
	base_api: &'a BinanceApi,
}

impl<'a> SpotTradingApi<'a> {
	pub fn new(binance_api: &BinanceApi) -> SpotTradingApi {
		SpotTradingApi { base_api: binance_api }
	}

	/// Test connectivity to the Rest API
	pub async fn test_connectivity(&self) -> Result<(), Error> {
		let endpoint = format!("{}/ping", SPOT_TRADING_API);
		self.base_api.make_public_get_request(&endpoint, None).await
	}

	/// Check server time
	pub async fn get_server_time(&self) -> Result<u64, Error> {
		let endpoint = format!("{}/time", SPOT_TRADING_API);
		let server_time: ServerTime =
			self.base_api.make_public_get_request(&endpoint, None).await?;
		Ok(server_time.server_time)
	}

	pub async fn get_exchange_info(
		&self,
		symbol: Option<String>,
		symbols: Option<Vec<String>>,
		permissions: Option<Permission>,
	) -> Result<ExchangeInfo, Error> {
		let endpoint = format!("{}/exchangeInfo", SPOT_TRADING_API);
		let mut params = HashMap::new();
		if let Some(symbol) = symbol {
			params.insert("symbol".to_string(), symbol);
		}
		if let Some(symbols) = symbols {
			params.insert("symbols".to_string(), symbols.join(","));
		}
		if let Some(permissions) = permissions {
			params.insert("permissions".to_string(), format!("{}", permissions));
		}

		self.base_api.make_public_get_request(&endpoint, Some(params)).await
	}
}
