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
	#[serde(rename = "orderStatus")]
	OrderStatus { user: String, oid: String },
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
pub struct PerpAsset {
	pub name: String,
	#[serde(rename = "szDecimals")]
	pub sz_decimals: u8,
	#[serde(rename = "maxLeverage")]
	pub max_leverage: u32,
}

#[derive(Debug, Deserialize)]
pub struct OrderStatusResponse {
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
	pub asset: u32,
	#[serde(rename = "isBuy")]
	pub is_buy: bool,
	#[serde(rename = "limitPx")]
	pub limit_px: String,
	pub sz: String,
	#[serde(rename = "reduceOnly")]
	pub reduce_only: bool,
	pub cloid: Option<String>,
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

	pub async fn get_order_status(
		&self,
		user_address: &str,
		cloid: &str,
	) -> Result<OrderStatusResponse, String> {
		let request = HyperCoreRequest::OrderStatus {
			user: user_address.to_string(),
			oid: cloid.to_string(),
		};

		debug!("Querying order status for cloid: {}", cloid);

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
