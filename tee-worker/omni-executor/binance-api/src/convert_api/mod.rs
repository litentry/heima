mod types;

use crate::{error::Error, types::AssetSymbol, BinanceApi, Method};
use log::error;
use std::collections::HashMap;
use types::{
	AssetInfo, CancelOrderLimit, ConvertOrder, ConvertOrderStatus, ConvertTradeHistory, LimitOrder,
	OpenOrders, PlaceLimitOrderParams, Quote, RequestQuoteParams, TokenPair,
};

/// https://developers.binance.com/docs/convert/general-info
const CONVERT_API: &str = "/sapi/v1/convert";

const MAX_LIMIT: u16 = 1000;

pub struct ConvertApi<'a> {
	base_api: &'a BinanceApi,
}

impl<'a> ConvertApi<'a> {
	pub fn new(binance_api: &BinanceApi) -> ConvertApi {
		ConvertApi { base_api: binance_api }
	}

	/// List all convertible token pairs and the tokens’ respective upper/lower limits
	pub async fn get_exchange_info(
		&self,
		from_asset: Option<AssetSymbol>,
		to_asset: Option<AssetSymbol>,
	) -> Result<Vec<TokenPair>, Error> {
		let endpoint = format!("{}/exchangeInfo", CONVERT_API);
		let mut params = HashMap::new();
		if let Some(from_asset) = from_asset {
			params.insert("fromAsset".to_string(), from_asset.to_string());
		}
		if let Some(to_asset) = to_asset {
			params.insert("toAsset".to_string(), to_asset.to_string());
		}
		let token_pairs = self.base_api.make_public_get_request(&endpoint, Some(params)).await?;

		Ok(token_pairs)
	}

	/// List all supported asset’s precision information
	pub async fn get_asset_info(&self, recv_window: Option<u32>) -> Result<Vec<AssetInfo>, Error> {
		let endpoint = format!("{}/assetInfo", CONVERT_API);
		self.base_api
			.make_signed_request(&endpoint, Method::GET, None, recv_window)
			.await
	}

	/// Request a quote for the requested token pairs
	pub async fn get_quote(
		&self,
		request_quote_params: RequestQuoteParams,
		recv_window: Option<u32>,
	) -> Result<Quote, Error> {
		let endpoint = format!("{}/getQuote", CONVERT_API);
		let params = request_quote_params.try_into_params().map_err(|e| {
			error!("Error converting request quote params: {}", e);
			Error::InvalidParams
		})?;
		self.base_api
			.make_signed_request(&endpoint, Method::POST, Some(params), recv_window)
			.await
	}

	/// Accept the offered quote by quote ID.
	pub async fn accept_quote(
		&self,
		quote_id: &str,
		recv_window: Option<u32>,
	) -> Result<ConvertOrder, Error> {
		let endpoint = format!("{}/acceptQuote", CONVERT_API);
		let mut params = HashMap::new();
		params.insert("quoteId".to_string(), quote_id.to_string());
		self.base_api
			.make_signed_request(&endpoint, Method::POST, Some(params), recv_window)
			.await
	}

	/// Get Convert Trade History
	pub async fn get_convert_trade_history(
		&self,
		start_time: u64,
		end_time: u64,
		limit: Option<u16>,
		recv_window: Option<u32>,
	) -> Result<ConvertTradeHistory, Error> {
		if let Some(limit) = limit {
			if limit > MAX_LIMIT {
				return Err(Error::LimitExceeded);
			}
		}
		// The max interval between startTime and endTime is 30 days.
		if end_time - start_time > 30 * 24 * 60 * 60 * 1000 {
			return Err(Error::InvalidParams);
		}

		let mut params = HashMap::new();
		params.insert("startTime".to_string(), start_time.to_string());
		params.insert("endTime".to_string(), end_time.to_string());
		if let Some(limit) = limit {
			params.insert("limit".to_string(), limit.to_string());
		}

		let endpoint = format!("{}/tradeFlow", CONVERT_API);
		self.base_api
			.make_signed_request(&endpoint, Method::GET, Some(params), recv_window)
			.await
	}

	/// Query order status by order ID.
	pub async fn get_order_status(
		&self,
		order_id: Option<String>,
		quote_id: Option<String>,
		recv_window: Option<u32>,
	) -> Result<ConvertOrderStatus, Error> {
		if order_id.is_none() && quote_id.is_none() {
			return Err(Error::InvalidParams);
		}
		let mut params = HashMap::new();

		if let Some(order_id) = order_id {
			params.insert("orderId".to_string(), order_id);
		} else if let Some(quote_id) = quote_id {
			params.insert("quoteId".to_string(), quote_id);
		}

		let endpoint = format!("{}/orderStatus", CONVERT_API);
		self.base_api
			.make_signed_request(&endpoint, Method::GET, Some(params), recv_window)
			.await
	}

	/// Enable users to place a limit order
	pub async fn place_limit_order(
		&self,
		place_limit_order_params: PlaceLimitOrderParams,
		recv_window: Option<u32>,
	) -> Result<LimitOrder, Error> {
		let endpoint = format!("{}/limit/placeOrder", CONVERT_API);
		let params = place_limit_order_params.try_into_params().map_err(|e| {
			error!("Error converting place limit order params: {}", e);
			Error::InvalidParams
		})?;
		self.base_api
			.make_signed_request(&endpoint, Method::POST, Some(params), recv_window)
			.await
	}

	/// Enable users to cancel a limit order
	pub async fn cancel_limit_order(
		&self,
		order_id: String,
		recv_window: Option<u32>,
	) -> Result<CancelOrderLimit, Error> {
		let endpoint = format!("{}/limit/cancelOrder", CONVERT_API);
		let mut params = HashMap::new();
		params.insert("orderId".to_string(), order_id.to_string());
		self.base_api
			.make_signed_request(&endpoint, Method::POST, Some(params), recv_window)
			.await
	}

	/// Query open orders
	pub async fn get_open_orders(&self, recv_window: Option<u32>) -> Result<OpenOrders, Error> {
		let endpoint = format!("{}/limit/queryOpenOrders", CONVERT_API);
		self.base_api
			.make_signed_request(&endpoint, Method::POST, None, recv_window)
			.await
	}
}
