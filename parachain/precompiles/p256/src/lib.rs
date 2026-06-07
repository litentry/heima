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

//! RIP-7212 secp256r1 (P-256) signature verification precompile.
//!
//! Registered at the standard address `0x100` (256). Flat gas cost: 3450.
//!
//! Input layout (exactly 160 bytes):
//! ```text
//!   [  0.. 32) message hash
//!   [ 32.. 64) signature r
//!   [ 64.. 96) signature s
//!   [ 96..128) public key x
//!   [128..160) public key y
//! ```
//!
//! Output:
//! - valid signature: 32 bytes equal to `uint256(1)` (31 zero bytes, last byte `0x01`)
//! - invalid signature OR malformed input: empty output (`Vec::new()`)
//!
//! In ALL cases the call returns `ExitSucceed::Returned` (it never reverts/errors).
//! This matches the RIP-7212 spec and the on-chain interface of the deployed
//! daimo `P256Verifier` Solidity contract, and deliberately differs from
//! Frontier's `ed25519` precompile (which errors on malformed input).
//!
//! Note: RIP-7212 follows FIPS-186-5 and does NOT require canonical low-`s`
//! signatures. `verify_prehash` does not enforce low-`s`, which is correct —
//! do not add a malleability check.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{vec, vec::Vec};
use fp_evm::{ExitSucceed, Precompile, PrecompileHandle, PrecompileOutput, PrecompileResult};
use p256::ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey};

/// Flat gas cost defined by RIP-7212.
pub const P256VERIFY_GAS_COST: u64 = 3450;

/// Length of the RIP-7212 precompile input, in bytes.
const INPUT_LENGTH: usize = 160;

/// secp256r1 (P-256) signature verification precompile, RIP-7212.
pub struct P256Verify;

impl P256Verify {
	/// Returns `true` iff `input` is exactly 160 bytes, the public key and
	/// signature decode, and the signature verifies against the prehashed
	/// message. Any failure (bad length, malformed key/sig, bad signature)
	/// returns `false`.
	pub fn verify(input: &[u8]) -> bool {
		if input.len() != INPUT_LENGTH {
			return false;
		}

		let hash = &input[0..32];
		let r = &input[32..64];
		let s = &input[64..96];
		let x = &input[96..128];
		let y = &input[128..160];

		// SEC1 uncompressed encoding: 0x04 || x || y (65 bytes).
		let mut sec1 = [0u8; 65];
		sec1[0] = 0x04;
		sec1[1..33].copy_from_slice(x);
		sec1[33..65].copy_from_slice(y);

		let verifying_key = match VerifyingKey::from_sec1_bytes(&sec1) {
			Ok(key) => key,
			Err(_) => return false,
		};

		// Fixed-size r || s signature (64 bytes).
		let mut rs = [0u8; 64];
		rs[0..32].copy_from_slice(r);
		rs[32..64].copy_from_slice(s);

		let signature = match Signature::from_slice(&rs) {
			Ok(sig) => sig,
			Err(_) => return false,
		};

		verifying_key.verify_prehash(hash, &signature).is_ok()
	}
}

impl Precompile for P256Verify {
	fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
		// Flat cost, recorded unconditionally up front — exactly like the
		// ethereum-spec precompiles. Insufficient gas => record_cost returns
		// Err(OutOfGas) and the call reverts (the only failure path).
		handle.record_cost(P256VERIFY_GAS_COST)?;

		let output = if Self::verify(handle.input()) {
			// uint256(1): 31 zero bytes followed by 0x01.
			let mut buf = vec![0u8; 32];
			buf[31] = 1u8;
			buf
		} else {
			// Malformed input or invalid signature: empty output, still success.
			Vec::new()
		};

		Ok(PrecompileOutput { exit_status: ExitSucceed::Returned, output })
	}
}

#[cfg(test)]
mod tests;
