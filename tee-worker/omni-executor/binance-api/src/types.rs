use serde::Deserialize;
use std::collections::HashMap;

pub type AssetSymbol = String;

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
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
	Process,
	AcceptSuccess,
	Success,
	Fail,
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
}
