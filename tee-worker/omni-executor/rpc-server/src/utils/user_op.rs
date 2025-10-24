// Copyright 2020-2024 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::utils::hex::decode_hex;

/// Pack verification and call gas limits into a single 32-byte value
/// Format: verification_gas (16 bytes) | call_gas (16 bytes)
pub fn pack_account_gas_limits(verification_gas: u128, call_gas: u128) -> FixedBytes<32> {
	let packed: U256 = (U256::from(verification_gas) << 128) | U256::from(call_gas);
	FixedBytes::from(packed.to_be_bytes())
}

#[cfg(test)]
pub fn unpack_verification_gas_limit(packed: FixedBytes<32>) -> u128 {
	let value = U256::from_be_bytes(packed.0);
	let result: U256 = (value >> 128) & U256::from(u128::MAX);
	result.to::<u128>()
}

#[cfg(test)]
pub fn unpack_call_gas_limit(packed: FixedBytes<32>) -> u128 {
	let value = U256::from_be_bytes(packed.0);
	let result: U256 = value & U256::from(u128::MAX);
	result.to::<u128>()
}

/// Convert Substrate signature to Ethereum ECDSA format
/// Returns signature in format: [r (32 bytes), s (32 bytes), v (1 byte)]
pub fn substrate_to_ethereum_signature(substrate_sig: &[u8]) -> Result<[u8; 65], &'static str> {
	if substrate_sig.len() != 65 {
		return Err("Invalid signature length");
	}

	// Parse as (r, s, v) format - most common
	let mut r = [0u8; 32];
	let mut s = [0u8; 32];
	r.copy_from_slice(&substrate_sig[0..32]);
	s.copy_from_slice(&substrate_sig[32..64]);
	let substrate_v = substrate_sig[64];

	// Convert recovery parameter: 0/1 -> 27/28
	let ethereum_v = match substrate_v {
		0 => 27,
		1 => 28,
		27 => 27, // Already Ethereum format
		28 => 28, // Already Ethereum format
		_ => return Err("Invalid recovery parameter"),
	};

	// Build Ethereum signature: [r, s, v]
	let mut ethereum_sig = [0u8; 65];
	ethereum_sig[0..32].copy_from_slice(&r);
	ethereum_sig[32..64].copy_from_slice(&s);
	ethereum_sig[64] = ethereum_v;

	Ok(ethereum_sig)
}

/// Convert SerializablePackedUserOperation to aa_contracts_client::PackedUserOperation
pub fn convert_to_packed_user_op(
	user_op: SerializablePackedUserOperation,
) -> Result<aa_contracts_client::PackedUserOperation, String> {
	use std::str::FromStr;

	// Helper function to parse hex string to fixed bytes
	let parse_hex_fixed =
		|hex_str: &str, expected_len: usize, name: &str| -> Result<Vec<u8>, String> {
			let bytes = decode_hex(hex_str)
				.map_err(|e| format!("Invalid hex string '{}' '{}': {}", hex_str, name, e))?;
			if bytes.len() != expected_len {
				return Err(format!(
					"Expected {} bytes, got {} for '{}'",
					expected_len,
					bytes.len(),
					hex_str
				));
			}
			Ok(bytes)
		};

	Ok(aa_contracts_client::PackedUserOperation {
		sender: Address::from_str(&user_op.sender)
			.map_err(|e| format!("Invalid sender address '{}': {}", user_op.sender, e))?,
		nonce: U256::from(user_op.nonce),
		initCode: Bytes::from(
			decode_hex(&user_op.init_code).map_err(|e| format!("Invalid init_code hex: {}", e))?,
		),
		callData: Bytes::from(
			decode_hex(&user_op.call_data).map_err(|e| format!("Invalid call_data hex: {}", e))?,
		),
		accountGasLimits: {
			let bytes = parse_hex_fixed(&user_op.account_gas_limits, 32, "account_gas_limits")?;
			FixedBytes::from_slice(&bytes)
		},
		preVerificationGas: U256::from(user_op.pre_verification_gas),
		gasFees: {
			let bytes = parse_hex_fixed(&user_op.gas_fees, 32, "gas_fees")?;
			FixedBytes::from_slice(&bytes)
		},
		paymasterAndData: Bytes::from(
			decode_hex(&user_op.paymaster_and_data)
				.map_err(|e| format!("Invalid paymaster_and_data hex: {}", e))?,
		),
		signature: match user_op.signature {
			Some(sig) => {
				Bytes::from(decode_hex(&sig).map_err(|e| format!("Invalid signature hex: {}", e))?)
			},
			None => Bytes::new(), // Empty signature for unsigned operations
		},
	})
}
