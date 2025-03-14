mod types;

use crate::{error::Error, traits::TryIntoParams, BinanceApi, Method};
use std::collections::HashMap;
use types::{CreateOrderParams, ExchangeInfo, Permission, ServerTime, TradeOrder};

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

	/// Get Current exchange trading rules and symbol information
	/// https://developers.binance.com/docs/binance-spot-api-docs/rest-api/general-endpoints#exchange-information
	pub async fn get_exchange_info(
		&self,
		symbol: Option<String>,
		symbols: Option<Vec<String>>,
		permissions: Option<Permission>,
	) -> Result<ExchangeInfo, Error> {
		let endpoint = format!("{}/exchangeInfo", SPOT_TRADING_API);
		let mut params = HashMap::new();
		if let Some(permissions) = permissions {
			if symbols.is_some() || symbol.is_some() {
				return Err(Error::InvalidParams);
			}
			params.insert("permissions".to_string(), format!("{}", permissions));
		}
		if let Some(symbol) = symbol {
			params.insert("symbol".to_string(), symbol);
		}
		if let Some(symbols) = symbols {
			params.insert("symbols".to_string(), symbols.join(","));
		}

		self.base_api.make_public_get_request(&endpoint, Some(params)).await
	}

	/// Create a new order
	/// https://developers.binance.com/docs/binance-spot-api-docs/rest-api/trading-endpoints#new-order-trade
	pub async fn create_order(
		&self,
		create_order_params: CreateOrderParams,
		recv_window: Option<u32>,
	) -> Result<TradeOrder, Error> {
		let endpoint = format!("{}/order", SPOT_TRADING_API);
		let params = create_order_params.try_into_params().map_err(|e| {
			log::error!("Error converting create order params: {}", e);
			Error::InvalidParams
		})?;
		self.base_api
			.make_signed_request(&endpoint, Method::POST, Some(params), recv_window)
			.await
	}
}
