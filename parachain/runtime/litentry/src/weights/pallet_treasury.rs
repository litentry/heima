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

//! Autogenerated weights for `pallet_treasury`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-08-28, STEPS: `20`, REPEAT: `50`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `litentry-benchmark-server`, CPU: `Intel(R) Xeon(R) CPU E5-2686 v4 @ 2.30GHz`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("litentry-dev")`, DB CACHE: 20

// Executed Command:
// ./heima-node
// benchmark
// pallet
// --chain=litentry-dev
// --execution=wasm
// --db-cache=20
// --wasm-execution=compiled
// --pallet=pallet_treasury
// --extrinsic=*
// --heap-pages=4096
// --steps=20
// --repeat=50
// --header=./LICENSE_HEADER
// --output=./runtime/litentry/src/weights/pallet_treasury.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_treasury`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_treasury::WeightInfo for WeightInfo<T> {
	/// Storage: `Treasury::ProposalCount` (r:1 w:1)
	/// Proof: `Treasury::ProposalCount` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Treasury::Approvals` (r:1 w:1)
	/// Proof: `Treasury::Approvals` (`max_values`: Some(1), `max_size`: Some(402), added: 897, mode: `MaxEncodedLen`)
	/// Storage: `Treasury::Proposals` (r:0 w:1)
	/// Proof: `Treasury::Proposals` (`max_values`: None, `max_size`: Some(108), added: 2583, mode: `MaxEncodedLen`)
	fn spend() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `80`
		//  Estimated: `1887`
		// Minimum execution time: 14_729_000 picoseconds.
		Weight::from_parts(15_324_000, 0)
			.saturating_add(Weight::from_parts(0, 1887))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `Treasury::ProposalCount` (r:1 w:1)
	/// Proof: `Treasury::ProposalCount` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Treasury::Proposals` (r:0 w:1)
	/// Proof: `Treasury::Proposals` (`max_values`: None, `max_size`: Some(108), added: 2583, mode: `MaxEncodedLen`)
	fn propose_spend() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `181`
		//  Estimated: `1489`
		// Minimum execution time: 26_716_000 picoseconds.
		Weight::from_parts(27_908_000, 0)
			.saturating_add(Weight::from_parts(0, 1489))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Treasury::Proposals` (r:1 w:1)
	/// Proof: `Treasury::Proposals` (`max_values`: None, `max_size`: Some(108), added: 2583, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	fn reject_proposal() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `442`
		//  Estimated: `6196`
		// Minimum execution time: 41_735_000 picoseconds.
		Weight::from_parts(43_198_000, 0)
			.saturating_add(Weight::from_parts(0, 6196))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `Treasury::Proposals` (r:1 w:0)
	/// Proof: `Treasury::Proposals` (`max_values`: None, `max_size`: Some(108), added: 2583, mode: `MaxEncodedLen`)
	/// Storage: `Treasury::Approvals` (r:1 w:1)
	/// Proof: `Treasury::Approvals` (`max_values`: Some(1), `max_size`: Some(402), added: 897, mode: `MaxEncodedLen`)
	/// The range of component `p` is `[0, 99]`.
	fn approve_proposal(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `486 + p * (9 ±0)`
		//  Estimated: `3573`
		// Minimum execution time: 9_526_000 picoseconds.
		Weight::from_parts(12_519_784, 0)
			.saturating_add(Weight::from_parts(0, 3573))
			// Standard Error: 1_869
			.saturating_add(Weight::from_parts(110_843, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Treasury::Approvals` (r:1 w:1)
	/// Proof: `Treasury::Approvals` (`max_values`: Some(1), `max_size`: Some(402), added: 897, mode: `MaxEncodedLen`)
	fn remove_approval() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `165`
		//  Estimated: `1887`
		// Minimum execution time: 7_496_000 picoseconds.
		Weight::from_parts(7_835_000, 0)
			.saturating_add(Weight::from_parts(0, 1887))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `System::Account` (r:201 w:201)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `Treasury::Deactivated` (r:1 w:1)
	/// Proof: `Treasury::Deactivated` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
	/// Storage: `Treasury::Approvals` (r:1 w:1)
	/// Proof: `Treasury::Approvals` (`max_values`: Some(1), `max_size`: Some(402), added: 897, mode: `MaxEncodedLen`)
	/// Storage: `Treasury::Proposals` (r:100 w:100)
	/// Proof: `Treasury::Proposals` (`max_values`: None, `max_size`: Some(108), added: 2583, mode: `MaxEncodedLen`)
	/// Storage: `Bounties::BountyApprovals` (r:1 w:1)
	/// Proof: `Bounties::BountyApprovals` (`max_values`: Some(1), `max_size`: Some(402), added: 897, mode: `MaxEncodedLen`)
	/// The range of component `p` is `[0, 100]`.
	fn on_initialize_proposals(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `149 + p * (252 ±0)`
		//  Estimated: `3593 + p * (5206 ±0)`
		// Minimum execution time: 31_378_000 picoseconds.
		Weight::from_parts(32_079_000, 0)
			.saturating_add(Weight::from_parts(0, 3593))
			// Standard Error: 217_012
			.saturating_add(Weight::from_parts(52_183_221, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().reads((3_u64).saturating_mul(p.into())))
			.saturating_add(T::DbWeight::get().writes(4))
			.saturating_add(T::DbWeight::get().writes((3_u64).saturating_mul(p.into())))
			.saturating_add(Weight::from_parts(0, 5206).saturating_mul(p.into()))
	}
}
