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

//! Autogenerated weights for `pallet_proxy`
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
// --pallet=pallet_proxy
// --extrinsic=*
// --heap-pages=4096
// --steps=20
// --repeat=50
// --header=./LICENSE_HEADER
// --output=./runtime/litentry/src/weights/pallet_proxy.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_proxy`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_proxy::WeightInfo for WeightInfo<T> {
	/// Storage: `Proxy::Proxies` (r:1 w:0)
	/// Proof: `Proxy::Proxies` (`max_values`: None, `max_size`: Some(1241), added: 3716, mode: `MaxEncodedLen`)
	/// The range of component `p` is `[1, 31]`.
	fn proxy(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `227 + p * (37 ±0)`
		//  Estimated: `4706`
		// Minimum execution time: 15_097_000 picoseconds.
		Weight::from_parts(15_861_496, 0)
			.saturating_add(Weight::from_parts(0, 4706))
			// Standard Error: 1_818
			.saturating_add(Weight::from_parts(49_217, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	/// Storage: `Proxy::Proxies` (r:1 w:0)
	/// Proof: `Proxy::Proxies` (`max_values`: None, `max_size`: Some(1241), added: 3716, mode: `MaxEncodedLen`)
	/// Storage: `Proxy::Announcements` (r:1 w:1)
	/// Proof: `Proxy::Announcements` (`max_values`: None, `max_size`: Some(2233), added: 4708, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// The range of component `a` is `[0, 31]`.
	/// The range of component `p` is `[1, 31]`.
	fn proxy_announced(a: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `517 + a * (68 ±0) + p * (37 ±0)`
		//  Estimated: `5698`
		// Minimum execution time: 36_845_000 picoseconds.
		Weight::from_parts(39_683_541, 0)
			.saturating_add(Weight::from_parts(0, 5698))
			// Standard Error: 10_132
			.saturating_add(Weight::from_parts(170_647, 0).saturating_mul(a.into()))
			// Standard Error: 10_476
			.saturating_add(Weight::from_parts(28_423, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Proxy::Announcements` (r:1 w:1)
	/// Proof: `Proxy::Announcements` (`max_values`: None, `max_size`: Some(2233), added: 4708, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// The range of component `a` is `[0, 31]`.
	/// The range of component `p` is `[1, 31]`.
	fn remove_announcement(a: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `432 + a * (68 ±0)`
		//  Estimated: `5698`
		// Minimum execution time: 23_501_000 picoseconds.
		Weight::from_parts(23_737_536, 0)
			.saturating_add(Weight::from_parts(0, 5698))
			// Standard Error: 6_140
			.saturating_add(Weight::from_parts(200_975, 0).saturating_mul(a.into()))
			// Standard Error: 6_348
			.saturating_add(Weight::from_parts(44_993, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Proxy::Announcements` (r:1 w:1)
	/// Proof: `Proxy::Announcements` (`max_values`: None, `max_size`: Some(2233), added: 4708, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// The range of component `a` is `[0, 31]`.
	/// The range of component `p` is `[1, 31]`.
	fn reject_announcement(a: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `432 + a * (68 ±0)`
		//  Estimated: `5698`
		// Minimum execution time: 23_434_000 picoseconds.
		Weight::from_parts(23_965_881, 0)
			.saturating_add(Weight::from_parts(0, 5698))
			// Standard Error: 6_336
			.saturating_add(Weight::from_parts(202_268, 0).saturating_mul(a.into()))
			// Standard Error: 6_551
			.saturating_add(Weight::from_parts(34_404, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Proxy::Proxies` (r:1 w:0)
	/// Proof: `Proxy::Proxies` (`max_values`: None, `max_size`: Some(1241), added: 3716, mode: `MaxEncodedLen`)
	/// Storage: `Proxy::Announcements` (r:1 w:1)
	/// Proof: `Proxy::Announcements` (`max_values`: None, `max_size`: Some(2233), added: 4708, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// The range of component `a` is `[0, 31]`.
	/// The range of component `p` is `[1, 31]`.
	fn announce(a: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `449 + a * (68 ±0) + p * (37 ±0)`
		//  Estimated: `5698`
		// Minimum execution time: 32_192_000 picoseconds.
		Weight::from_parts(33_653_504, 0)
			.saturating_add(Weight::from_parts(0, 5698))
			// Standard Error: 10_224
			.saturating_add(Weight::from_parts(167_400, 0).saturating_mul(a.into()))
			// Standard Error: 10_571
			.saturating_add(Weight::from_parts(69_985, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Proxy::Proxies` (r:1 w:1)
	/// Proof: `Proxy::Proxies` (`max_values`: None, `max_size`: Some(1241), added: 3716, mode: `MaxEncodedLen`)
	/// The range of component `p` is `[1, 31]`.
	fn add_proxy(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `227 + p * (37 ±0)`
		//  Estimated: `4706`
		// Minimum execution time: 24_032_000 picoseconds.
		Weight::from_parts(25_505_663, 0)
			.saturating_add(Weight::from_parts(0, 4706))
			// Standard Error: 7_223
			.saturating_add(Weight::from_parts(44_816, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Proxy::Proxies` (r:1 w:1)
	/// Proof: `Proxy::Proxies` (`max_values`: None, `max_size`: Some(1241), added: 3716, mode: `MaxEncodedLen`)
	/// The range of component `p` is `[1, 31]`.
	fn remove_proxy(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `227 + p * (37 ±0)`
		//  Estimated: `4706`
		// Minimum execution time: 24_059_000 picoseconds.
		Weight::from_parts(25_610_666, 0)
			.saturating_add(Weight::from_parts(0, 4706))
			// Standard Error: 9_278
			.saturating_add(Weight::from_parts(70_157, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Proxy::Proxies` (r:1 w:1)
	/// Proof: `Proxy::Proxies` (`max_values`: None, `max_size`: Some(1241), added: 3716, mode: `MaxEncodedLen`)
	/// The range of component `p` is `[1, 31]`.
	fn remove_proxies(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `227 + p * (37 ±0)`
		//  Estimated: `4706`
		// Minimum execution time: 21_248_000 picoseconds.
		Weight::from_parts(22_160_942, 0)
			.saturating_add(Weight::from_parts(0, 4706))
			// Standard Error: 7_720
			.saturating_add(Weight::from_parts(79_386, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Proxy::Proxies` (r:1 w:1)
	/// Proof: `Proxy::Proxies` (`max_values`: None, `max_size`: Some(1241), added: 3716, mode: `MaxEncodedLen`)
	/// The range of component `p` is `[1, 31]`.
	fn create_pure(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `239`
		//  Estimated: `4706`
		// Minimum execution time: 25_880_000 picoseconds.
		Weight::from_parts(26_821_197, 0)
			.saturating_add(Weight::from_parts(0, 4706))
			// Standard Error: 4_642
			.saturating_add(Weight::from_parts(49_058, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Proxy::Proxies` (r:1 w:1)
	/// Proof: `Proxy::Proxies` (`max_values`: None, `max_size`: Some(1241), added: 3716, mode: `MaxEncodedLen`)
	/// The range of component `p` is `[0, 30]`.
	fn kill_pure(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `264 + p * (37 ±0)`
		//  Estimated: `4706`
		// Minimum execution time: 22_343_000 picoseconds.
		Weight::from_parts(23_117_657, 0)
			.saturating_add(Weight::from_parts(0, 4706))
			// Standard Error: 2_365
			.saturating_add(Weight::from_parts(52_581, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
