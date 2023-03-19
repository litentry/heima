// Copyright 2020-2023 Litentry Technologies GmbH.
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

//! Autogenerated weights for `pallet_bridge`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-03-15, STEPS: `20`, REPEAT: 50, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `parachain-benchmark`, CPU: `Intel(R) Xeon(R) Platinum 8259CL CPU @ 2.50GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("litentry-dev"), DB CACHE: 20

// Executed Command:
// ./litentry-collator
// benchmark
// pallet
// --chain=litentry-dev
// --execution=wasm
// --db-cache=20
// --wasm-execution=compiled
// --pallet=pallet_bridge
// --extrinsic=*
// --heap-pages=4096
// --steps=20
// --repeat=50
// --header=./LICENSE_HEADER
// --output=./runtime/litentry/src/weights/pallet_bridge.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_bridge`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bridge::WeightInfo for WeightInfo<T> {
	// Storage: ChainBridge RelayerThreshold (r:0 w:1)
	fn set_threshold() -> Weight {
		// Minimum execution time: 17_751 nanoseconds.
		Weight::from_ref_time(18_086_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: ChainBridge Resources (r:0 w:1)
	fn set_resource() -> Weight {
		// Minimum execution time: 7_293 nanoseconds.
		Weight::from_ref_time(7_449_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: ChainBridge Resources (r:0 w:1)
	fn remove_resource() -> Weight {
		// Minimum execution time: 6_427 nanoseconds.
		Weight::from_ref_time(6_737_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: ChainBridge ChainNonces (r:1 w:1)
	fn whitelist_chain() -> Weight {
		// Minimum execution time: 21_462 nanoseconds.
		Weight::from_ref_time(22_584_000)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: ChainBridge Relayers (r:1 w:1)
	// Storage: ChainBridge RelayerCount (r:1 w:1)
	fn add_relayer() -> Weight {
		// Minimum execution time: 24_143 nanoseconds.
		Weight::from_ref_time(25_208_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: ChainBridge Relayers (r:1 w:1)
	// Storage: ChainBridge RelayerCount (r:1 w:1)
	fn remove_relayer() -> Weight {
		// Minimum execution time: 27_452 nanoseconds.
		Weight::from_ref_time(28_157_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	// Storage: ChainBridge BridgeFee (r:0 w:1)
	fn update_fee() -> Weight {
		// Minimum execution time: 19_009 nanoseconds.
		Weight::from_ref_time(19_507_000)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: ChainBridge Relayers (r:1 w:0)
	// Storage: ChainBridge ChainNonces (r:1 w:0)
	// Storage: ChainBridge Resources (r:1 w:0)
	// Storage: ChainBridge Votes (r:1 w:1)
	// Storage: ChainBridge RelayerThreshold (r:1 w:0)
	// Storage: ChainBridge RelayerCount (r:1 w:0)
	fn acknowledge_proposal() -> Weight {
		// Minimum execution time: 54_376 nanoseconds.
		Weight::from_ref_time(56_680_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: ChainBridge Relayers (r:1 w:0)
	// Storage: ChainBridge ChainNonces (r:1 w:0)
	// Storage: ChainBridge Resources (r:1 w:0)
	// Storage: ChainBridge Votes (r:1 w:1)
	// Storage: ChainBridge RelayerThreshold (r:1 w:0)
	// Storage: ChainBridge RelayerCount (r:1 w:0)
	fn reject_proposal() -> Weight {
		// Minimum execution time: 47_287 nanoseconds.
		Weight::from_ref_time(49_526_000)
			.saturating_add(T::DbWeight::get().reads(6))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	// Storage: ChainBridge Votes (r:1 w:1)
	// Storage: ChainBridge RelayerThreshold (r:1 w:0)
	// Storage: ChainBridge RelayerCount (r:1 w:0)
	fn eval_vote_state() -> Weight {
		// Minimum execution time: 23_926 nanoseconds.
		Weight::from_ref_time(24_851_000)
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
