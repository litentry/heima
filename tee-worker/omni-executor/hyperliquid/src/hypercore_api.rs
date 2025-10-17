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

pub fn get_spot_asset_id(ticker: &str, spot_meta: &SpotMetaResponse) -> Result<u32, String> {
	let token = spot_meta
		.tokens
		.iter()
		.find(|t| t.name.eq_ignore_ascii_case(ticker))
		.ok_or_else(|| format!("Token {} not found in spot meta", ticker))?;

	let usdc_token = spot_meta
		.tokens
		.iter()
		.find(|t| t.name.eq_ignore_ascii_case("USDC"))
		.ok_or_else(|| "USDC token not found in spot meta".to_string())?;

	let pair = spot_meta
		.universe
		.iter()
		.find(|p| p.tokens.contains(&token.index) && p.tokens.contains(&usdc_token.index))
		.ok_or_else(|| format!("No spot pair found for {}/USDC", ticker))?;

	Ok(10000 + pair.index)
}

pub fn get_perp_asset_id(ticker: &str, meta: &MetaResponse) -> Result<u32, String> {
	let asset = meta
		.universe
		.iter()
		.enumerate()
		.find(|(_, a)| a.name.eq_ignore_ascii_case(ticker))
		.ok_or_else(|| format!("Perp asset {} not found in meta", ticker))?;

	Ok(asset.0 as u32)
}

/// Calculate the USDC received from a spot sell fill
/// For a sell order: USDC received = price * size - fee (if fee is in USDC)
/// Returns the net USDC amount received
pub fn calculate_usdc_received_from_spot_sell(fill: &Fill) -> Result<f64, String> {
	// Parse price and size
	let price = fill
		.px
		.parse::<f64>()
		.map_err(|e| format!("Failed to parse fill price: {}", e))?;
	let size = fill
		.sz
		.parse::<f64>()
		.map_err(|e| format!("Failed to parse fill size: {}", e))?;
	let fee = fill
		.fee
		.parse::<f64>()
		.map_err(|e| format!("Failed to parse fill fee: {}", e))?;

	// Verify it's a sell order
	if !fill.side.eq_ignore_ascii_case("A") && !fill.dir.to_lowercase().contains("sell") {
		return Err(format!("Fill is not a sell order: side={}, dir={}", fill.side, fill.dir));
	}

	// Calculate gross USDC from the trade (price * size for sell)
	let gross_usdc = price * size;

	// Subtract fee if it's in USDC
	let net_usdc = if fill.fee_token.eq_ignore_ascii_case("USDC") {
		gross_usdc - fee
	} else {
		// If fee is not in USDC, we don't subtract it from the USDC amount
		// but we log a warning
		debug!("Warning: Fee is in {} not USDC, returning gross USDC amount", fill.fee_token);
		gross_usdc
	};

	debug!(
		"Spot sell fill: price={}, size={}, fee={} {}, gross_usdc={:.2}, net_usdc={:.2}",
		price, size, fee, fill.fee_token, gross_usdc, net_usdc
	);

	Ok(net_usdc)
}

/// Clamps a price to comply with HyperLiquid tick size rules.
///
/// Price rules:
/// - Maximum 5 significant figures (excluding leading zeros)
/// - Maximum decimal places = MAX_DECIMALS - sz_decimals
///   - For perps: MAX_DECIMALS = 6
///   - For spot: MAX_DECIMALS = 8
/// - Integer prices are always allowed (regardless of significant figures)
///
/// # Arguments
/// * `price` - The price to clamp
/// * `sz_decimals` - The size decimals of the asset
/// * `is_spot` - Whether this is a spot market (true) or perp market (false)
///
/// # Returns
/// The clamped price as a string
pub fn clamp_price(price: f64, sz_decimals: u8, is_spot: bool) -> String {
	const MAX_SIGNIFICANT_FIGURES: usize = 5;
	const PERP_MAX_DECIMALS: u8 = 6;
	const SPOT_MAX_DECIMALS: u8 = 8;

	// Determine max decimal places based on market type
	let max_decimals = if is_spot { SPOT_MAX_DECIMALS } else { PERP_MAX_DECIMALS };
	let max_price_decimals = max_decimals.saturating_sub(sz_decimals);

	// Check if the price is effectively an integer
	if (price - price.floor()).abs() < 1e-10 {
		// Integer prices are always allowed
		return format!("{:.0}", price);
	}

	// Convert to string to analyze significant figures
	let price_str = format!("{:.15}", price); // Use high precision initially

	// Count significant figures (excluding leading zeros)
	let mut _sig_figs = 0;
	let mut counting = false;
	let mut _decimal_point_seen = false;
	let mut _decimal_places = 0;

	for ch in price_str.chars() {
		if ch == '.' {
			_decimal_point_seen = true;
		} else if ch.is_ascii_digit() {
			if ch != '0' || counting {
				_sig_figs += 1;
				counting = true;
			}
			if _decimal_point_seen {
				_decimal_places += 1;
			}
		}
	}

	// Determine the limiting factor: significant figures or decimal places
	let target_decimals = if price >= 1.0 {
		// For prices >= 1, significant figures typically limit first
		// Calculate how many decimal places we can have with 5 sig figs
		let integer_part = price.floor();
		let integer_digits =
			if integer_part == 0.0 { 0 } else { (integer_part.log10().floor() as usize) + 1 };
		let decimals_from_sig_figs = MAX_SIGNIFICANT_FIGURES.saturating_sub(integer_digits);

		// Take the minimum of sig fig limit and max decimals limit
		std::cmp::min(decimals_from_sig_figs, max_price_decimals as usize)
	} else {
		// For prices < 1, we need to count leading zeros after decimal point
		let mut leading_zeros = 0;
		let mut after_decimal = false;
		for ch in price_str.chars() {
			if ch == '.' {
				after_decimal = true;
			} else if after_decimal {
				if ch == '0' {
					leading_zeros += 1;
				} else {
					break;
				}
			}
		}

		// With 5 sig figs, we can have leading_zeros + 5 total decimal places
		let decimals_from_sig_figs = leading_zeros + MAX_SIGNIFICANT_FIGURES;

		// Take the minimum of sig fig limit and max decimals limit
		std::cmp::min(decimals_from_sig_figs, max_price_decimals as usize)
	};

	// Truncate to target decimal places (drop extra digits, don't round)
	let multiplier = 10f64.powi(target_decimals as i32);
	let truncated = (price * multiplier).floor() / multiplier;

	// Format with the appropriate number of decimals, removing trailing zeros
	let formatted = format!("{:.prec$}", truncated, prec = target_decimals);

	// Remove trailing zeros and decimal point if not needed
	let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');

	// If we ended up with an empty string or just "0", return the price as is
	if trimmed.is_empty() || trimmed == "0" {
		format!("{}", price)
	} else {
		trimmed.to_string()
	}
}

/// Clamps a size to comply with HyperLiquid lot size rules.
///
/// Size rules:
/// - Sizes are rounded to the sz_decimals of the asset
/// - Example: if sz_decimals = 3, then 1.001 is valid but 1.0001 is not
///
/// # Arguments
/// * `size` - The size to clamp
/// * `sz_decimals` - The size decimals of the asset
///
/// # Returns
/// The clamped size as a string
pub fn clamp_size(size: f64, sz_decimals: u8) -> String {
	// Round to sz_decimals
	let multiplier = 10f64.powi(sz_decimals as i32);
	let rounded = (size * multiplier).round() / multiplier;

	// Format with the appropriate number of decimals
	if sz_decimals == 0 {
		// For integer sizes, format as integer
		format!("{:.0}", rounded)
	} else {
		let formatted = format!("{:.prec$}", rounded, prec = sz_decimals as usize);
		// Remove trailing zeros and decimal point if not needed, but only after the decimal point
		let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
		trimmed.to_string()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_clamp_size() {
		// Test with sz_decimals = 3
		assert_eq!(clamp_size(1.001, 3), "1.001");
		assert_eq!(clamp_size(1.0001, 3), "1"); // Should round down
		assert_eq!(clamp_size(1.0015, 3), "1.002"); // Should round up
		assert_eq!(clamp_size(1.0, 3), "1");
		assert_eq!(clamp_size(0.999, 3), "0.999");

		// Test with sz_decimals = 0 (integer only)
		assert_eq!(clamp_size(1.5, 0), "2");
		assert_eq!(clamp_size(1.4, 0), "1");
		assert_eq!(clamp_size(10.0, 0), "10");

		// Test with sz_decimals = 2
		assert_eq!(clamp_size(1.234, 2), "1.23");
		assert_eq!(clamp_size(1.236, 2), "1.24");
		assert_eq!(clamp_size(0.01, 2), "0.01");
	}

	#[test]
	fn test_clamp_price_perp_basic() {
		// Perp: MAX_DECIMALS = 6, sz_decimals impacts max price decimals
		// With sz_decimals = 0, max price decimals = 6

		// Test 5 significant figures limit
		assert_eq!(clamp_price(1234.5, 0, false), "1234.5");
		assert_eq!(clamp_price(1234.56, 0, false), "1234.5"); // Truncates to 5 sig figs

		// Test small numbers
		assert_eq!(clamp_price(0.001234, 0, false), "0.001234"); // 4 sig figs, 6 decimals - OK
		assert_eq!(clamp_price(0.0012345, 0, false), "0.001234"); // 5 sig figs, 7 decimals - truncated to 6 decimals

		// Test integer prices (always allowed)
		assert_eq!(clamp_price(123456.0, 0, false), "123456");
		assert_eq!(clamp_price(1000000.0, 0, false), "1000000");
	}

	#[test]
	fn test_clamp_price_perp_with_sz_decimals() {
		// With sz_decimals = 2, max price decimals = 6 - 2 = 4

		// Test that max decimals constraint applies
		assert_eq!(clamp_price(1.123456, 2, false), "1.1234"); // Limited to 4 decimals, truncated
		assert_eq!(clamp_price(10.12345, 2, false), "10.123"); // 5 sig figs = 3 decimals after 10

		// Test with sz_decimals = 4, max price decimals = 2
		assert_eq!(clamp_price(1.234, 4, false), "1.23"); // Limited to 2 decimals
		assert_eq!(clamp_price(100.12, 4, false), "100.12"); // 5 sig figs (1,0,0,1,2) = 2 decimals, max 2 allowed
	}

	#[test]
	fn test_clamp_price_spot_basic() {
		// Spot: MAX_DECIMALS = 8, sz_decimals impacts max price decimals
		// With sz_decimals = 0, max price decimals = 8

		// Test 5 significant figures limit
		assert_eq!(clamp_price(1234.5, 0, true), "1234.5");
		assert_eq!(clamp_price(1234.56, 0, true), "1234.5"); // Truncates to 5 sig figs

		// Test small numbers - spot allows more decimals
		assert_eq!(clamp_price(0.00001234, 0, true), "0.00001234"); // 4 sig figs, 8 decimals - OK
		assert_eq!(clamp_price(0.000012345, 0, true), "0.00001234"); // 5 sig figs, 9 decimals - truncated to 8

		// Test integer prices (always allowed)
		assert_eq!(clamp_price(123456.0, 0, true), "123456");
	}

	#[test]
	fn test_clamp_price_spot_with_sz_decimals() {
		// With sz_decimals = 1, max price decimals = 8 - 1 = 7
		assert_eq!(clamp_price(0.0001234, 1, true), "0.0001234");
		assert_eq!(clamp_price(0.00012345, 1, true), "0.0001234"); // Truncates to 5 sig figs

		// With sz_decimals = 2, max price decimals = 8 - 2 = 6
		// This should be invalid per docs if sz_decimals > 2
		assert_eq!(clamp_price(0.0001234, 2, true), "0.000123"); // Limited to 6 decimals

		// With sz_decimals = 6, max price decimals = 2
		assert_eq!(clamp_price(1.234, 6, true), "1.23"); // Limited to 2 decimals
	}

	#[test]
	fn test_clamp_price_edge_cases() {
		// Test zero
		assert_eq!(clamp_price(0.0, 0, false), "0");

		// Test very small numbers
		assert_eq!(clamp_price(0.00000001, 0, false), "0.00000001"); // 1 sig fig, 8 decimals

		// Test truncation behavior
		assert_eq!(clamp_price(1.23456, 0, false), "1.2345"); // Truncates
		assert_eq!(clamp_price(1.23454, 0, false), "1.2345"); // Truncates

		// Test that integers don't get unnecessary decimals
		assert_eq!(clamp_price(100.0, 0, false), "100");
		assert_eq!(clamp_price(1.0, 3, false), "1");
	}

	#[test]
	fn test_clamp_price_sig_figs_vs_decimals() {
		// Test cases where significant figures limit comes into play

		// For 1234.5 with sz_decimals=0, perp:
		// - Max price decimals = 6
		// - Integer digits = 4, so with 5 sig figs we can have 1 decimal
		// - Min(1, 6) = 1 decimal allowed
		assert_eq!(clamp_price(1234.56, 0, false), "1234.5"); // Truncates

		// For 12.345 with sz_decimals=0, perp:
		// - Max price decimals = 6
		// - Integer digits = 2, so with 5 sig figs we can have 3 decimals
		// - Min(3, 6) = 3 decimals allowed
		assert_eq!(clamp_price(12.3456, 0, false), "12.345"); // Truncates

		// For 1.23456 with sz_decimals=0, perp:
		// - Max price decimals = 6
		// - Integer digits = 1, so with 5 sig figs we can have 4 decimals
		// - Min(4, 6) = 4 decimals allowed
		assert_eq!(clamp_price(1.234567, 0, false), "1.2345"); // Truncates
	}

	#[test]
	fn test_clamp_price_small_numbers() {
		// Test prices less than 1
		// For perp: max_decimals = 6

		// 0.001234 has 4 sig figs, 6 decimals - OK
		assert_eq!(clamp_price(0.001234, 0, false), "0.001234");

		// 0.0012345 has 5 sig figs, 7 decimals - truncated to 6 decimals
		assert_eq!(clamp_price(0.0012345, 0, false), "0.001234");

		// 0.00123456 has 6 sig figs, should be truncated to 5 sig figs = 6 decimals
		assert_eq!(clamp_price(0.00123456, 0, false), "0.001234");

		// Very small number with leading zeros - these exceed 6 decimals for perp
		// 0.0000012345 has 5 sig figs, 10 decimals - clamped to 6 decimals = 0.000001
		assert_eq!(clamp_price(0.0000012345, 0, false), "0.000001");
		// 0.00000123456 has 6 sig figs, 11 decimals - clamped to 5 sig figs and 6 decimals
		assert_eq!(clamp_price(0.00000123456, 0, false), "0.000001");
	}

	#[test]
	fn test_real_world_examples() {
		// ETH price around 2000 with perp (sz_decimals typically 3)
		// Max price decimals = 6 - 3 = 3
		assert_eq!(clamp_price(2000.5, 3, false), "2000.5");
		assert_eq!(clamp_price(2000.12, 3, false), "2000.1"); // 5 sig figs = 1 decimal

		// BTC price around 50000 with perp (sz_decimals typically 4)
		// Max price decimals = 6 - 4 = 2
		assert_eq!(clamp_price(50000.0, 4, false), "50000");
		assert_eq!(clamp_price(50123.0, 4, false), "50123");
		assert_eq!(clamp_price(50123.45, 4, false), "50123"); // 5 sig figs exceeded

		// Small cap token at 0.001 on spot (sz_decimals 0)
		// Max price decimals = 8
		assert_eq!(clamp_price(0.001234, 0, true), "0.001234");
		assert_eq!(clamp_price(0.0012345, 0, true), "0.0012345");

		// USDC price (should be close to 1.0)
		assert_eq!(clamp_price(1.0, 2, true), "1");
		assert_eq!(clamp_price(0.9999, 2, true), "0.9999");
	}
}
