pub mod types;

use crate::{error::Error, traits::TryIntoParams, BinanceApi, Method};
use std::collections::HashMap;
use types::{
	AccountInfo, CancelOrderRestrictions, CreateOrderParams, EmptyResponse, ExchangeInfo,
	Permission, ServerTime, SymbolPrice, TestTradeOrder, TradeOrder,
};

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
	pub async fn test_connectivity(&self) -> Result<EmptyResponse, Error> {
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

	/// Cancel an active order.
	/// https://developers.binance.com/docs/binance-spot-api-docs/rest-api/trading-endpoints#cancel-order-trade
	pub async fn cancel_order(
		&self,
		symbol: &str,
		order_id: Option<u64>,
		orig_client_order_id: Option<&str>,
		new_client_order_id: Option<&str>,
		cancel_restrictions: Option<CancelOrderRestrictions>,
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
		if let Some(new_client_order_id) = new_client_order_id {
			params.insert("newClientOrderId".to_string(), new_client_order_id.to_string());
		}
		if let Some(cancel_restrictions) = cancel_restrictions {
			params.insert("cancelRestrictions".to_string(), format!("{}", cancel_restrictions));
		}
		self.base_api
			.make_signed_request(&endpoint, Method::DELETE, Some(params), recv_window)
			.await
	}

	/// Get all account orders; active, canceled, or filled.
	/// https://developers.binance.com/docs/binance-spot-api-docs/rest-api/trading-endpoints#all-orders-user_data
	pub async fn get_all_orders(
		&self,
		symbol: &str,
		order_id: Option<u64>,
		start_time: Option<u64>,
		end_time: Option<u64>,
		limit: Option<u16>,
		recv_window: Option<u32>,
	) -> Result<Vec<TradeOrder>, Error> {
		let endpoint = format!("{}/allOrders", SPOT_TRADING_API);
		let mut params = HashMap::new();
		params.insert("symbol".to_string(), symbol.to_string());
		if let Some(order_id) = order_id {
			params.insert("orderId".to_string(), order_id.to_string());
		}
		if let Some(start_time) = start_time {
			params.insert("startTime".to_string(), start_time.to_string());
		}
		if let Some(end_time) = end_time {
			params.insert("endTime".to_string(), end_time.to_string());
		}
		if let Some(limit) = limit {
			params.insert("limit".to_string(), limit.to_string());
		}
		self.base_api
			.make_signed_request(&endpoint, Method::GET, Some(params), recv_window)
			.await
	}

	/// Get current account information.
	/// https://developers.binance.com/docs/binance-spot-api-docs/rest-api/account-endpoints#account-information-user_data
	pub async fn get_account_info(
		&self,
		omit_zero_balances: Option<bool>,
		recv_window: Option<u32>,
	) -> Result<AccountInfo, Error> {
		let endpoint = format!("{}/account", SPOT_TRADING_API);
		let mut params = HashMap::new();
		if let Some(omit_zero_balances) = omit_zero_balances {
			params.insert("omitZeroBalances".to_string(), format!("{}", omit_zero_balances));
		}
		self.base_api
			.make_signed_request(&endpoint, Method::GET, Some(params), recv_window)
			.await
	}

	/// Get latest price for a symbol
	/// https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints#symbol-price-ticker
	pub async fn get_symbol_price(&self, symbol: &str) -> Result<String, Error> {
		let endpoint = format!("{}/ticker/price", SPOT_TRADING_API);
		let mut params = HashMap::new();
		params.insert("symbol".to_string(), symbol.to_string());
		let symbol_price: SymbolPrice =
			self.base_api.make_public_get_request(&endpoint, Some(params)).await?;
		Ok(symbol_price.price)
	}
}
