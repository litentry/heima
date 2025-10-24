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
	let encoded_tif: u8 = 2; // Gtc

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

pub fn build_usd_class_transfer_to_spot(ntl: u64) -> Vec<u8> {
	encode_usd_class_transfer_action(ntl, false)
}

pub fn build_perp_close_order(asset_id: u32, size: u64, price: u64, cloid: u128) -> Vec<u8> {
	// Close position by selling (is_buy = false) with reduce_only = true
	// Note: For long positions, we need to sell to close
	// The size should match the position size we want to close
	encode_limit_order_action_with_reduce_only(asset_id, false, price, size, cloid, true)
}

pub fn build_spot_buy_order(asset_id: u32, size: u64, price: u64, cloid: u128) -> Vec<u8> {
	let is_buy = true;
	encode_limit_order_action(asset_id, is_buy, price, size, cloid)
}

pub fn build_cancel_order_by_cloid(asset_id: u32, cloid: u128) -> Vec<u8> {
	let encoded =
		ethabi::encode(&[ethabi::Token::Uint(asset_id.into()), ethabi::Token::Uint(cloid.into())]);

	let mut data = Vec::new();
	data.push(0x01); // version
	data.extend_from_slice(&[0x00, 0x00, 0x02]); // action_id = 2 (cancel)
	data.extend_from_slice(&encoded);
	data
}

fn encode_limit_order_action_with_reduce_only(
	asset: u32,
	is_buy: bool,
	limit_px: u64,
	sz: u64,
	cloid: u128,
	reduce_only: bool,
) -> Vec<u8> {
	let encoded_tif: u8 = 2; // Gtc

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
