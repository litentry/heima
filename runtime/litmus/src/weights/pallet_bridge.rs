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

//! Autogenerated weights for `pallet_bridge`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-10-04, STEPS: `20`, REPEAT: 50, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! HOSTNAME: `parachain-benchmark`, CPU: `Intel(R) Xeon(R) Platinum 8275CL CPU @ 3.00GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("litmus-dev"), DB CACHE: 20

// Executed Command:
// ./litentry-collator
// benchmark
// pallet
// --chain=litmus-dev
// --execution=wasm
// --db-cache=20
// --wasm-execution=compiled
// --pallet=pallet_bridge
// --extrinsic=*
// --heap-pages=4096
// --steps=20
// --repeat=50
// --header=./LICENSE_HEADER
// --output=./runtime/litmus/src/weights/pallet_bridge.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_bridge`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bridge::WeightInfo for WeightInfo<T> {
	// Storage: ChainBridge RelayerThreshold (r:0 w:1)
	/// The range of component `b` is `[1, 100]`.
	fn set_threshold(_b: u32, ) -> Weight {
		(12_074_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ChainBridge Resources (r:0 w:1)
	fn set_resource() -> Weight {
		(4_549_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ChainBridge Resources (r:0 w:1)
	fn remove_resource() -> Weight {
		(4_110_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ChainBridge ChainNonces (r:1 w:1)
	fn whitelist_chain() -> Weight {
		(14_198_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ChainBridge Relayers (r:1 w:1)
	// Storage: ChainBridge RelayerCount (r:1 w:1)
	fn add_relayer() -> Weight {
		(17_556_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: ChainBridge Relayers (r:1 w:1)
	// Storage: ChainBridge RelayerCount (r:1 w:1)
	fn remove_relayer() -> Weight {
		(19_385_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: ChainBridge BridgeFee (r:0 w:1)
	fn update_fee() -> Weight {
		(12_472_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ChainBridge Relayers (r:1 w:0)
	// Storage: ChainBridge ChainNonces (r:1 w:0)
	// Storage: ChainBridge Resources (r:1 w:0)
	// Storage: ChainBridge Votes (r:1 w:1)
	// Storage: ChainBridge RelayerThreshold (r:1 w:0)
	// Storage: ChainBridge RelayerCount (r:1 w:0)
	fn acknowledge_proposal() -> Weight {
		(46_067_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ChainBridge Relayers (r:1 w:0)
	// Storage: ChainBridge ChainNonces (r:1 w:0)
	// Storage: ChainBridge Resources (r:1 w:0)
	// Storage: ChainBridge Votes (r:1 w:1)
	// Storage: ChainBridge RelayerThreshold (r:1 w:0)
	// Storage: ChainBridge RelayerCount (r:1 w:0)
	fn reject_proposal() -> Weight {
		(39_949_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ChainBridge Votes (r:1 w:1)
	// Storage: ChainBridge RelayerThreshold (r:1 w:0)
	// Storage: ChainBridge RelayerCount (r:1 w:0)
	fn eval_vote_state() -> Weight {
		(16_176_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
