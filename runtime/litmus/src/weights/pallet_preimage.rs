// Copyright 2020-2022 Litentry Technologies GmbH.
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

//! Autogenerated weights for `pallet_preimage`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-07-20, STEPS: `20`, REPEAT: 50, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `parachain-benchmark`, CPU: `Intel(R) Xeon(R) Platinum 8175M CPU @ 2.50GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("litmus-dev"), DB CACHE: 20

// Executed Command:
// ./litentry-collator
// benchmark
// pallet
// --chain=litmus-dev
// --execution=wasm
// --db-cache=20
// --wasm-execution=compiled
// --pallet=pallet_preimage
// --extrinsic=*
// --heap-pages=4096
// --steps=20
// --repeat=50
// --header=./LICENSE_HEADER
// --output=./runtime/litmus/src/weights/pallet_preimage.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_preimage`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_preimage::WeightInfo for WeightInfo<T> {
	// Storage: Preimage PreimageFor (r:1 w:1)
	// Storage: Preimage StatusFor (r:1 w:1)
	/// The range of component `s` is `[0, 4194304]`.
	fn note_preimage(s: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 0
			.saturating_add((3_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Preimage PreimageFor (r:1 w:1)
	// Storage: Preimage StatusFor (r:1 w:0)
	/// The range of component `s` is `[0, 4194304]`.
	fn note_requested_preimage(s: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 0
			.saturating_add((3_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Preimage PreimageFor (r:1 w:1)
	// Storage: Preimage StatusFor (r:1 w:0)
	/// The range of component `s` is `[0, 4194304]`.
	fn note_no_deposit_preimage(s: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 0
			.saturating_add((3_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Preimage StatusFor (r:1 w:1)
	// Storage: Preimage PreimageFor (r:0 w:1)
	fn unnote_preimage() -> Weight {
		(82_437_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Preimage StatusFor (r:1 w:1)
	// Storage: Preimage PreimageFor (r:0 w:1)
	fn unnote_no_deposit_preimage() -> Weight {
		(59_854_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Preimage StatusFor (r:1 w:1)
	fn request_preimage() -> Weight {
		(78_614_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Preimage StatusFor (r:1 w:1)
	fn request_no_deposit_preimage() -> Weight {
		(58_045_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Preimage StatusFor (r:1 w:1)
	fn request_unnoted_preimage() -> Weight {
		(36_576_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Preimage StatusFor (r:1 w:1)
	fn request_requested_preimage() -> Weight {
		(18_406_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Preimage StatusFor (r:1 w:1)
	// Storage: Preimage PreimageFor (r:0 w:1)
	fn unrequest_preimage() -> Weight {
		(58_567_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Preimage StatusFor (r:1 w:1)
	// Storage: Preimage PreimageFor (r:0 w:1)
	fn unrequest_unnoted_preimage() -> Weight {
		(38_770_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Preimage StatusFor (r:1 w:1)
	fn unrequest_multi_referenced_preimage() -> Weight {
		(18_937_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
