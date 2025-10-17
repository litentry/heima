mod corewriter;
mod hypercore_api;
mod utils;

pub use corewriter::*;
pub use hypercore_api::*;
pub use utils::*;

use alloy::primitives::Address;

pub const CORE_WRITER_ADDRESS: &str = "0x3333333333333333333333333333333333333333";
pub const SIMPLE_PAYMASTER_ADDRESS: &str = "0x6255B9F4A4E80BC20eE389fD35DE9d2c029D5912"; // staging-v1

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
