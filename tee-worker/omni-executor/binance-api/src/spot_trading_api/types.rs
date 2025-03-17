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
	#[serde(deserialize_with = "deserialize_permissions")]
	pub permissions: Vec<Permission>,
	#[serde(deserialize_with = "deserialize_permission_sets")]
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

#[derive(Debug, Deserialize, Default)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum OrderType {
	#[default]
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
		applyMinToMarket: bool,
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
		minTrailingAboveDelta: u32,
		maxTrailingAboveDelta: u32,
		minTrailingBelowDelta: u32,
		maxTrailingBelowDelta: u32,
	},
}

#[derive(Debug, Deserialize)]
#[allow(clippy::upper_case_acronyms, non_camel_case_types)]
pub enum Permission {
	SPOT,
	MARGIN,
	LEVERAGED,
	TradingGroup(u32), // Holds the number extracted from "TRD_GRP_XXX"
	Unknown(String),   //  Catch-all for unexpected values
}

impl From<String> for Permission {
	fn from(s: String) -> Self {
		match s.as_str() {
			"SPOT" => Permission::SPOT,
			"MARGIN" => Permission::MARGIN,
			"LEVERAGED" => Permission::LEVERAGED,
			_ => match s.strip_prefix("TRD_GRP_") {
				Some(group_number) => {
					if let Ok(group_number) = group_number.parse() {
						Permission::TradingGroup(group_number)
					} else {
						Permission::Unknown(s)
					}
				},
				None => Permission::Unknown(s),
			},
		}
	}
}

impl std::fmt::Display for Permission {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Permission::SPOT => write!(f, "SPOT"),
			Permission::MARGIN => write!(f, "MARGIN"),
			Permission::LEVERAGED => write!(f, "LEVERAGED"),
			Permission::TradingGroup(group_number) => write!(f, "TRD_GRP_{}", group_number),
			Permission::Unknown(s) => write!(f, "{}", s),
		}
	}
}

fn deserialize_permissions<'de, D>(deserializer: D) -> Result<Vec<Permission>, D::Error>
where
	D: serde::Deserializer<'de>,
{
	let permissions: Vec<String> = Deserialize::deserialize(deserializer)?;
	Ok(permissions.into_iter().map(Permission::from).collect())
}

fn deserialize_permission_sets<'de, D>(deserializer: D) -> Result<Vec<Vec<Permission>>, D::Error>
where
	D: serde::Deserializer<'de>,
{
	let permission_sets: Vec<Vec<String>> = Deserialize::deserialize(deserializer)?;
	Ok(permission_sets
		.into_iter()
		.map(|permissions| permissions.into_iter().map(Permission::from).collect())
		.collect())
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

#[derive(Debug, Deserialize, Default)]
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
	pub recv_window: Option<u32>,
}

impl TryIntoParams for CreateOrderParams {
	fn try_into_params(&self) -> Result<HashMap<String, String>, &'static str> {
		match self.order_type {
			OrderType::LIMIT => {
				if self.price.is_none() && self.quantity.is_none() && self.time_in_force.is_none() {
					return Err("price, quantity, and time_in_force are required for this order type (LIMIT)");
				}
			},
			OrderType::MARKET => {
				if self.quantity.is_none() && self.quote_order_qty.is_none() {
					return Err(
						"quantity or quote_order_qty are required for this order type (MARKET)",
					);
				}
			},
			OrderType::STOP_LOSS => {
				if self.quantity.is_none()
					&& (self.stop_price.is_none() && self.trailing_delta.is_none())
				{
					return Err("quantity, stop_price, or trailing_delta are required for this order type (STOP_LOSS)");
				}
			},
			OrderType::STOP_LOSS_LIMIT => {
				if self.time_in_force.is_none()
					&& self.quantity.is_none()
					&& self.price.is_none()
					&& (self.stop_price.is_none() && self.trailing_delta.is_none())
				{
					return Err("price, quantity, stop_price, and time_in_force are required for this order type (STOP_LOSS_LIMIT)");
				}
			},
			OrderType::TAKE_PROFIT => {
				if self.quantity.is_none()
					&& (self.stop_price.is_none() && self.trailing_delta.is_none())
				{
					return Err("quantity, stop_price, or trailing_delta are required for this order type (TAKE_PROFIT)");
				}
			},
			OrderType::TAKE_PROFIT_LIMIT => {
				if self.time_in_force.is_none()
					&& self.quantity.is_none()
					&& self.price.is_none()
					&& (self.stop_price.is_none() && self.trailing_delta.is_none())
				{
					return Err("price, quantity, stop_price, and time_in_force are required for this order type (TAKE_PROFIT_LIMIT)");
				}
			},
			OrderType::LIMIT_MAKER => {
				if self.price.is_none() && self.quantity.is_none() {
					return Err(
						"price and quantity are required for this order type (LIMIT_MAKER)",
					);
				}
			},
		}

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
		Ok(params)
	}
}

#[derive(Debug, Deserialize, Default)]
#[allow(clippy::upper_case_acronyms)]
pub enum OrderSide {
	#[default]
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
	pub transact_time: Option<u64>,
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
	pub is_working: Option<bool>,
	pub self_trade_prevention_mode: SelfTradePreventionMode,
	pub fills: Option<Vec<Fill>>,
	pub iceberg_qty: Option<String>,
	pub prevented_match_id: Option<u64>,
	pub prevented_quantity: Option<String>,
	pub stop_price: Option<String>,
	pub strategy_id: Option<u64>,
	pub strategy_type: Option<u32>,
	pub trailing_delta: Option<u32>,
	pub trailing_time: Option<i64>,
	pub used_sor: Option<bool>,
	pub working_flor: Option<String>,
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

#[derive(Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum TestTradeOrder {
	WithCommissionRates {
		standard_commission_for_order: CommissionForOrder,
		tax_commission_for_order: CommissionForOrder,
		discount: Discount,
	},
	Empty(EmptyResponse),
}

#[derive(Debug, Deserialize)]
pub struct EmptyResponse {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionForOrder {
	pub maker: String,
	pub taker: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Discount {
	pub enabled_for_account: bool,
	pub enabled_for_symbol: bool,
	pub discount_asset: AssetSymbol,
	pub discount: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum CancelOrderRestrictions {
	ONLY_NEW,
	ONLY_PARTIALLY_FILLED,
	PARTIALLY_FILLED,
}

impl std::fmt::Display for CancelOrderRestrictions {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			CancelOrderRestrictions::ONLY_NEW => write!(f, "ONLY_NEW"),
			CancelOrderRestrictions::ONLY_PARTIALLY_FILLED => write!(f, "ONLY_PARTIALLY_FILLED"),
			CancelOrderRestrictions::PARTIALLY_FILLED => write!(f, "PARTIALLY_FILLED"),
		}
	}
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
	pub maker_commission: u32,
	pub taker_commission: u32,
	pub buyer_commission: u32,
	pub seller_commission: u32,
	pub commission_rates: CommissionRates,
	pub can_trade: bool,
	pub can_withdraw: bool,
	pub can_deposit: bool,
	pub brokered: bool,
	pub require_self_trade_prevention: bool,
	pub prevent_sor: bool,
	pub update_time: u64,
	pub account_type: Permission, // Account type and Symbol Permissions are the same
	pub balances: Vec<Balance>,
	pub permissions: Vec<Permission>,
	pub uid: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionRates {
	pub maker: String,
	pub taker: String,
	pub buyer: String,
	pub seller: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
	pub asset: AssetSymbol,
	pub free: String,
	pub locked: String,
}

#[derive(Debug, Deserialize)]
pub struct SymbolPrice {
	pub price: String,
}
