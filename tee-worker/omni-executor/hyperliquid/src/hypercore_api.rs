use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error};

const HYPERCORE_API_MAINNET: &str = "https://api.hyperliquid.xyz/info";
const HYPERCORE_API_TESTNET: &str = "https://api.hyperliquid-testnet.xyz/info";

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum HyperCoreRequest {
	#[serde(rename = "spotMeta")]
	SpotMeta,
	#[serde(rename = "meta")]
	Meta,
	#[serde(rename = "allMids")]
	AllMids,
	#[serde(rename = "orderStatus")]
	OrderStatus { user: String, oid: String },
	#[serde(rename = "spotClearinghouseState")]
	SpotClearinghouseState { user: String },
	#[serde(rename = "clearinghouseState")]
	ClearinghouseState { user: String },
	#[serde(rename = "userFills")]
	UserFills { user: String },
}

#[derive(Debug, Deserialize)]
pub struct SpotMetaResponse {
	pub universe: Vec<SpotPair>,
	pub tokens: Vec<SpotToken>,
}

#[derive(Debug, Deserialize)]
pub struct SpotPair {
	pub tokens: Vec<u32>,
	pub name: String,
	pub index: u32,
	#[serde(rename = "isCanonical")]
	pub is_canonical: bool,
}

#[derive(Debug, Deserialize)]
pub struct SpotToken {
	pub name: String,
	#[serde(rename = "szDecimals")]
	pub sz_decimals: u8,
	#[serde(rename = "weiDecimals")]
	pub wei_decimals: u8,
	pub index: u32,
	#[serde(rename = "tokenId")]
	pub token_id: String,
	#[serde(rename = "isCanonical")]
	pub is_canonical: bool,
}

#[derive(Debug, Deserialize)]
pub struct MetaResponse {
	pub universe: Vec<PerpAsset>,
}

#[derive(Debug, Deserialize)]
pub struct AllMidsResponse(pub std::collections::HashMap<String, String>);

#[derive(Debug, Deserialize)]
pub struct PerpAsset {
	pub name: String,
	#[serde(rename = "szDecimals")]
	pub sz_decimals: u8,
	#[serde(rename = "maxLeverage")]
	pub max_leverage: u32,
}

#[derive(Debug, Deserialize)]
pub struct OrderStatusResponse {
	pub status: String,
	pub order: Option<OrderInfo>,
}

#[derive(Debug, Deserialize)]
pub struct OrderInfo {
	pub order: OrderDetail,
	pub status: String,
	#[serde(rename = "statusTimestamp")]
	pub status_timestamp: u64,
}

#[derive(Debug, Deserialize)]
pub struct OrderDetail {
	// Perp order fields
	pub asset: Option<u32>,
	#[serde(rename = "isBuy")]
	pub is_buy: Option<bool>,
	// Spot order fields
	pub coin: Option<String>,
	pub side: Option<String>,
	// Common fields
	#[serde(rename = "limitPx")]
	pub limit_px: String,
	pub sz: String,
	#[serde(rename = "reduceOnly")]
	pub reduce_only: bool,
	pub cloid: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SpotClearinghouseStateResponse {
	pub balances: Vec<SpotBalance>,
}

#[derive(Debug, Deserialize)]
pub struct SpotBalance {
	pub coin: String,
	pub hold: String,
	pub total: String,
}

#[derive(Debug, Deserialize)]
pub struct ClearinghouseStateResponse {
	#[serde(rename = "assetPositions")]
	pub asset_positions: Vec<AssetPosition>,
	#[serde(rename = "marginSummary")]
	pub margin_summary: MarginSummary,
	#[serde(rename = "crossMarginSummary")]
	pub cross_margin_summary: CrossMarginSummary,
	pub withdrawable: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AssetPosition {
	pub position: PositionData,
	#[serde(rename = "type")]
	pub position_type: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PositionData {
	pub coin: String,
	#[serde(rename = "entryPx")]
	pub entry_px: Option<String>,
	pub leverage: Leverage,
	#[serde(rename = "liquidationPx")]
	pub liquidation_px: Option<String>,
	#[serde(rename = "marginUsed")]
	pub margin_used: String,
	#[serde(rename = "maxTrade")]
	pub max_trade: String,
	#[serde(rename = "positionValue")]
	pub position_value: String,
	#[serde(rename = "returnOnEquity")]
	pub return_on_equity: String,
	pub szi: String,
	#[serde(rename = "unrealizedPnl")]
	pub unrealized_pnl: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Leverage {
	#[serde(rename = "type")]
	pub leverage_type: String,
	pub value: u32,
	#[serde(rename = "rawUsd")]
	pub raw_usd: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MarginSummary {
	#[serde(rename = "accountValue")]
	pub account_value: String,
	#[serde(rename = "totalMarginUsed")]
	pub total_margin_used: String,
	#[serde(rename = "totalNtlPos")]
	pub total_ntl_pos: String,
	#[serde(rename = "totalRawUsd")]
	pub total_raw_usd: String,
}

#[derive(Debug, Deserialize)]
pub struct CrossMarginSummary {
	#[serde(rename = "accountValue")]
	pub account_value: String,
	#[serde(rename = "totalMarginUsed")]
	pub total_margin_used: String,
	#[serde(rename = "totalNtlPos")]
	pub total_ntl_pos: String,
	#[serde(rename = "totalRawUsd")]
	pub total_raw_usd: String,
}

#[derive(Debug, Deserialize)]
pub struct Fill {
	pub coin: String,
	pub px: String,
	pub sz: String,
	pub side: String,
	pub time: u64,
	#[serde(rename = "startPosition")]
	pub start_position: String,
	pub dir: String,
	#[serde(rename = "closedPnl")]
	pub closed_pnl: String,
	pub hash: String,
	pub oid: u64,
	pub cloid: Option<String>,
	pub tid: u64,
	pub fee: String,
	#[serde(rename = "feeToken")]
	pub fee_token: String,
	#[serde(rename = "builderFee")]
	pub builder_fee: Option<String>,
}

pub struct HyperCoreClient {
	api_url: String,
	client: reqwest::Client,
}

impl HyperCoreClient {
	pub fn new(chain_id: u64) -> Result<Self, String> {
		let api_url = match chain_id {
			999 => HYPERCORE_API_MAINNET,
			998 => HYPERCORE_API_TESTNET,
			_ => return Err(format!("Unsupported chain_id for HyperCore: {}", chain_id)),
		};

		let client = reqwest::Client::builder()
			.timeout(Duration::from_secs(30))
			.build()
			.map_err(|e| format!("Failed to create HTTP client: {}", e))?;

		Ok(Self { api_url: api_url.to_string(), client })
	}

	pub async fn get_spot_meta(&self) -> Result<SpotMetaResponse, String> {
		let request = HyperCoreRequest::SpotMeta;

		debug!("Fetching spot meta from HyperCore API");

		let response = self
			.client
			.post(&self.api_url)
			.json(&request)
			.send()
			.await
			.map_err(|e| format!("Failed to send spot meta request: {}", e))?;

		if !response.status().is_success() {
			let status = response.status();
			let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
			error!("HyperCore API error {}: {}", status, error_text);
			return Err(format!("HyperCore API error {}: {}", status, error_text));
		}

		response
			.json::<SpotMetaResponse>()
			.await
			.map_err(|e| format!("Failed to parse spot meta response: {}", e))
	}

	pub async fn get_meta(&self) -> Result<MetaResponse, String> {
		let request = HyperCoreRequest::Meta;

		debug!("Fetching perp meta from HyperCore API");

		let response = self
			.client
			.post(&self.api_url)
			.json(&request)
			.send()
			.await
			.map_err(|e| format!("Failed to send meta request: {}", e))?;

		if !response.status().is_success() {
			let status = response.status();
			let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
			error!("HyperCore API error {}: {}", status, error_text);
			return Err(format!("HyperCore API error {}: {}", status, error_text));
		}

		response
			.json::<MetaResponse>()
			.await
			.map_err(|e| format!("Failed to parse meta response: {}", e))
	}

	pub async fn get_all_mids(&self) -> Result<AllMidsResponse, String> {
		let request = HyperCoreRequest::AllMids;

		debug!("Fetching all mid prices from HyperCore API");

		let response = self
			.client
			.post(&self.api_url)
			.json(&request)
			.send()
			.await
			.map_err(|e| format!("Failed to send allMids request: {}", e))?;

		if !response.status().is_success() {
			let status = response.status();
			let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
			error!("HyperCore API error {}: {}", status, error_text);
			return Err(format!("HyperCore API error {}: {}", status, error_text));
		}

		response
			.json::<AllMidsResponse>()
			.await
			.map_err(|e| format!("Failed to parse allMids response: {}", e))
	}

	pub async fn get_mid_price(&self, ticker: &str) -> Result<f64, String> {
		let all_mids = self.get_all_mids().await?;
		let price_str = all_mids
			.0
			.get(ticker)
			.ok_or_else(|| format!("Price for {} not found in allMids", ticker))?;
		price_str
			.parse::<f64>()
			.map_err(|e| format!("Failed to parse price for {}: {}", ticker, e))
	}

	pub async fn get_order_status(
		&self,
		user_address: &str,
		cloid: &str,
	) -> Result<OrderStatusResponse, String> {
		// Convert cloid to 16-byte hex string as per HyperLiquid API spec
		// The API expects "Either u64 representing the order id or 16-byte hex string representing the client order id"
		let cloid_u128 = cloid
			.parse::<u128>()
			.map_err(|e| format!("Failed to parse cloid as u128: {}", e))?;

		// Convert to 16-byte hex string (32 hex chars) with 0x prefix
		let cloid_hex = format!("0x{:032x}", cloid_u128);

		let request = HyperCoreRequest::OrderStatus {
			user: user_address.to_string(),
			oid: cloid_hex.clone(),
		};

		debug!("Querying order status for cloid: {} (raw: {}, hex: {})", cloid, cloid, cloid_hex);

		let response = self
			.client
			.post(&self.api_url)
			.json(&request)
			.send()
			.await
			.map_err(|e| format!("Failed to send order status request: {}", e))?;

		if !response.status().is_success() {
			let status = response.status();
			let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
			error!("HyperCore API error {}: {}", status, error_text);
			return Err(format!("HyperCore API error {}: {}", status, error_text));
		}

		response
			.json::<OrderStatusResponse>()
			.await
			.map_err(|e| format!("Failed to parse order status response: {}", e))
	}

	pub async fn wait_for_order_completion(
		&self,
		user_address: &str,
		cloid: &str,
		max_wait_seconds: u64,
	) -> Result<bool, String> {
		let start_time = std::time::Instant::now();
		let max_duration = Duration::from_secs(max_wait_seconds);

		loop {
			if start_time.elapsed() >= max_duration {
				return Err(format!(
					"Timeout waiting for order completion after {} seconds",
					max_wait_seconds
				));
			}

			let status = self.get_order_status(user_address, cloid).await?;

			if let Some(order_info) = status.order {
				match order_info.status.as_str() {
					"filled" => {
						debug!("Order {} filled successfully", cloid);
						return Ok(true);
					},
					"rejected" | "canceled" | "expired" => {
						error!("Order {} failed with status: {}", cloid, order_info.status);
						return Ok(false);
					},
					"open" | "partial_fill" => {
						debug!("Order {} still pending: {}", cloid, order_info.status);
					},
					_ => {
						debug!("Order {} unknown status: {}", cloid, order_info.status);
					},
				}
			} else {
				debug!("Order {} not found yet, waiting...", cloid);
			}

			tokio::time::sleep(Duration::from_secs(2)).await;
		}
	}

	/// Waits for the perp account balance to increase by at least the expected amount.
	/// This is useful for verifying that a USD class transfer has completed.
	///
	/// # Arguments
	/// * `initial_balance` - The perp balance before the transfer
	/// * `expected_increase` - The amount we expect the balance to increase by
	pub async fn wait_for_perp_balance_increase(
		&self,
		user_address: &str,
		initial_balance: f64,
		expected_increase: f64,
		max_wait_seconds: u64,
	) -> Result<f64, String> {
		let start_time = std::time::Instant::now();
		let max_duration = Duration::from_secs(max_wait_seconds);
		let tolerance = 0.01; // 1 cent tolerance for floating point comparison
		let expected_final_balance = initial_balance + expected_increase;

		loop {
			if start_time.elapsed() >= max_duration {
				return Err(format!(
					"Timeout waiting for perp balance to increase from {:.2} by {:.2} (to {:.2}) after {} seconds",
					initial_balance, expected_increase, expected_final_balance, max_wait_seconds
				));
			}

			let state = self.get_perp_clearinghouse_state(user_address).await?;
			let account_value = state
				.cross_margin_summary
				.account_value
				.parse::<f64>()
				.map_err(|e| format!("Failed to parse account value: {}", e))?;

			debug!(
				"Current perp account value: {:.2}, initial: {:.2}, expected final: {:.2}",
				account_value, initial_balance, expected_final_balance
			);

			// Check if account value has increased by the expected amount (with tolerance)
			if account_value >= expected_final_balance - tolerance {
				debug!(
					"Perp balance increased successfully: {:.2} (increased by {:.2})",
					account_value,
					account_value - initial_balance
				);
				return Ok(account_value);
			}

			tokio::time::sleep(Duration::from_secs(2)).await;
		}
	}

	pub async fn get_spot_balance(&self, user_address: &str, ticker: &str) -> Result<f64, String> {
		let request = HyperCoreRequest::SpotClearinghouseState { user: user_address.to_string() };

		debug!("Fetching spot balance for {} from HyperCore API", user_address);

		let response = self
			.client
			.post(&self.api_url)
			.json(&request)
			.send()
			.await
			.map_err(|e| format!("Failed to send spot clearinghouse state request: {}", e))?;

		if !response.status().is_success() {
			let status = response.status();
			let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
			error!("HyperCore API error {}: {}", status, error_text);
			return Err(format!("HyperCore API error {}: {}", status, error_text));
		}

		let state = response
			.json::<SpotClearinghouseStateResponse>()
			.await
			.map_err(|e| format!("Failed to parse spot clearinghouse state response: {}", e))?;

		let balance = state
			.balances
			.iter()
			.find(|b| b.coin.eq_ignore_ascii_case(ticker))
			.ok_or_else(|| format!("Token {} not found in user balances", ticker))?;

		balance
			.total
			.parse::<f64>()
			.map_err(|e| format!("Failed to parse balance value: {}", e))
	}

	pub async fn get_spot_clearinghouse_state(
		&self,
		user_address: &str,
	) -> Result<SpotClearinghouseStateResponse, String> {
		let request = HyperCoreRequest::SpotClearinghouseState { user: user_address.to_string() };

		debug!("Fetching spot clearinghouse state for {} from HyperCore API", user_address);

		let response = self
			.client
			.post(&self.api_url)
			.json(&request)
			.send()
			.await
			.map_err(|e| format!("Failed to send spot clearinghouse state request: {}", e))?;

		if !response.status().is_success() {
			let status = response.status();
			let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
			error!("HyperCore API error {}: {}", status, error_text);
			return Err(format!("HyperCore API error {}: {}", status, error_text));
		}

		response
			.json::<SpotClearinghouseStateResponse>()
			.await
			.map_err(|e| format!("Failed to parse spot clearinghouse state response: {}", e))
	}

	pub async fn get_perp_clearinghouse_state(
		&self,
		user_address: &str,
	) -> Result<ClearinghouseStateResponse, String> {
		let request = HyperCoreRequest::ClearinghouseState { user: user_address.to_string() };

		debug!("Fetching perp clearinghouse state for {} from HyperCore API", user_address);

		let response = self
			.client
			.post(&self.api_url)
			.json(&request)
			.send()
			.await
			.map_err(|e| format!("Failed to send clearinghouse state request: {}", e))?;

		if !response.status().is_success() {
			let status = response.status();
			let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
			error!("HyperCore API error {}: {}", status, error_text);
			return Err(format!("HyperCore API error {}: {}", status, error_text));
		}

		response
			.json::<ClearinghouseStateResponse>()
			.await
			.map_err(|e| format!("Failed to parse clearinghouse state response: {}", e))
	}

	pub async fn get_open_positions(
		&self,
		user_address: &str,
	) -> Result<Vec<AssetPosition>, String> {
		let state = self.get_perp_clearinghouse_state(user_address).await?;
		Ok(state.asset_positions)
	}

	/// Get user fills (up to 2000 most recent fills)
	pub async fn get_user_fills(&self, user_address: &str) -> Result<Vec<Fill>, String> {
		let request = HyperCoreRequest::UserFills { user: user_address.to_string() };

		debug!("Fetching user fills for {} from HyperCore API", user_address);

		let response = self
			.client
			.post(&self.api_url)
			.json(&request)
			.send()
			.await
			.map_err(|e| format!("Failed to send user fills request: {}", e))?;

		if !response.status().is_success() {
			let status = response.status();
			let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
			error!("HyperCore API error {}: {}", status, error_text);
			return Err(format!("HyperCore API error {}: {}", status, error_text));
		}

		response
			.json::<Vec<Fill>>()
			.await
			.map_err(|e| format!("Failed to parse user fills response: {}", e))
	}

	/// Get a specific fill by client order ID (cloid)
	pub async fn get_fill_by_cloid(&self, user_address: &str, cloid: u128) -> Result<Fill, String> {
		let fills = self.get_user_fills(user_address).await?;

		// Convert cloid to hex string format for comparison
		let cloid_hex = format!("0x{:032x}", cloid);

		debug!("Searching for fill with cloid: {} (hex: {})", cloid, cloid_hex);

		fills
			.into_iter()
			.find(|fill| {
				if let Some(ref fill_cloid) = fill.cloid {
					fill_cloid == &cloid_hex || fill_cloid == &cloid.to_string()
				} else {
					false
				}
			})
			.ok_or_else(|| format!("No fill found with cloid: {}", cloid))
	}
}
