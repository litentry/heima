use serde::{Deserialize, Serialize};

/// Convertible token pair
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPair {
	pub from_asset: String,
	pub to_asset: String,
	pub from_asset_min_amount: String,
	pub from_asset_max_amount: String,
	pub to_asset_min_amount: String,
	pub to_asset_max_amount: String,
}
