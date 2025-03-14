use std::collections::HashMap;

use crate::{traits::TryIntoParams, types::AssetSymbol};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerTime {
	pub server_time: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInfo {
	pub timezone: String,
	pub server_time: u64,
	pub rate_limits: Vec<RateLimit>,
	pub exchange_filters: Vec<ExchangeFilter>,
	pub symbols: Vec<Symbol>,
	pub sors: Option<Vec<Sor>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
	pub rate_limit_type: RateLimitType,
	pub interval: RateLimitInterval,
	pub interval_num: u32,
	pub limit: u32,
}

#[derive(Debug, Deserialize)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum RateLimitType {
	REQUEST_WEIGHT,
	ORDERS,
	RAW_REQUESTS,
}

#[derive(Debug, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum RateLimitInterval {
	SECOND,
	MINUTE,
	DAY,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "filterType")]
#[allow(non_snake_case, clippy::enum_variant_names)]
pub enum ExchangeFilter {
	#[serde(rename = "EXCHANGE_MAX_NUM_ORDERS")]
	MaxNumOrders { maxNumOrders: u32 },

	#[serde(rename = "EXCHANGE_MAX_NUM_ALGO_ORDERS")]
	MaxNumAlgoOrders { maxNumAlgoOrders: u32 },

	#[serde(rename = "EXCHANGE_MAX_NUM_ICEBERG_ORDERS")]
	MaxNumIcebergOrders { maxNumIcebergOrders: u32 },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Symbol {
	pub symbol: String,
	pub status: SymbolStatus,
	pub base_asset: AssetSymbol,
	pub base_asset_precision: u8,
	pub quote_asset: AssetSymbol,
	pub quote_asset_precision: u8, // will be removed in future api versions (v4+)
	pub base_commission_precision: u8,
	pub quote_commission_precision: u8,
	pub order_types: Vec<OrderType>,
	pub iceberg_allowed: bool,
	pub oco_allowed: bool,
	pub oto_allowed: bool,
	pub quote_order_qty_market_allowed: bool,
	pub allow_trailing_stop: bool,
	pub cancel_replace_allowed: bool,
	pub is_spot_trading_allowed: bool,
	pub is_margin_trading_allowed: bool,
	pub filters: Vec<SymbolFilter>,
	pub permissions: Vec<Permission>,
	pub permission_sets: Vec<Vec<Permission>>,
	pub default_self_trade_prevention_mode: SelfTradePreventionMode,
	pub allowed_self_trade_prevention_modes: Vec<SelfTradePreventionMode>,
}

#[derive(Debug, Deserialize)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum SymbolStatus {
	PRE_TRADING,
	TRADING,
	POST_TRADING,
	END_OF_DAY,
	HALT,
	AUCTION_MATCH,
	BREAK,
}

#[derive(Debug, Deserialize)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum OrderType {
	LIMIT,
	LIMIT_MAKER,
	MARKET,
	STOP_LOSS,
	STOP_LOSS_LIMIT,
	TAKE_PROFIT,
	TAKE_PROFIT_LIMIT,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "filterType")]
#[allow(non_snake_case)]
pub enum SymbolFilter {
	#[serde(rename = "PRICE_FILTER")]
	PriceFilter { minPrice: String, maxPrice: String, tickSize: String },

	#[serde(rename = "PERCENT_PRICE")]
	PercentPrice { multiplierUp: String, multiplierDown: String, avgPriceMins: u32 },

	#[serde(rename = "PERCENT_PRICE_BY_SIDE")]
	PercentPriceBySide {
		bidMultiplierUp: String,
		bidMultiplierDown: String,
		askMultiplierUp: String,
		askMultiplierDown: String,
		avgPriceMins: u32,
	},

	#[serde(rename = "LOT_SIZE")]
	LotSize { minQty: String, maxQty: String, stepSize: String },

	#[serde(rename = "MIN_NOTIONAL")]
	MinNotional { minNotional: String, applyToMarket: bool, avgPriceMins: u32 },

	#[serde(rename = "NOTIONAL")]
	Notional {
		minNotional: String,
		applyToMarket: bool,
		maxNotional: String,
		applyMaxToMarket: bool,
		avgPriceMins: u32,
	},

	#[serde(rename = "ICEBERG_PARTS")]
	IcebergParts { limit: u32 },

	#[serde(rename = "MARKET_LOT_SIZE")]
	MarketLotSize { minQty: String, maxQty: String, stepSize: String },

	#[serde(rename = "MAX_NUM_ORDERS")]
	MaxNumOrders { maxNumOrders: u32 },

	#[serde(rename = "MAX_NUM_ALGO_ORDERS")]
	MaxNumAlgoOrders { maxNumAlgoOrders: u32 },

	#[serde(rename = "MAX_NUM_ICEBERG_ORDERS")]
	MaxNumIcebergOrders { maxNumIcebergOrders: u32 },

	#[serde(rename = "MAX_POSITION")]
	MaxPosition { maxPosition: String },

	#[serde(rename = "TRAILING_DELTA")]
	TrailingDelta {
		minTrailingAboveDelta: String,
		maxTrailingAboveDelta: String,
		minTrailingBelowDelta: String,
		maxTrailingBelowDelta: String,
	},
}

#[derive(Debug, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum Permission {
	SPOT,
	MARGIN,
	LEVERAGED,
}

impl std::fmt::Display for Permission {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Permission::SPOT => write!(f, "SPOT"),
			Permission::MARGIN => write!(f, "MARGIN"),
			Permission::LEVERAGED => write!(f, "LEVERAGED"),
		}
	}
}

#[derive(Debug, Deserialize)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum SelfTradePreventionMode {
	NONE,
	EXPIRE_MAKER,
	EXPIRE_TAKER,
	EXPIRE_BOTH,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sor {
	pub base_asset: AssetSymbol,
	pub symbols: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderParams {
	pub symbol: String,
	pub side: OrderSide,
	#[serde(rename = "type")]
	pub order_type: OrderType,
	pub time_in_force: Option<TimeInForce>,
	pub quantity: Option<String>,
	pub quote_order_qty: Option<String>,
	pub price: Option<String>,
	pub new_client_order_id: Option<String>,
	pub strategy_id: Option<u64>,
	pub strategy_type: Option<u32>,
	pub stop_price: Option<String>,
	pub trailing_delta: Option<u64>,
	pub iceberg_qty: Option<String>,
	pub new_order_resp_type: Option<NewOrderRespType>,
	pub self_trade_prevention_mode: Option<SelfTradePreventionMode>,
	pub recv_window: Option<u64>,
	pub timestamp: u64,
}

impl CreateOrderParams {
	pub fn new(
		symbol: String,
		side: OrderSide,
		order_type: OrderType,
		time_in_force: Option<TimeInForce>,
		quantity: Option<String>,
		quote_order_qty: Option<String>,
		price: Option<String>,
		new_client_order_id: Option<String>,
		strategy_id: Option<u64>,
		strategy_type: Option<u32>,
		stop_price: Option<String>,
		trailing_delta: Option<u64>,
		iceberg_qty: Option<String>,
		new_order_resp_type: Option<NewOrderRespType>,
		self_trade_prevention_mode: Option<SelfTradePreventionMode>,
		recv_window: Option<u64>,
		timestamp: u64,
	) -> CreateOrderParams {
		CreateOrderParams {
			symbol,
			side,
			order_type,
			time_in_force,
			quantity,
			quote_order_qty,
			price,
			new_client_order_id,
			strategy_id,
			strategy_type,
			stop_price,
			trailing_delta,
			iceberg_qty,
			new_order_resp_type,
			self_trade_prevention_mode,
			recv_window,
			timestamp,
		}
	}
}

impl TryIntoParams for CreateOrderParams {
	fn try_into_params(&self) -> Result<HashMap<String, String>, &'static str> {
		let mut params = HashMap::new();
		params.insert("symbol".to_string(), self.symbol.clone());
		params.insert("side".to_string(), format!("{:?}", self.side));
		params.insert("type".to_string(), format!("{:?}", self.order_type));
		if let Some(ref time_in_force) = self.time_in_force {
			params.insert("timeInForce".to_string(), format!("{:?}", time_in_force));
		}
		if let Some(quantity) = &self.quantity {
			params.insert("quantity".to_string(), quantity.clone());
		}
		if let Some(quote_order_qty) = &self.quote_order_qty {
			params.insert("quoteOrderQty".to_string(), quote_order_qty.clone());
		}
		if let Some(price) = &self.price {
			params.insert("price".to_string(), price.clone());
		}
		if let Some(new_client_order_id) = &self.new_client_order_id {
			params.insert("newClientOrderId".to_string(), new_client_order_id.clone());
		}
		if let Some(strategy_id) = self.strategy_id {
			params.insert("strategyId".to_string(), strategy_id.to_string());
		}
		if let Some(strategy_type) = self.strategy_type {
			params.insert("strategyType".to_string(), strategy_type.to_string());
		}
		if let Some(stop_price) = &self.stop_price {
			params.insert("stopPrice".to_string(), stop_price.clone());
		}
		if let Some(trailing_delta) = self.trailing_delta {
			params.insert("trailingDelta".to_string(), trailing_delta.to_string());
		}
		if let Some(iceberg_qty) = &self.iceberg_qty {
			params.insert("icebergQty".to_string(), iceberg_qty.clone());
		}
		if let Some(ref new_order_resp_type) = self.new_order_resp_type {
			params.insert("newOrderRespType".to_string(), format!("{:?}", new_order_resp_type));
		}
		if let Some(ref self_trade_prevention_mode) = self.self_trade_prevention_mode {
			params.insert(
				"selfTradePreventionMode".to_string(),
				format!("{:?}", self_trade_prevention_mode),
			);
		}
		if let Some(recv_window) = self.recv_window {
			params.insert("recvWindow".to_string(), recv_window.to_string());
		}
		params.insert("timestamp".to_string(), self.timestamp.to_string());
		Ok(params)
	}
}

#[derive(Debug, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum OrderSide {
	BUY,
	SELL,
}

/// This sets how long an order will be active before expiration.
#[derive(Debug, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum TimeInForce {
	GTC,
	IOC,
	FOK,
}

#[derive(Debug, Deserialize, Default)]
#[allow(clippy::upper_case_acronyms)]
pub enum NewOrderRespType {
	#[default]
	ACK,
	RESULT,
	FULL,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeOrder {
	pub symbol: String,
	pub order_id: u64,
	pub order_list_id: i64,
	pub client_order_id: String,
	pub transact_time: u64,
	pub price: String,
	pub orig_qty: String,
	pub executed_qty: String,
	pub orig_quote_order_qty: String,
	pub cummulative_quote_qty: String,
	pub status: OrderStatus,
	pub time_in_force: TimeInForce,
	#[serde(rename = "type")]
	pub order_type: OrderType,
	pub side: OrderSide,
	pub working_time: u64,
	pub self_trade_prevention_mode: SelfTradePreventionMode,
	pub fills: Vec<Fill>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fill {
	pub price: String,
	pub qty: String,
	pub commission: String,
	pub commission_asset: AssetSymbol,
	pub trade_id: u64,
}

#[derive(Debug, Deserialize)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum OrderStatus {
	NEW,
	PENDING_NEW,
	PARTIALLY_FILLED,
	FILLED,
	CANCELED,
	PENDING_CANCEL,
	REJECTED,
	EXPIRED,
	EXPIRED_IN_MATCH,
}
