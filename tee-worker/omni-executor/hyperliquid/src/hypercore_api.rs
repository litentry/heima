use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error};

const HYPERCORE_API_MAINNET: &str = "https://api.hyperliquid.xyz/info";
const HYPERCORE_API_TESTNET: &str = "https://api.hyperliquid-testnet.xyz/info";

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum HyperCoreRequest {
	SpotMeta,
	Meta,
	AllMids,
	OrderStatus { user: String, oid: String },
	SpotClearinghouseState { user: String },
	ClearinghouseState { user: String },
	UserFills { user: String },
}

#[derive(Debug, Deserialize)]
pub struct SpotMetaResponse {
	pub universe: Vec<SpotPair>,
	pub tokens: Vec<SpotToken>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotPair {
	pub tokens: Vec<u32>,
	pub name: String,
	pub index: u32,
	pub is_canonical: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotToken {
	pub name: String,
	pub sz_decimals: u8,
	pub wei_decimals: u8,
	pub index: u32,
	pub token_id: String,
	pub is_canonical: bool,
}

#[derive(Debug, Deserialize)]
pub struct MetaResponse {
	pub universe: Vec<PerpAsset>,
}

#[derive(Debug, Deserialize)]
pub struct AllMidsResponse(pub std::collections::HashMap<String, String>);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerpAsset {
	pub name: String,
	pub sz_decimals: u8,
	pub max_leverage: u32,
}

#[derive(Debug, Deserialize)]
pub struct OrderStatusResponse {
	pub status: String,
	pub order: Option<OrderInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderInfo {
	pub order: OrderDetail,
	pub status: String,
	pub status_timestamp: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderDetail {
	// Perp order fields
	pub asset: Option<u32>,
	pub is_buy: Option<bool>,
	// Spot order fields
	pub coin: Option<String>,
	pub side: Option<String>,
	// Common fields
	pub limit_px: String,
	pub sz: String,
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
#[serde(rename_all = "camelCase")]
pub struct ClearinghouseStateResponse {
	pub asset_positions: Vec<AssetPosition>,
	pub margin_summary: MarginSummary,
	pub cross_margin_summary: CrossMarginSummary,
	pub cross_maintenance_margin_used: String,
	pub withdrawable: String,
	pub time: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetPosition {
	pub position: PositionData,
	#[serde(rename = "type")]
	pub position_type: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PositionData {
	pub coin: String,
	pub entry_px: Option<String>,
	pub leverage: Leverage,
	pub liquidation_px: Option<String>,
	pub margin_used: String,
	pub max_leverage: u32,
	pub position_value: String,
	pub return_on_equity: String,
	pub szi: String,
	pub unrealized_pnl: String,
	// cumFunding is present in the API but we don't need it for our use case
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Leverage {
	#[serde(rename = "type")]
	pub leverage_type: String,
	pub value: u32,
	pub raw_usd: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginSummary {
	pub account_value: String,
	pub total_margin_used: String,
	pub total_ntl_pos: String,
	pub total_raw_usd: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossMarginSummary {
	pub account_value: String,
	pub total_margin_used: String,
	pub total_ntl_pos: String,
	pub total_raw_usd: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fill {
	pub coin: String,
	pub px: String,
	pub sz: String,
	pub side: String,
	pub time: u64,
	pub start_position: String,
	pub dir: String,
	pub closed_pnl: String,
	pub hash: String,
	pub oid: u64,
	pub cloid: Option<String>,
	pub tid: u64,
	pub fee: String,
	pub fee_token: String,
	pub builder_fee: Option<String>,
}

/// Specifies what condition to wait for when polling an order
#[derive(Debug, Clone, Copy)]
pub enum OrderWaitCondition {
	/// Wait until the order is filled
	Filled,
	/// Wait until the order is opened (retrievable via API)
	Opened,
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

	/// Get mid price for a perpetual futures contract
	/// For perps, the key is just the ticker name (e.g., "PURR", "HYPE")
	pub async fn get_perp_mid_price(&self, ticker: &str) -> Result<f64, String> {
		let all_mids = self.get_all_mids().await?;
		let price_str = all_mids
			.0
			.get(ticker)
			.ok_or_else(|| format!("Perp price for {} not found in allMids", ticker))?;
		price_str
			.parse::<f64>()
			.map_err(|e| format!("Failed to parse perp price for {}: {}", ticker, e))
	}

	/// Get mid price for a spot trading pair (ticker/USDC)
	/// For spot, we need to:
	/// 1. Get the token index from spot_meta
	/// 2. Find the trading pair containing (token_index, usdc_index)
	/// 3. Get the pair's "name" field
	/// 4. Use that name as the key in allMids
	///
	/// Example: For HYPE (index 1105), find pair (1105, 0) with name "@1035",
	/// then use "@1035" as the key in allMids
	pub async fn get_spot_mid_price(
		&self,
		ticker: &str,
		spot_meta: &SpotMetaResponse,
	) -> Result<f64, String> {
		// Find the token by ticker
		let token = spot_meta
			.tokens
			.iter()
			.find(|t| t.name.eq_ignore_ascii_case(ticker))
			.ok_or_else(|| format!("Token {} not found in spot meta", ticker))?;

		// Find USDC token (index 0)
		let usdc_token = spot_meta
			.tokens
			.iter()
			.find(|t| t.name.eq_ignore_ascii_case("USDC"))
			.ok_or_else(|| "USDC token not found in spot meta".to_string())?;

		// Find the trading pair containing both tokens
		let pair = spot_meta
			.universe
			.iter()
			.find(|p| p.tokens.contains(&token.index) && p.tokens.contains(&usdc_token.index))
			.ok_or_else(|| format!("No spot pair found for {}/USDC", ticker))?;

		// Get all mid prices
		let all_mids = self.get_all_mids().await?;

		// Use the pair's name as the key
		let price_str = all_mids.0.get(&pair.name).ok_or_else(|| {
			format!("Spot price for {} (pair name: {}) not found in allMids", ticker, pair.name)
		})?;

		price_str
			.parse::<f64>()
			.map_err(|e| format!("Failed to parse spot price for {}: {}", ticker, e))
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

	/// Waits for an order to reach a specific condition by polling the HyperLiquid API.
	///
	/// # Arguments
	/// * `user_address` - The user's wallet address
	/// * `cloid` - The client order ID
	/// * `max_wait_seconds` - Maximum time to wait before timing out
	/// * `condition` - The condition to wait for (Filled or Opened)
	///
	/// # Returns
	/// * `Ok(true)` - Order reached the desired condition
	/// * `Ok(false)` - Order was rejected, canceled, or expired
	/// * `Err(String)` - Timeout or API error
	pub async fn wait_for_order(
		&self,
		user_address: &str,
		cloid: &str,
		max_wait_seconds: u64,
		condition: OrderWaitCondition,
	) -> Result<bool, String> {
		let start_time = std::time::Instant::now();
		let max_duration = Duration::from_secs(max_wait_seconds);
		let condition_name = match condition {
			OrderWaitCondition::Filled => "completion",
			OrderWaitCondition::Opened => "to be opened",
		};

		loop {
			if start_time.elapsed() >= max_duration {
				return Err(format!(
					"Timeout waiting for order {} after {} seconds",
					condition_name, max_wait_seconds
				));
			}

			let status = self.get_order_status(user_address, cloid).await?;

			if let Some(order_info) = status.order {
				let order_status = order_info.status.as_str();

				// Check if the condition is satisfied
				let is_satisfied = match condition {
					OrderWaitCondition::Filled => order_status == "filled",
					OrderWaitCondition::Opened => {
						matches!(order_status, "open" | "partial_fill" | "filled")
					},
				};

				if is_satisfied {
					debug!(
						"Order {} satisfied condition {:?} with status: {}",
						cloid, condition, order_status
					);
					return Ok(true);
				}

				// Check for failure states
				if matches!(order_status, "rejected" | "canceled" | "expired") {
					error!("Order {} failed with status: {}", cloid, order_status);
					return Ok(false);
				}

				// Still pending
				debug!("Order {} still pending: {}", cloid, order_status);
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
	/// * `user_address` - The user's wallet address
	/// * `initial_balance` - The perp balance before the transfer
	/// * `expected_increase` - The amount we expect the balance to increase by
	/// * `max_wait_seconds` - Maximum time to wait before timing out
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

	/// Waits for the spot balance of a specific token to increase by at least the expected amount.
	/// This is useful for verifying that a transfer from perp to spot has completed.
	///
	/// # Arguments
	/// * `user_address` - The user's wallet address
	/// * `ticker` - The token ticker (e.g., "USDC", "ETH")
	/// * `initial_balance` - The spot balance before the transfer
	/// * `expected_increase` - The amount we expect the balance to increase by
	/// * `max_wait_seconds` - Maximum time to wait before timing out
	pub async fn wait_for_spot_balance_increase(
		&self,
		user_address: &str,
		ticker: &str,
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
					"Timeout waiting for spot {} balance to increase from {:.2} by {:.2} (to {:.2}) after {} seconds",
					ticker, initial_balance, expected_increase, expected_final_balance, max_wait_seconds
				));
			}

			let current_balance = self.get_spot_balance(user_address, ticker).await?;

			debug!(
				"Current spot {} balance: {:.2}, initial: {:.2}, expected final: {:.2}",
				ticker, current_balance, initial_balance, expected_final_balance
			);

			// Check if balance has increased by the expected amount (with tolerance)
			if current_balance >= expected_final_balance - tolerance {
				debug!(
					"Spot {} balance increased successfully: {:.2} (increased by {:.2})",
					ticker,
					current_balance,
					current_balance - initial_balance
				);
				return Ok(current_balance);
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

	/// Helper function to print account state for debugging
	pub async fn print_account_state(&self, user_address: &str, label: &str) {
		tracing::info!("========== Account State: {} ==========", label);

		// Print spot balances
		match self.get_spot_clearinghouse_state(user_address).await {
			Ok(spot_state) => {
				tracing::info!("Spot Balances:");
				for balance in &spot_state.balances {
					let total: f64 = balance.total.parse().unwrap_or(0.0);
					let hold: f64 = balance.hold.parse().unwrap_or(0.0);
					if total > 0.0 || hold > 0.0 {
						tracing::info!(
							"  {} - Total: {}, Hold: {}",
							balance.coin,
							balance.total,
							balance.hold
						);
					}
				}
			},
			Err(e) => {
				tracing::info!("Failed to fetch spot balances: {}", e);
			},
		}

		// Print perp clearinghouse state
		match self.get_perp_clearinghouse_state(user_address).await {
			Ok(perp_state) => {
				tracing::info!("Perp Margin Summary:");
				tracing::info!(
					"  Account Value: {}, Total Margin Used: {}, Withdrawable: {}",
					perp_state.margin_summary.account_value,
					perp_state.margin_summary.total_margin_used,
					perp_state.withdrawable
				);

				if !perp_state.asset_positions.is_empty() {
					tracing::info!("Open Positions:");
					for asset_pos in &perp_state.asset_positions {
						let pos = &asset_pos.position;
						tracing::info!(
							"  {} - Size: {}, Entry Px: {}, Position Value: {}, Unrealized PnL: {}, Leverage: {}x",
							pos.coin,
							pos.szi,
							pos.entry_px.as_ref().unwrap_or(&"N/A".to_string()),
							pos.position_value,
							pos.unrealized_pnl,
							pos.leverage.value
						);
					}
				} else {
					tracing::info!("Open Positions: None");
				}
			},
			Err(e) => {
				tracing::info!("Failed to fetch perp clearinghouse state: {}", e);
			},
		}

		tracing::info!("==========================================");
	}
}
