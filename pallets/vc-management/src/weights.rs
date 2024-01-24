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

//! Autogenerated weights for pallet_vc_management
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-06-22, STEPS: `20`, REPEAT: 50, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("rococo-dev"), DB CACHE: 20

// Executed Command:
// ./target/release/litentry-collator
// benchmark
// pallet
// --chain=rococo-dev
// --execution=wasm
// --db-cache=20
// --wasm-execution=compiled
// --pallet=pallet_vc_management
// --extrinsic=*
// --heap-pages=4096
// --steps=20
// --repeat=50
// --header=./LICENSE_HEADER
// --template=./templates/benchmark/pallet-weight-template.hbs
// --output=./pallets/vc-management/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_vc_management.
pub trait WeightInfo {
	fn add_delegatee() -> Weight;
	fn remove_delegatee() -> Weight;
	fn request_vc() -> Weight;
	fn vc_issued() -> Weight;
	fn some_error() -> Weight;
	fn set_admin() -> Weight;
	fn add_schema() -> Weight;
	fn disable_schema() -> Weight;
	fn activate_schema() -> Weight;
	fn revoke_schema() -> Weight;
}

/// Weights for pallet_vc_management using the Litentry node and recommended hardware.
pub struct LitentryWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for LitentryWeight<T> {
	// Storage: VCManagement Delegatee (r:0 w:1)
	// Proof: VCManagement Delegatee (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	fn add_delegatee() -> Weight {
		Weight::from_parts(12_840_000 as u64, 0)
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: VCManagement Delegatee (r:1 w:1)
	// Proof: VCManagement Delegatee (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	fn remove_delegatee() -> Weight {
		Weight::from_parts(20_988_000 as u64, 0)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: VCMPExtrinsicWhitelist GroupControlOn (r:1 w:0)
	// Proof Skipped: VCMPExtrinsicWhitelist GroupControlOn (max_values: Some(1), max_size: None, mode: Measured)
	fn request_vc() -> Weight {
		Weight::from_parts(35_640_000 as u64, 0)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
	}
	// Storage: Teerex EnclaveIndex (r:1 w:0)
	// Proof Skipped: Teerex EnclaveIndex (max_values: None, max_size: None, mode: Measured)
	// Storage: VCManagement VCRegistry (r:1 w:1)
	// Proof: VCManagement VCRegistry (max_values: None, max_size: Some(312), added: 2787, mode: MaxEncodedLen)
	fn vc_issued() -> Weight {
		Weight::from_parts(26_888_000 as u64, 0)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: Teerex EnclaveIndex (r:1 w:0)
	// Proof Skipped: Teerex EnclaveIndex (max_values: None, max_size: None, mode: Measured)
	fn some_error() -> Weight {
		Weight::from_parts(20_441_000 as u64, 0)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
	}
	// Storage: VCManagement Admin (r:1 w:1)
	// Proof: VCManagement Admin (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	fn set_admin() -> Weight {
		Weight::from_parts(18_220_000 as u64, 0)
			.saturating_add(T::DbWeight::get().reads(1 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: VCManagement Admin (r:1 w:0)
	// Proof: VCManagement Admin (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	// Storage: VCManagement SchemaRegistryIndex (r:1 w:1)
	// Proof: VCManagement SchemaRegistryIndex (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	// Storage: VCManagement SchemaRegistry (r:0 w:1)
	// Proof: VCManagement SchemaRegistry (max_values: None, max_size: Some(2621), added: 5096, mode: MaxEncodedLen)
	fn add_schema() -> Weight {
		Weight::from_parts(50_676_000 as u64, 0)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(2 as u64))
	}
	// Storage: VCManagement Admin (r:1 w:0)
	// Proof: VCManagement Admin (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	// Storage: VCManagement SchemaRegistry (r:1 w:1)
	// Proof: VCManagement SchemaRegistry (max_values: None, max_size: Some(2621), added: 5096, mode: MaxEncodedLen)
	fn disable_schema() -> Weight {
		Weight::from_parts(23_276_000 as u64, 0)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: VCManagement Admin (r:1 w:0)
	// Proof: VCManagement Admin (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	// Storage: VCManagement SchemaRegistry (r:1 w:1)
	// Proof: VCManagement SchemaRegistry (max_values: None, max_size: Some(2621), added: 5096, mode: MaxEncodedLen)
	fn activate_schema() -> Weight {
		Weight::from_parts(23_211_000 as u64, 0)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	// Storage: VCManagement Admin (r:1 w:0)
	// Proof: VCManagement Admin (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	// Storage: VCManagement SchemaRegistry (r:1 w:1)
	// Proof: VCManagement SchemaRegistry (max_values: None, max_size: Some(2621), added: 5096, mode: MaxEncodedLen)
	fn revoke_schema() -> Weight {
		Weight::from_parts(23_754_000 as u64, 0)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: VCManagement Delegatee (r:0 w:1)
	// Proof: VCManagement Delegatee (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	fn add_delegatee() -> Weight {
		Weight::from_parts(12_840_000 as u64, 0)
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: VCManagement Delegatee (r:1 w:1)
	// Proof: VCManagement Delegatee (max_values: None, max_size: Some(48), added: 2523, mode: MaxEncodedLen)
	fn remove_delegatee() -> Weight {
		Weight::from_parts(20_988_000 as u64, 0)
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: VCMPExtrinsicWhitelist GroupControlOn (r:1 w:0)
	// Proof Skipped: VCMPExtrinsicWhitelist GroupControlOn (max_values: Some(1), max_size: None, mode: Measured)
	fn request_vc() -> Weight {
		Weight::from_parts(35_640_000 as u64, 0)
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
	}
	// Storage: Teerex EnclaveIndex (r:1 w:0)
	// Proof Skipped: Teerex EnclaveIndex (max_values: None, max_size: None, mode: Measured)
	// Storage: VCManagement VCRegistry (r:1 w:1)
	// Proof: VCManagement VCRegistry (max_values: None, max_size: Some(312), added: 2787, mode: MaxEncodedLen)
	fn vc_issued() -> Weight {
		Weight::from_parts(26_888_000 as u64, 0)
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: Teerex EnclaveIndex (r:1 w:0)
	// Proof Skipped: Teerex EnclaveIndex (max_values: None, max_size: None, mode: Measured)
	fn some_error() -> Weight {
		Weight::from_parts(20_441_000 as u64, 0)
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
	}
	// Storage: VCManagement Admin (r:1 w:1)
	// Proof: VCManagement Admin (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	fn set_admin() -> Weight {
		Weight::from_parts(18_220_000 as u64, 0)
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: VCManagement Admin (r:1 w:0)
	// Proof: VCManagement Admin (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	// Storage: VCManagement SchemaRegistryIndex (r:1 w:1)
	// Proof: VCManagement SchemaRegistryIndex (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	// Storage: VCManagement SchemaRegistry (r:0 w:1)
	// Proof: VCManagement SchemaRegistry (max_values: None, max_size: Some(2621), added: 5096, mode: MaxEncodedLen)
	fn add_schema() -> Weight {
		Weight::from_parts(50_676_000 as u64, 0)
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(2 as u64))
	}
	// Storage: VCManagement Admin (r:1 w:0)
	// Proof: VCManagement Admin (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	// Storage: VCManagement SchemaRegistry (r:1 w:1)
	// Proof: VCManagement SchemaRegistry (max_values: None, max_size: Some(2621), added: 5096, mode: MaxEncodedLen)
	fn disable_schema() -> Weight {
		Weight::from_parts(23_276_000 as u64, 0)
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: VCManagement Admin (r:1 w:0)
	// Proof: VCManagement Admin (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	// Storage: VCManagement SchemaRegistry (r:1 w:1)
	// Proof: VCManagement SchemaRegistry (max_values: None, max_size: Some(2621), added: 5096, mode: MaxEncodedLen)
	fn activate_schema() -> Weight {
		Weight::from_parts(23_211_000 as u64, 0)
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	// Storage: VCManagement Admin (r:1 w:0)
	// Proof: VCManagement Admin (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
	// Storage: VCManagement SchemaRegistry (r:1 w:1)
	// Proof: VCManagement SchemaRegistry (max_values: None, max_size: Some(2621), added: 5096, mode: MaxEncodedLen)
	fn revoke_schema() -> Weight {
		Weight::from_parts(23_754_000 as u64, 0)
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
}

