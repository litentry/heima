use alloy::primitives::Address;
use executor_primitives::utils::hex::hex_encode;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn generate_cloid() -> u128 {
	SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}

pub fn encode_limit_order_action(
	asset: u32,
	is_buy: bool,
	limit_px: u64,
	sz: u64,
	cloid: u128,
) -> Vec<u8> {
	let reduce_only = false;
	let encoded_tif: u8 = 3; // Ioc

	let encoded = ethabi::encode(&[
		ethabi::Token::Uint(asset.into()),
		ethabi::Token::Bool(is_buy),
		ethabi::Token::Uint(limit_px.into()),
		ethabi::Token::Uint(sz.into()),
		ethabi::Token::Bool(reduce_only),
		ethabi::Token::Uint(encoded_tif.into()),
		ethabi::Token::Uint(cloid.into()),
	]);

	let mut data = Vec::new();
	data.push(0x01); // version
	data.extend_from_slice(&[0x00, 0x00, 0x01]); // action_id = 1
	data.extend_from_slice(&encoded);
	data
}

pub fn encode_omni_account_execute(target: Address, call_data: Vec<u8>) -> String {
	const EXECUTE_SELECTOR: [u8; 4] = [0xb6, 0x1d, 0x27, 0xf6];

	let target_h160 = ethabi::ethereum_types::H160::from_slice(target.as_slice());

	let encoded = ethabi::encode(&[
		ethabi::Token::Address(target_h160),
		ethabi::Token::Uint(0.into()),
		ethabi::Token::Bytes(call_data),
	]);

	let mut result = Vec::new();
	result.extend_from_slice(&EXECUTE_SELECTOR);
	result.extend_from_slice(&encoded);

	hex_encode(&result)
}

pub fn encode_send_raw_action(action_data: Vec<u8>) -> Vec<u8> {
	const SEND_RAW_ACTION_SELECTOR: [u8; 4] = [0x17, 0x93, 0x8e, 0x13];

	let encoded = ethabi::encode(&[ethabi::Token::Bytes(action_data)]);

	let mut result = Vec::new();
	result.extend_from_slice(&SEND_RAW_ACTION_SELECTOR);
	result.extend_from_slice(&encoded);

	result
}

/// Calculate spot order size for CoreWriter action
///
/// Formula: size = human_readable_amount * 10^8
/// As per HyperLiquid docs: "limitPx and sz should be sent as 10^8 * the human readable value"
///
/// # Arguments
/// * `amount` - Human-readable amount (e.g., 100.5 PURR)
pub fn calculate_corewriter_size(amount: f64) -> u64 {
	(amount * 100_000_000.0) as u64
}

/// Calculate order price for CoreWriter action
///
/// Formula: price = usdc_price * 10^8
/// As per HyperLiquid docs: "limitPx and sz should be sent as 10^8 * the human readable value"
///
/// # Arguments
/// * `usdc_price` - Price in USDC (e.g., 3000.5 USDC per token)
pub fn calculate_corewriter_price(usdc_price: f64) -> u64 {
	(usdc_price * 100_000_000.0) as u64
}

/// Calculate perp order size for CoreWriter action
///
/// Formula:
/// 1. Calculate effective leverage = min(1 / (1 - lending_ratio), max_leverage)
/// 2. Calculate notional = margin * leverage
/// 3. Calculate size = notional / market_price
/// 4. Convert to CoreWriter units: size * 10^8
///
/// # Arguments
/// * `margin` - Margin amount in USDC
/// * `lending_ratio` - Lending ratio (0.0 to 1.0)
/// * `market_price` - Current market price in USDC
/// * `max_leverage` - Maximum leverage allowed for this asset
pub fn calculate_corewriter_perp_size(
	margin: f64,
	lending_ratio: f64,
	market_price: f64,
	max_leverage: u32,
) -> u64 {
	// Calculate desired leverage, capped at max
	let desired_leverage = 1.0 / (1.0 - lending_ratio);
	let effective_leverage = desired_leverage.min(max_leverage as f64);

	// Calculate notional value
	let notional = margin * effective_leverage;

	// Calculate size in human-readable units
	let size = notional / market_price;

	// Convert to CoreWriter units: 10^8
	(size * 100_000_000.0) as u64
}

/// Calculate spot order size for HyperCore API (used for USD transfers)
///
/// Formula: size = human_readable_amount * 10^weiDecimals
/// Then round down to respect minimum tradable increment: 10^(weiDecimals - szDecimals)
///
/// # Arguments
/// * `amount` - Human-readable amount (e.g., 100.5 PURR)
/// * `wei_decimals` - Token wei decimals from spot meta
/// * `sz_decimals` - Token size decimals from spot meta
pub fn calculate_spot_size(amount: f64, wei_decimals: u8, sz_decimals: u8) -> u64 {
	let multiplier = 10f64.powi(wei_decimals as i32);
	let size_raw = amount * multiplier;

	// Calculate minimum tradable increment
	let increment_decimals = wei_decimals.saturating_sub(sz_decimals);
	let min_increment = 10f64.powi(increment_decimals as i32);

	// Round down to nearest increment
	let size_rounded = (size_raw / min_increment).floor() * min_increment;
	size_rounded as u64
}

pub fn build_spot_sell_order(asset_id: u32, size: u64, price: u64, cloid: u128) -> Vec<u8> {
	let is_buy = false;
	encode_limit_order_action(asset_id, is_buy, price, size, cloid)
}

pub fn build_perp_long_order(asset_id: u32, size: u64, price: u64, cloid: u128) -> Vec<u8> {
	let is_buy = true;
	encode_limit_order_action(asset_id, is_buy, price, size, cloid)
}

pub fn encode_usd_class_transfer_action(ntl: u64, to_perp: bool) -> Vec<u8> {
	let encoded = ethabi::encode(&[ethabi::Token::Uint(ntl.into()), ethabi::Token::Bool(to_perp)]);

	let mut data = Vec::new();
	data.push(0x01); // version
	data.extend_from_slice(&[0x00, 0x00, 0x07]); // action_id = 7
	data.extend_from_slice(&encoded);

	data
}

pub fn build_usd_class_transfer_to_perp(ntl: u64) -> Vec<u8> {
	encode_usd_class_transfer_action(ntl, true)
}
