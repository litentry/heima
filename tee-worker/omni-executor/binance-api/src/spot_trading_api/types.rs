use crate::types::AssetSymbol;
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
