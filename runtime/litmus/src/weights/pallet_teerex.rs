// Copyright 2020-2023 Trust Computing GmbH.
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

//! Autogenerated weights for `pallet_teerex`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-10-17, STEPS: `20`, REPEAT: `50`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `parachain-benchmark`, CPU: `Intel(R) Xeon(R) Platinum 8259CL CPU @ 2.50GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("litmus-dev"), DB CACHE: 20

// Executed Command:
// ./litentry-collator
// benchmark
// pallet
// --chain=litmus-dev
// --execution=wasm
// --db-cache=20
// --wasm-execution=compiled
// --pallet=pallet_teerex
// --extrinsic=*
// --heap-pages=4096
// --steps=20
// --repeat=50
// --header=./LICENSE_HEADER
// --output=./runtime/litmus/src/weights/pallet_teerex.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_teerex`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_teerex::WeightInfo for WeightInfo<T> {
	/// Storage: Timestamp Now (r:1 w:0)
	/// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	/// Storage: Teerex AllowSGXDebugMode (r:1 w:0)
	/// Proof Skipped: Teerex AllowSGXDebugMode (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Teerex ScheduledEnclave (r:2 w:0)
	/// Proof Skipped: Teerex ScheduledEnclave (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teerex EnclaveIndex (r:1 w:0)
	/// Proof Skipped: Teerex EnclaveIndex (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teerex EnclaveRegistry (r:0 w:1)
	/// Proof Skipped: Teerex EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	fn register_enclave() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `451`
		//  Estimated: `6391`
		// Minimum execution time: 2_135_064_000 picoseconds.
		Weight::from_parts(2_253_162_000, 0)
			.saturating_add(Weight::from_parts(0, 6391))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Teerex EnclaveIndex (r:1 w:2)
	/// Proof Skipped: Teerex EnclaveIndex (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teerex EnclaveCount (r:1 w:1)
	/// Proof Skipped: Teerex EnclaveCount (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Teerex EnclaveRegistry (r:1 w:2)
	/// Proof Skipped: Teerex EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	fn unregister_enclave() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `477`
		//  Estimated: `3942`
		// Minimum execution time: 40_903_000 picoseconds.
		Weight::from_parts(41_801_000, 0)
			.saturating_add(Weight::from_parts(0, 3942))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(5))
	}
	fn invoke() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 14_997_000 picoseconds.
		Weight::from_parts(15_528_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// Storage: Teerex EnclaveIndex (r:1 w:0)
	/// Proof Skipped: Teerex EnclaveIndex (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teerex EnclaveRegistry (r:1 w:1)
	/// Proof Skipped: Teerex EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: Timestamp Now (r:1 w:0)
	/// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	fn confirm_processed_parentchain_block() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `377`
		//  Estimated: `3842`
		// Minimum execution time: 37_869_000 picoseconds.
		Weight::from_parts(39_091_000, 0)
			.saturating_add(Weight::from_parts(0, 3842))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: Teerex EnclaveIndex (r:1 w:0)
	/// Proof Skipped: Teerex EnclaveIndex (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teerex EnclaveRegistry (r:1 w:0)
	/// Proof Skipped: Teerex EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: System EventTopics (r:6 w:6)
	/// Proof Skipped: System EventTopics (max_values: None, max_size: None, mode: Measured)
	/// The range of component `l` is `[0, 100]`.
	/// The range of component `t` is `[1, 5]`.
	fn publish_hash(l: u32, t: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `335`
		//  Estimated: `3800 + t * (2475 ±0)`
		// Minimum execution time: 32_844_000 picoseconds.
		Weight::from_parts(30_741_266, 0)
			.saturating_add(Weight::from_parts(0, 3800))
			// Standard Error: 1_584
			.saturating_add(Weight::from_parts(11_359, 0).saturating_mul(l.into()))
			// Standard Error: 35_361
			.saturating_add(Weight::from_parts(2_647_005, 0).saturating_mul(t.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(t.into())))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(t.into())))
			.saturating_add(Weight::from_parts(0, 2475).saturating_mul(t.into()))
	}
}
