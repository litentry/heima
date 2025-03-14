use crate::types::AssetSymbol;
use serde::Deserialize;
use std::collections::HashMap;

/// Convertible token pair
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPair {
	pub from_asset: AssetSymbol,
	pub to_asset: AssetSymbol,
	pub from_asset_min_amount: String,
	pub from_asset_max_amount: String,
	pub to_asset_min_amount: String,
	pub to_asset_max_amount: String,
}

/// Asset’s precision information
#[derive(Debug, Deserialize)]
pub struct AssetInfo {
	pub asset: AssetSymbol,
	pub fraction: u32,
}

/// Quote response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
	pub quote_id: String,
	pub ratio: String,
	pub inverse_ratio: String,
	pub valid_timestamp: u64,
	pub to_amount: String,
	pub from_amount: String,
}

/// Convert order status
#[derive(Debug, Deserialize)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum OrderStatus {
	PROCESS,
	ACCEPT_SUCCESS,
	SUCCESS,
	FAIL,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertOrder {
	pub order_id: String,
	pub create_time: u64,
	pub order_status: OrderStatus,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertTrade {
	pub quote_id: String,
	pub order_id: String,
	pub order_status: OrderStatus,
	pub from_asset: AssetSymbol,
	pub from_amount: String,
	pub to_asset: AssetSymbol,
	pub to_amount: String,
	pub ratio: String,
	pub inverse_ratio: String,
	pub create_time: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertTradeHistory {
	pub start_time: u64,
	pub end_time: u64,
	pub limit: u16,
	pub list: Vec<ConvertTrade>,
	pub more_data: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertOrderStatus {
	pub order_id: String,
	pub order_status: OrderStatus,
	pub from_asset: AssetSymbol,
	pub from_amount: String,
	pub to_asset: AssetSymbol,
	pub to_amount: String,
	pub ratio: String,
	pub inverse_ratio: String,
	pub create_time: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrderLimit {
	pub order_id: String,
	pub status: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrders {
	pub list: Vec<OpenOrder>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrder {
	pub quote_id: String,
	pub order_id: String,
	pub order_status: OrderStatus,
	pub from_asset: AssetSymbol,
	pub from_amount: String,
	pub to_asset: AssetSymbol,
	pub to_amount: String,
	pub ratio: String,
	pub inverse_ratio: String,
	pub create_time: u64,
	pub expired_timestamp: u64,
}

pub enum OrderSide {
	Buy,
	Sell,
}

pub enum ExpiredType {
	OneDay,
	ThreeDays,
	SevenDays,
	ThirtyDays,
}

/// base_asset or quote_asset can be determined via exchangeInfo endpoint.
/// Limit price is defined from base_asset to quote_asset.
/// Either base_amount or quote_amount is used.
pub struct PlaceLimitOrderParams {
	base_asset: AssetSymbol,
	quote_asset: AssetSymbol,
	limit_price: String,
	base_amount: Option<String>,
	quote_amount: Option<String>,
	side: OrderSide,
	wallet_type: Option<WalletType>,
	expired_type: ExpiredType,
}

impl PlaceLimitOrderParams {
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		base_asset: AssetSymbol,
		quote_asset: AssetSymbol,
		limit_price: f64,
		side: OrderSide,
		expired_type: ExpiredType,
		base_amount: Option<f64>,
		quote_amount: Option<f64>,
		wallet_type: Option<WalletType>,
	) -> Self {
		Self {
			base_asset,
			quote_asset,
			limit_price: limit_price.to_string(),
			base_amount: base_amount.map(|a| a.to_string()),
			quote_amount: quote_amount.map(|a| a.to_string()),
			side,
			wallet_type,
			expired_type,
		}
	}

	pub fn try_into_params(self) -> Result<HashMap<String, String>, &'static str> {
		let mut params = HashMap::new();
		params.insert("baseAsset".to_string(), self.base_asset);
		params.insert("quoteAsset".to_string(), self.quote_asset);
		params.insert("limitPrice".to_string(), self.limit_price);
		if self.base_amount.is_none() && self.quote_amount.is_none() {
			return Err("Missing amount for limit order");
		}
		if let Some(base_amount) = self.base_amount {
			params.insert("baseAmount".to_string(), base_amount);
		} else if let Some(quote_amount) = self.quote_amount {
			params.insert("quoteAmount".to_string(), quote_amount);
		}
		let side = match self.side {
			OrderSide::Buy => "BUY".to_string(),
			OrderSide::Sell => "SELL".to_string(),
		};
		params.insert("side".to_string(), side);
		if let Some(wallet_type) = self.wallet_type {
			let wallet_type = match wallet_type {
				WalletType::Spot => "SPOT".to_string(),
				WalletType::Funding => "FUNDING".to_string(),
			};
			params.insert("walletType".to_string(), wallet_type);
		}
		let expired_type = match self.expired_type {
			ExpiredType::OneDay => "1_D".to_string(),
			ExpiredType::ThreeDays => "3_D".to_string(),
			ExpiredType::SevenDays => "7_D".to_string(),
			ExpiredType::ThirtyDays => "30_D".to_string(),
		};
		params.insert("expiredType".to_string(), expired_type);

		Ok(params)
	}
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LimitOrder {
	pub quote_id: String,
	pub ratio: String,
	pub inverse_ratio: String,
	pub valid_timestamp: u64,
	pub to_amount: String,
	pub from_amount: String,
}

pub enum WalletType {
	Spot,
	Funding,
}

/// Quote request parameters
pub struct RequestQuoteParams {
	from_asset: AssetSymbol,
	to_asset: AssetSymbol,
	/// When specified, it is the amount you will be debited after the conversion
	from_amount: Option<String>,
	/// When specified, it is the amount you will be credited after the conversion
	to_amount: Option<String>,
	wallet_type: Option<WalletType>,
}

impl RequestQuoteParams {
	pub fn new(
		from_asset: AssetSymbol,
		to_asset: AssetSymbol,
		from_amount: Option<f64>,
		to_amount: Option<f64>,
		wallet_type: Option<WalletType>,
	) -> Self {
		Self {
			from_asset,
			to_asset,
			from_amount: from_amount.map(|a| a.to_string()),
			to_amount: to_amount.map(|a| a.to_string()),
			wallet_type,
		}
	}

	pub fn try_into_params(self) -> Result<HashMap<String, String>, &'static str> {
		let mut params = HashMap::new();
		params.insert("fromAsset".to_string(), self.from_asset);
		params.insert("toAsset".to_string(), self.to_asset);
		if self.from_amount.is_none() && self.to_amount.is_none() {
			return Err("Missing amount for quote");
		}
		if let Some(from_amount) = self.from_amount {
			params.insert("fromAmount".to_string(), from_amount);
		} else if let Some(to_amount) = self.to_amount {
			params.insert("toAmount".to_string(), to_amount);
		}
		if let Some(wallet_type) = self.wallet_type {
			let wallet_type = match wallet_type {
				WalletType::Spot => "SPOT".to_string(),
				WalletType::Funding => "FUNDING".to_string(),
			};
			params.insert("walletType".to_string(), wallet_type);
		}
		Ok(params)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_quote_params_with_from_amount() {
		let from_amount: f64 = 0.1;
		let params = RequestQuoteParams::new(
			"BTC".to_string(),
			"ETH".to_string(),
			Some(from_amount),
			None,
			None,
		);
		let result = params.try_into_params().expect("Should convert to params");
		assert_eq!(result.get("fromAsset"), Some(&"BTC".to_string()));
		assert_eq!(result.get("toAsset"), Some(&"ETH".to_string()));
		assert_eq!(result.get("fromAmount"), Some(&"0.1".to_string()));
		assert_eq!(result.get("toAmount"), None);
		assert_eq!(result.get("walletType"), None);
	}

	#[test]
	fn test_quote_params_with_to_amount() {
		let to_amount: f64 = 1000.0;
		let params = RequestQuoteParams::new(
			"BTC".to_string(),
			"USDT".to_string(),
			None,
			Some(to_amount),
			None,
		);

		let result = params.try_into_params().expect("Should convert to params");
		assert_eq!(result.get("fromAsset"), Some(&"BTC".to_string()));
		assert_eq!(result.get("toAsset"), Some(&"USDT".to_string()));
		assert_eq!(result.get("fromAmount"), None);
		assert_eq!(result.get("toAmount"), Some(&"1000".to_string()));
	}

	#[test]
	fn test_quote_params_with_wallet_type_spot() {
		let params = RequestQuoteParams {
			from_asset: "ETH".to_string(),
			to_asset: "BTC".to_string(),
			from_amount: Some("10".to_string()),
			to_amount: None,
			wallet_type: Some(WalletType::Spot),
		};

		let result = params.try_into_params().expect("Should convert to params");
		assert_eq!(result.get("walletType"), Some(&"SPOT".to_string()));
	}

	#[test]
	fn test_quote_params_with_wallet_type_funding() {
		let params = RequestQuoteParams::new(
			"ETH".to_string(),
			"BTC".to_string(),
			Some(10.0),
			None,
			Some(WalletType::Funding),
		);

		let result = params.try_into_params().expect("Should convert to params");
		assert_eq!(result.get("walletType"), Some(&"FUNDING".to_string()));
	}

	#[test]
	fn test_quote_params_missing_amount() {
		let params =
			RequestQuoteParams::new("ETH".to_string(), "BTC".to_string(), None, None, None);

		let result = params.try_into_params();
		assert!(result.is_err());
		assert_eq!(result.unwrap_err(), "Missing amount for quote");
	}

	#[test]
	fn test_limit_order_req_try_into_params_with_base_amount() {
		let limit_order_params = PlaceLimitOrderParams::new(
			"BTC".to_string(),
			"USDT".to_string(),
			50000.0,
			OrderSide::Buy,
			ExpiredType::OneDay,
			Some(0.1),
			None,
			Some(WalletType::Spot),
		);

		let result = limit_order_params.try_into_params().unwrap();

		assert_eq!(result.get("baseAsset").unwrap(), "BTC");
		assert_eq!(result.get("quoteAsset").unwrap(), "USDT");
		assert_eq!(result.get("limitPrice").unwrap(), "50000");
		assert_eq!(result.get("baseAmount").unwrap(), "0.1");
		assert_eq!(result.get("side").unwrap(), "BUY");
		assert_eq!(result.get("walletType").unwrap(), "SPOT");
		assert_eq!(result.get("expiredType").unwrap(), "1_D");
		assert!(!result.contains_key("quoteAmount"));
	}

	#[test]
	fn test_limit_order_req_try_into_params_with_quote_amount() {
		let limit_order_params = PlaceLimitOrderParams::new(
			"BTC".to_string(),
			"USDT".to_string(),
			50000.0,
			OrderSide::Sell,
			ExpiredType::SevenDays,
			None,
			Some(1000.0),
			Some(WalletType::Funding),
		);

		let result = limit_order_params.try_into_params().unwrap();

		assert_eq!(result.get("quoteAmount").unwrap(), "1000");
		assert_eq!(result.get("side").unwrap(), "SELL");
		assert_eq!(result.get("walletType").unwrap(), "FUNDING");
		assert_eq!(result.get("expiredType").unwrap(), "7_D");
		assert!(!result.contains_key("baseAmount"));
	}

	#[test]
	fn test_limit_order_req_try_into_params_missing_amount() {
		let limit_order_params = PlaceLimitOrderParams::new(
			"BTC".to_string(),
			"USDT".to_string(),
			50000.0,
			OrderSide::Buy,
			ExpiredType::OneDay,
			None,
			None,
			None,
		);

		let result = limit_order_params.try_into_params();
		assert!(result.is_err());
		assert_eq!(result.unwrap_err(), "Missing amount for limit order");
	}

	#[test]
	fn test_limit_order_req_try_into_params_different_expired_types() {
		let expired_types = vec![
			(ExpiredType::OneDay, "1_D"),
			(ExpiredType::ThreeDays, "3_D"),
			(ExpiredType::SevenDays, "7_D"),
			(ExpiredType::ThirtyDays, "30_D"),
		];

		for (expired_type, expected) in expired_types {
			let limit_order_params = PlaceLimitOrderParams::new(
				"BTC".to_string(),
				"USDT".to_string(),
				50000.0,
				OrderSide::Buy,
				expired_type,
				Some(0.1),
				None,
				None,
			);

			let result = limit_order_params.try_into_params().unwrap();
			assert_eq!(result.get("expiredType").unwrap(), expected);
			assert!(!result.contains_key("walletType"));
		}
	}
}
