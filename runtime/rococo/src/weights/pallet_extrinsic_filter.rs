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

//! Autogenerated weights for `pallet_extrinsic_filter`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-10-17, STEPS: `20`, REPEAT: `50`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `parachain-benchmark`, CPU: `Intel(R) Xeon(R) Platinum 8259CL CPU @ 2.50GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("rococo-dev"), DB CACHE: 20

// Executed Command:
// ./litentry-collator
// benchmark
// pallet
// --chain=rococo-dev
// --execution=wasm
// --db-cache=20
// --wasm-execution=compiled
// --pallet=pallet_extrinsic_filter
// --extrinsic=*
// --heap-pages=4096
// --steps=20
// --repeat=50
// --header=./LICENSE_HEADER
// --output=./runtime/rococo/src/weights/pallet_extrinsic_filter.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_extrinsic_filter`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_extrinsic_filter::WeightInfo for WeightInfo<T> {
	/// Storage: ExtrinsicFilter BlockedExtrinsics (r:1 w:1)
	/// Proof Skipped: ExtrinsicFilter BlockedExtrinsics (max_values: None, max_size: None, mode: Measured)
	/// The range of component `p` is `[1, 1024]`.
	/// The range of component `f` is `[1, 1024]`.
	fn block_extrinsics(p: u32, f: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `142`
		//  Estimated: `3607`
		// Minimum execution time: 22_532_000 picoseconds.
		Weight::from_parts(21_407_222, 0)
			.saturating_add(Weight::from_parts(0, 3607))
			// Standard Error: 134
			.saturating_add(Weight::from_parts(2_594, 0).saturating_mul(p.into()))
			// Standard Error: 134
			.saturating_add(Weight::from_parts(2_945, 0).saturating_mul(f.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: ExtrinsicFilter BlockedExtrinsics (r:1 w:1)
	/// Proof Skipped: ExtrinsicFilter BlockedExtrinsics (max_values: None, max_size: None, mode: Measured)
	/// The range of component `p` is `[1, 1024]`.
	/// The range of component `f` is `[1, 1024]`.
	fn unblock_extrinsics(p: u32, f: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `178 + f * (1 ±0) + p * (1 ±0)`
		//  Estimated: `3644 + f * (1 ±0) + p * (1 ±0)`
		// Minimum execution time: 34_403_000 picoseconds.
		Weight::from_parts(23_998_401, 0)
			.saturating_add(Weight::from_parts(0, 3644))
			// Standard Error: 181
			.saturating_add(Weight::from_parts(13_204, 0).saturating_mul(p.into()))
			// Standard Error: 181
			.saturating_add(Weight::from_parts(12_287, 0).saturating_mul(f.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(Weight::from_parts(0, 1).saturating_mul(f.into()))
			.saturating_add(Weight::from_parts(0, 1).saturating_mul(p.into()))
	}
}
