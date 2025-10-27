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
