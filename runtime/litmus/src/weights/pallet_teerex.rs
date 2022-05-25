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

//! Autogenerated weights for `pallet_teerex`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-05-09, STEPS: `20`, REPEAT: 50, LOW RANGE: `[]`, HIGH RANGE: `[]`
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

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_teerex`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_teerex::WeightInfo for WeightInfo<T> {
	// Storage: Timestamp Now (r:1 w:0)
	// Storage: Teerex AllowSGXDebugMode (r:1 w:0)
	// Storage: Teerex EnclaveIndex (r:1 w:0)
	// Storage: Teerex EnclaveRegistry (r:0 w:1)
	fn register_enclave() -> Weight {
		(2_021_763_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Teerex EnclaveIndex (r:1 w:2)
	// Storage: Teerex EnclaveCount (r:1 w:1)
	// Storage: Teerex EnclaveRegistry (r:1 w:2)
	fn unregister_enclave() -> Weight {
		(30_660_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	fn call_worker() -> Weight {
		(13_308_000 as Weight)
	}
	// Storage: Teerex EnclaveIndex (r:1 w:0)
	fn confirm_processed_parentchain_block() -> Weight {
		(14_909_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
	}
	// Storage: Teerex EnclaveIndex (r:1 w:0)
	// Storage: Teerex EnclaveRegistry (r:1 w:0)
	// Storage: Teerex WorkerForShard (r:0 w:1)
	fn confirm_proposed_sidechain_block() -> Weight {
		(22_591_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
