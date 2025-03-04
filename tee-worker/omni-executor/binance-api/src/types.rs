use serde::{Deserialize, Serialize};

pub type AssetSymbol = String;

/// Convertible token pair
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct AssetInfo {
	pub asset: AssetSymbol,
	pub fraction: u32,
}
