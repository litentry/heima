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

//! Autogenerated weights for `pallet_session`
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
// --pallet=pallet_session
// --extrinsic=*
// --heap-pages=4096
// --steps=20
// --repeat=50
// --header=./LICENSE_HEADER
// --output=./runtime/litmus/src/weights/pallet_session.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_session`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_session::WeightInfo for WeightInfo<T> {
	/// Storage: Session NextKeys (r:1 w:1)
	/// Proof Skipped: Session NextKeys (max_values: None, max_size: None, mode: Measured)
	/// Storage: Session KeyOwner (r:1 w:1)
	/// Proof Skipped: Session KeyOwner (max_values: None, max_size: None, mode: Measured)
	fn set_keys() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `270`
		//  Estimated: `3735`
		// Minimum execution time: 21_669_000 picoseconds.
		Weight::from_parts(22_528_000, 0)
			.saturating_add(Weight::from_parts(0, 3735))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: Session NextKeys (r:1 w:1)
	/// Proof Skipped: Session NextKeys (max_values: None, max_size: None, mode: Measured)
	/// Storage: Session KeyOwner (r:0 w:1)
	/// Proof Skipped: Session KeyOwner (max_values: None, max_size: None, mode: Measured)
	fn purge_keys() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `242`
		//  Estimated: `3707`
		// Minimum execution time: 15_478_000 picoseconds.
		Weight::from_parts(15_906_000, 0)
			.saturating_add(Weight::from_parts(0, 3707))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(2))
	}
}
