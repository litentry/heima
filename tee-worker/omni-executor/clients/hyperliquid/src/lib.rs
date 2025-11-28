mod corewriter;
mod hypercore_api;
mod utils;

pub use corewriter::*;
pub use hypercore_api::*;
pub use utils::*;

use alloy::primitives::Address;

pub const CORE_WRITER_ADDRESS: &str = "0x3333333333333333333333333333333333333333";
pub const SIMPLE_PAYMASTER_ADDRESS: &str = "0x6255B9F4A4E80BC20eE389fD35DE9d2c029D5912"; // staging-v1

// Trading price adjustment ratios
/// Ratio for spot sell orders (2% below market to ensure fill)
pub const SPOT_SELL_PRICE_RATIO: f64 = 0.98;

/// Ratio for perp entry orders (currently at market price)
pub const PERP_ENTRY_PRICE_RATIO: f64 = 1.0;

/// Ratio for perp close orders (2% below market to ensure fill)
pub const PERP_CLOSE_PRICE_RATIO: f64 = 0.98;

/// Ratio for spot buy orders (2% above market to ensure fill)
pub const SPOT_BUY_PRICE_RATIO: f64 = 1.02;

/// Minimum notional value for perp and spot orders ($10 minimum)
pub const MIN_NOTIONAL_VALUE: f64 = 10.0;

// Unit conversion multipliers
/// Multiplier for converting decimal prices to HyperLiquid price units (8 decimals)
pub const PRICE_UNIT_MULTIPLIER: f64 = 100_000_000.0;

/// Multiplier for converting decimal USDC amounts to USDC units (6 decimals)
pub const USDC_UNIT_MULTIPLIER: f64 = 1_000_000.0;

/// Convert a decimal price value to HyperLiquid price units (8 decimals)
pub fn to_price_units(value: f64) -> u64 {
	(value * PRICE_UNIT_MULTIPLIER) as u64
}

/// Convert a decimal USDC amount to USDC units (6 decimals)
pub fn to_usdc_units(value: f64) -> u64 {
	(value * USDC_UNIT_MULTIPLIER) as u64
}

/// Extract bid (highest buy) and ask (lowest sell) prices from mark price and mid price.
///
/// # Arguments
/// * `mark_price` - The mark price from HyperLiquid API
/// * `mid_price` - The mid price from HyperLiquid API (if midPx was null, this will equal mark_price)
///
/// # Returns
/// * `(bid_price, ask_price)` - Tuple of (highest buy price, lowest sell price)
///
/// # Logic
/// - If mark_price > mid_price: mark is the ask (lowest sell), bid = mid * 2 - mark
/// - If mark_price < mid_price: mark is the bid (highest buy), ask = mid * 2 - mark
/// - If mark_price == mid_price: spread is zero or midPx was null, use mark for both
pub fn get_bid_ask_prices(mark_price: f64, mid_price: f64) -> (f64, f64) {
	const EPSILON: f64 = 0.00001;

	if (mark_price - mid_price).abs() < EPSILON {
		// mark_price == mid_price (with floating point tolerance)
		// This happens when spread is zero or when midPx was null (API returned markPx for both)
		(mark_price, mark_price)
	} else if mark_price > mid_price {
		// markPx is ask (lowest sell)
		let bid = mid_price * 2.0 - mark_price;
		(bid, mark_price)
	} else {
		// markPx is bid (highest buy)
		let ask = mid_price * 2.0 - mark_price;
		(mark_price, ask)
	}
}

/// Validate that the notional value (price * size) meets the minimum requirement
///
/// # Arguments
/// * `price` - The price of the asset
/// * `size` - The size/quantity (should be the clamped size)
/// * `operation` - Description of the operation for error messages (e.g., "spot sell", "perp open")
///
/// # Returns
/// * `Ok(notional)` - The calculated notional value if it meets the minimum
/// * `Err(String)` - Error message if notional is below minimum
pub fn validate_notional_value(price: f64, size: f64, operation: &str) -> Result<f64, String> {
	let notional = price * size;
	if notional < MIN_NOTIONAL_VALUE {
		Err(format!(
			"{} notional value ({:.2}) is below minimum required ({:.2})",
			operation, notional, MIN_NOTIONAL_VALUE
		))
	} else {
		Ok(notional)
	}
}

pub fn get_core_writer_address() -> Address {
	CORE_WRITER_ADDRESS.parse().unwrap()
}

pub fn get_simple_paymaster_address() -> Address {
	SIMPLE_PAYMASTER_ADDRESS.parse().unwrap()
}

/// Encode SimplePaymaster paymasterAndData
pub fn encode_simple_paymaster() -> String {
	SIMPLE_PAYMASTER_ADDRESS.to_string()
}

/// Pack account gas limits: verification_gas (128 bits) | call_gas (128 bits)
pub fn pack_account_gas_limits(verification_gas: u128, call_gas: u128) -> String {
	let mut bytes = [0u8; 32];
	bytes[0..16].copy_from_slice(&verification_gas.to_be_bytes());
	bytes[16..32].copy_from_slice(&call_gas.to_be_bytes());
	format!("0x{}", hex::encode(bytes))
}

/// Pack gas fees: max_fee_per_gas (128 bits) | max_priority_fee_per_gas (128 bits)
pub fn pack_gas_fees(max_fee_per_gas: u128, max_priority_fee_per_gas: u128) -> String {
	let mut bytes = [0u8; 32];
	bytes[0..16].copy_from_slice(&max_fee_per_gas.to_be_bytes());
	bytes[16..32].copy_from_slice(&max_priority_fee_per_gas.to_be_bytes());
	format!("0x{}", hex::encode(bytes))
}
