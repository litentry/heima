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

use crate::{P256Verify, P256VERIFY_GAS_COST};
use hex_literal::hex;

// Canonical RIP-7212 / Wycheproof valid vector — the first valid case from
// daimo-eth/p256-verifier `test/P256Verifier.t.sol`:
//   hash = 0xbb5a52f42f9c9261ed4361f59422a1e30036e7c32b270c8807a419feca605023
//   r = 19738613187745101558623338726804762177711919211234071563652772152683725073944
//   s = 34753961278895633991577816754222591531863837041401341770838584739693604822390
//   x = 18614955573315897657680976650685450080931919913269223958732452353593824192568
//   y = 90223116347859880166570198725387569567414254547569925327988539833150573990206
// (r/s/x/y shown here as their 32-byte big-endian hex encodings.)
const VALID_INPUT: [u8; 160] = hex!(
	"bb5a52f42f9c9261ed4361f59422a1e30036e7c32b270c8807a419feca605023" // hash
	"2ba3a8be6b94d5ec80a6d9d1190a436effe50d85a1eee859b8cc6af9bd5c2e18" // r
	"4cd60b855d442f5b3c7b11eb6c4e0ae7525fe710fab9aa7c77a67f79e6fadd76" // s
	"2927b10512bae3eddcfe467828128bad2903269919f7086069c8c4df6c732838" // x
	"c7787964eaac00e5921fb1498a60f4606766b3d9685001558d1a974e7341513e" // y
);

#[test]
fn valid_signature_verifies() {
	assert!(P256Verify::verify(&VALID_INPUT));
}

#[test]
fn invalid_signature_fails() {
	// Flip one bit of r.
	let mut bad = VALID_INPUT;
	bad[32] ^= 0x01;
	assert!(!P256Verify::verify(&bad));

	// Flip one bit of the public-key x coordinate.
	let mut bad_key = VALID_INPUT;
	bad_key[96] ^= 0x01;
	assert!(!P256Verify::verify(&bad_key));
}

#[test]
fn wrong_length_fails() {
	for len in [0usize, 96, 159, 161, 320] {
		let buf = vec![0u8; len];
		assert!(!P256Verify::verify(&buf), "len {len} should be rejected");
	}
	// A 160-byte all-zero input is well-formed length-wise but not a valid sig.
	assert!(!P256Verify::verify(&[0u8; 160]));
}

#[test]
fn gas_cost_is_rip7212_flat() {
	assert_eq!(P256VERIFY_GAS_COST, 3450);
}

// Handle-level test: exercises the full `Precompile::execute` path (gas
// recording + output framing) via the precompile-utils mock handle.
mod handle {
	use crate::{P256Verify, P256VERIFY_GAS_COST};
	use fp_evm::{Context, ExitSucceed, Precompile};
	use precompile_utils::testing::MockHandle;
	use sp_core::{H160, U256};

	fn handle(input: Vec<u8>) -> MockHandle {
		let addr = H160::from_low_u64_be(256);
		let context = Context { address: addr, caller: H160::zero(), apparent_value: U256::zero() };
		let mut h = MockHandle::new(addr, context);
		h.input = input;
		h
	}

	#[test]
	fn execute_valid_returns_uint256_one() {
		let mut h = handle(super::VALID_INPUT.to_vec());
		let out = P256Verify::execute(&mut h).expect("must not revert on valid input");
		assert_eq!(out.exit_status, ExitSucceed::Returned);
		assert_eq!(out.output.len(), 32);
		assert_eq!(out.output[31], 1u8);
		assert!(out.output[..31].iter().all(|b| *b == 0));
		assert_eq!(h.gas_used, P256VERIFY_GAS_COST);
	}

	#[test]
	fn execute_invalid_returns_empty_success() {
		let mut bad = super::VALID_INPUT;
		bad[64] ^= 0x01; // flip a bit of s
		let mut h = handle(bad.to_vec());
		let out = P256Verify::execute(&mut h).expect("must not revert on invalid sig");
		assert_eq!(out.exit_status, ExitSucceed::Returned);
		assert!(out.output.is_empty());
	}

	#[test]
	fn execute_wrong_length_returns_empty_success() {
		let mut h = handle(vec![0u8; 159]);
		let out = P256Verify::execute(&mut h).expect("must not revert on bad length");
		assert_eq!(out.exit_status, ExitSucceed::Returned);
		assert!(out.output.is_empty());
	}
}
