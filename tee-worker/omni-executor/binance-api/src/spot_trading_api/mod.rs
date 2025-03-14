mod types;

use crate::{error::Error, traits::TryIntoParams, BinanceApi, Method};
use std::collections::HashMap;
use types::{CreateOrderParams, ExchangeInfo, Permission, ServerTime, TestTradeOrder, TradeOrder};

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
	) -> Result<TradeOrder, Error> {
		let endpoint = format!("{}/order", SPOT_TRADING_API);
		let recv_window = create_order_params.recv_window;
		let params = create_order_params.try_into_params().map_err(|e| {
			log::error!("Error converting create order params: {}", e);
			Error::InvalidParams
		})?;
		self.base_api
			.make_signed_request(&endpoint, Method::POST, Some(params), recv_window)
			.await
	}

	/// Test new order creation and signature/recvWindow long. Creates and validates a new order but does not send it into the matching engine.
	pub async fn test_new_order(
		&self,
		create_order_params: CreateOrderParams,
		compute_commission_rates: bool,
	) -> Result<TestTradeOrder, Error> {
		let endpoint = format!("{}/order/test", SPOT_TRADING_API);
		let recv_window = create_order_params.recv_window;
		let mut params = create_order_params.try_into_params().map_err(|e| {
			log::error!("Error converting create order params: {}", e);
			Error::InvalidParams
		})?;
		params
			.insert("computeCommissionRates".to_string(), format!("{}", compute_commission_rates));
		self.base_api
			.make_signed_request(&endpoint, Method::POST, Some(params), recv_window)
			.await
	}

	/// Get order
	/// https://developers.binance.com/docs/binance-spot-api-docs/rest-api/trading-endpoints#query-order-user_data
	pub async fn get_order(
		&self,
		symbol: &str,
		order_id: Option<u64>,
		orig_client_order_id: Option<&str>,
		recv_window: Option<u32>,
	) -> Result<TradeOrder, Error> {
		let endpoint = format!("{}/order", SPOT_TRADING_API);
		if order_id.is_none() && orig_client_order_id.is_none() {
			return Err(Error::InvalidParams);
		}
		let mut params = HashMap::new();
		params.insert("symbol".to_string(), symbol.to_string());
		if let Some(order_id) = order_id {
			params.insert("orderId".to_string(), order_id.to_string());
		}
		if let Some(orig_client_order_id) = orig_client_order_id {
			params.insert("origClientOrderId".to_string(), orig_client_order_id.to_string());
		}
		self.base_api
			.make_signed_request(&endpoint, Method::GET, Some(params), recv_window)
			.await
	}
}
