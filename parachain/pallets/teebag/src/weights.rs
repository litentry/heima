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

//! Autogenerated weights for pallet_teebag
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2024-07-31, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `azure-dev-2`, CPU: `Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("rococo-dev"), DB CACHE: 20

// Executed Command:
// ./target/release/litentry-collator
// benchmark
// pallet
// --chain=rococo-dev
// --execution=wasm
// --db-cache=20
// --wasm-execution=compiled
// --pallet=pallet_teebag
// --extrinsic=*
// --heap-pages=4096
// --steps=50
// --repeat=20
// --header=./LICENSE_HEADER
// --template=./templates/benchmark/pallet-weight-template.hbs
// --output=./pallets/teebag/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_teebag.
pub trait WeightInfo {
	fn force_add_enclave() -> Weight;
	fn force_remove_enclave() -> Weight;
	fn force_remove_enclave_by_mrenclave() -> Weight;
	fn force_remove_enclave_by_worker_type() -> Weight;
	fn force_add_authorized_enclave() -> Weight;
	fn force_remove_authorized_enclave() -> Weight;
	fn register_enclave_with_ias_attestation() -> Weight;
	fn register_enclave_with_dcap_attestation() -> Weight;
	fn unregister_enclave() -> Weight;
	fn register_quoting_enclave() -> Weight;
	fn register_tcb_info() -> Weight;
	fn post_opaque_task() -> Weight;
	fn parentchain_block_processed() -> Weight;
	fn sidechain_block_imported() -> Weight;
	fn add_enclave_identifier() -> Weight;
}

/// Weights for pallet_teebag using the Substrate node and recommended hardware.
pub struct LitentryWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for LitentryWeight<T> {
	/// Storage: Teebag EnclaveRegistry (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveIdentifier (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveIdentifier (max_values: None, max_size: None, mode: Measured)
	fn force_add_enclave() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `213`
		//  Estimated: `3678`
		// Minimum execution time: 21_864_000 picoseconds.
		Weight::from_parts(22_185_000, 3678)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: Teebag EnclaveRegistry (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveIdentifier (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveIdentifier (max_values: None, max_size: None, mode: Measured)
	fn force_remove_enclave() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `541`
		//  Estimated: `4006`
		// Minimum execution time: 25_374_000 picoseconds.
		Weight::from_parts(26_135_000, 4006)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: Teebag EnclaveRegistry (r:4 w:3)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveIdentifier (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveIdentifier (max_values: None, max_size: None, mode: Measured)
	fn force_remove_enclave_by_mrenclave() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `720`
		//  Estimated: `11610`
		// Minimum execution time: 65_746_000 picoseconds.
		Weight::from_parts(66_865_000, 11610)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: Teebag EnclaveRegistry (r:4 w:3)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveIdentifier (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveIdentifier (max_values: None, max_size: None, mode: Measured)
	fn force_remove_enclave_by_worker_type() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `720`
		//  Estimated: `11610`
		// Minimum execution time: 65_668_000 picoseconds.
		Weight::from_parts(66_489_000, 11610)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
	}
	/// Storage: Teebag AuthorizedEnclave (r:1 w:1)
	/// Proof Skipped: Teebag AuthorizedEnclave (max_values: None, max_size: None, mode: Measured)
	fn force_add_authorized_enclave() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `383`
		//  Estimated: `3848`
		// Minimum execution time: 18_972_000 picoseconds.
		Weight::from_parts(19_308_000, 3848)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Teebag AuthorizedEnclave (r:1 w:1)
	/// Proof Skipped: Teebag AuthorizedEnclave (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveRegistry (r:1 w:0)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	fn force_remove_authorized_enclave() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `415`
		//  Estimated: `3880`
		// Minimum execution time: 26_293_000 picoseconds.
		Weight::from_parts(26_898_000, 3880)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Timestamp Now (r:1 w:0)
	/// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	/// Storage: Teebag Mode (r:1 w:0)
	/// Proof Skipped: Teebag Mode (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Teebag AuthorizedEnclave (r:1 w:1)
	/// Proof Skipped: Teebag AuthorizedEnclave (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveRegistry (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveIdentifier (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveIdentifier (max_values: None, max_size: None, mode: Measured)
	fn register_enclave_with_ias_attestation() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `427`
		//  Estimated: `3892`
		// Minimum execution time: 1_591_546_000 picoseconds.
		Weight::from_parts(1_599_242_000, 3892)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: Timestamp Now (r:1 w:0)
	/// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	/// Storage: Teebag QuotingEnclaveRegistry (r:1 w:0)
	/// Proof Skipped: Teebag QuotingEnclaveRegistry (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Teebag TcbInfo (r:1 w:0)
	/// Proof Skipped: Teebag TcbInfo (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag Mode (r:1 w:0)
	/// Proof Skipped: Teebag Mode (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Teebag AuthorizedEnclave (r:1 w:1)
	/// Proof Skipped: Teebag AuthorizedEnclave (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveRegistry (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveIdentifier (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveIdentifier (max_values: None, max_size: None, mode: Measured)
	fn register_enclave_with_dcap_attestation() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `644`
		//  Estimated: `4109`
		// Minimum execution time: 4_090_222_000 picoseconds.
		Weight::from_parts(4_112_436_000, 4109)
			.saturating_add(T::DbWeight::get().reads(7_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
	/// Storage: Teebag EnclaveRegistry (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveIdentifier (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveIdentifier (max_values: None, max_size: None, mode: Measured)
	fn unregister_enclave() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `458`
		//  Estimated: `3923`
		// Minimum execution time: 26_349_000 picoseconds.
		Weight::from_parts(27_053_000, 3923)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: Timestamp Now (r:1 w:0)
	/// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	/// Storage: Teebag QuotingEnclaveRegistry (r:0 w:1)
	/// Proof Skipped: Teebag QuotingEnclaveRegistry (max_values: Some(1), max_size: None, mode: Measured)
	fn register_quoting_enclave() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `94`
		//  Estimated: `1493`
		// Minimum execution time: 2_040_904_000 picoseconds.
		Weight::from_parts(2_054_957_000, 1493)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: Timestamp Now (r:1 w:0)
	/// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	/// Storage: Teebag TcbInfo (r:0 w:1)
	/// Proof Skipped: Teebag TcbInfo (max_values: None, max_size: None, mode: Measured)
	fn register_tcb_info() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `94`
		//  Estimated: `1493`
		// Minimum execution time: 2_192_595_000 picoseconds.
		Weight::from_parts(2_214_305_000, 1493)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	fn post_opaque_task() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 10_938_000 picoseconds.
		Weight::from_parts(11_114_000, 0)
	}
	/// Storage: Teebag EnclaveRegistry (r:1 w:0)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: Timestamp Now (r:1 w:0)
	/// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	fn parentchain_block_processed() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `407`
		//  Estimated: `3872`
		// Minimum execution time: 21_505_000 picoseconds.
		Weight::from_parts(21_781_000, 3872)
			.saturating_add(T::DbWeight::get().reads(2_u64))
	}
	/// Storage: Teebag EnclaveRegistry (r:1 w:0)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: Timestamp Now (r:1 w:0)
	/// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	/// Storage: Teebag EnclaveIdentifier (r:1 w:0)
	/// Proof Skipped: Teebag EnclaveIdentifier (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag SidechainBlockFinalizationCandidate (r:1 w:1)
	/// Proof Skipped: Teebag SidechainBlockFinalizationCandidate (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag LatestSidechainBlockConfirmation (r:0 w:1)
	/// Proof Skipped: Teebag LatestSidechainBlockConfirmation (max_values: None, max_size: None, mode: Measured)
	fn sidechain_block_imported() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `528`
		//  Estimated: `3993`
		// Minimum execution time: 33_377_000 picoseconds.
		Weight::from_parts(34_051_000, 3993)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}

	fn add_enclave_identifier() -> Weight {
        Weight::from_parts(15_000_000, 3000)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: Teebag EnclaveRegistry (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveIdentifier (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveIdentifier (max_values: None, max_size: None, mode: Measured)
	fn force_add_enclave() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `213`
		//  Estimated: `3678`
		// Minimum execution time: 21_864_000 picoseconds.
		Weight::from_parts(22_185_000, 3678)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: Teebag EnclaveRegistry (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveIdentifier (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveIdentifier (max_values: None, max_size: None, mode: Measured)
	fn force_remove_enclave() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `541`
		//  Estimated: `4006`
		// Minimum execution time: 25_374_000 picoseconds.
		Weight::from_parts(26_135_000, 4006)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: Teebag EnclaveRegistry (r:4 w:3)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveIdentifier (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveIdentifier (max_values: None, max_size: None, mode: Measured)
	fn force_remove_enclave_by_mrenclave() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `720`
		//  Estimated: `11610`
		// Minimum execution time: 65_746_000 picoseconds.
		Weight::from_parts(66_865_000, 11610)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
	/// Storage: Teebag EnclaveRegistry (r:4 w:3)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveIdentifier (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveIdentifier (max_values: None, max_size: None, mode: Measured)
	fn force_remove_enclave_by_worker_type() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `720`
		//  Estimated: `11610`
		// Minimum execution time: 65_668_000 picoseconds.
		Weight::from_parts(66_489_000, 11610)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(4_u64))
	}
	/// Storage: Teebag AuthorizedEnclave (r:1 w:1)
	/// Proof Skipped: Teebag AuthorizedEnclave (max_values: None, max_size: None, mode: Measured)
	fn force_add_authorized_enclave() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `383`
		//  Estimated: `3848`
		// Minimum execution time: 18_972_000 picoseconds.
		Weight::from_parts(19_308_000, 3848)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: Teebag AuthorizedEnclave (r:1 w:1)
	/// Proof Skipped: Teebag AuthorizedEnclave (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveRegistry (r:1 w:0)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	fn force_remove_authorized_enclave() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `415`
		//  Estimated: `3880`
		// Minimum execution time: 26_293_000 picoseconds.
		Weight::from_parts(26_898_000, 3880)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: Timestamp Now (r:1 w:0)
	/// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	/// Storage: Teebag Mode (r:1 w:0)
	/// Proof Skipped: Teebag Mode (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Teebag AuthorizedEnclave (r:1 w:1)
	/// Proof Skipped: Teebag AuthorizedEnclave (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveRegistry (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveIdentifier (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveIdentifier (max_values: None, max_size: None, mode: Measured)
	fn register_enclave_with_ias_attestation() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `427`
		//  Estimated: `3892`
		// Minimum execution time: 1_591_546_000 picoseconds.
		Weight::from_parts(1_599_242_000, 3892)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: Timestamp Now (r:1 w:0)
	/// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	/// Storage: Teebag QuotingEnclaveRegistry (r:1 w:0)
	/// Proof Skipped: Teebag QuotingEnclaveRegistry (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Teebag TcbInfo (r:1 w:0)
	/// Proof Skipped: Teebag TcbInfo (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag Mode (r:1 w:0)
	/// Proof Skipped: Teebag Mode (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: Teebag AuthorizedEnclave (r:1 w:1)
	/// Proof Skipped: Teebag AuthorizedEnclave (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveRegistry (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveIdentifier (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveIdentifier (max_values: None, max_size: None, mode: Measured)
	fn register_enclave_with_dcap_attestation() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `644`
		//  Estimated: `4109`
		// Minimum execution time: 4_090_222_000 picoseconds.
		Weight::from_parts(4_112_436_000, 4109)
			.saturating_add(RocksDbWeight::get().reads(7_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: Teebag EnclaveRegistry (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag EnclaveIdentifier (r:1 w:1)
	/// Proof Skipped: Teebag EnclaveIdentifier (max_values: None, max_size: None, mode: Measured)
	fn unregister_enclave() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `458`
		//  Estimated: `3923`
		// Minimum execution time: 26_349_000 picoseconds.
		Weight::from_parts(27_053_000, 3923)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: Timestamp Now (r:1 w:0)
	/// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	/// Storage: Teebag QuotingEnclaveRegistry (r:0 w:1)
	/// Proof Skipped: Teebag QuotingEnclaveRegistry (max_values: Some(1), max_size: None, mode: Measured)
	fn register_quoting_enclave() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `94`
		//  Estimated: `1493`
		// Minimum execution time: 2_040_904_000 picoseconds.
		Weight::from_parts(2_054_957_000, 1493)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: Timestamp Now (r:1 w:0)
	/// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	/// Storage: Teebag TcbInfo (r:0 w:1)
	/// Proof Skipped: Teebag TcbInfo (max_values: None, max_size: None, mode: Measured)
	fn register_tcb_info() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `94`
		//  Estimated: `1493`
		// Minimum execution time: 2_192_595_000 picoseconds.
		Weight::from_parts(2_214_305_000, 1493)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	fn post_opaque_task() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 10_938_000 picoseconds.
		Weight::from_parts(11_114_000, 0)
	}
	/// Storage: Teebag EnclaveRegistry (r:1 w:0)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: Timestamp Now (r:1 w:0)
	/// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	fn parentchain_block_processed() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `407`
		//  Estimated: `3872`
		// Minimum execution time: 21_505_000 picoseconds.
		Weight::from_parts(21_781_000, 3872)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
	}
	/// Storage: Teebag EnclaveRegistry (r:1 w:0)
	/// Proof Skipped: Teebag EnclaveRegistry (max_values: None, max_size: None, mode: Measured)
	/// Storage: Timestamp Now (r:1 w:0)
	/// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	/// Storage: Teebag EnclaveIdentifier (r:1 w:0)
	/// Proof Skipped: Teebag EnclaveIdentifier (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag SidechainBlockFinalizationCandidate (r:1 w:1)
	/// Proof Skipped: Teebag SidechainBlockFinalizationCandidate (max_values: None, max_size: None, mode: Measured)
	/// Storage: Teebag LatestSidechainBlockConfirmation (r:0 w:1)
	/// Proof Skipped: Teebag LatestSidechainBlockConfirmation (max_values: None, max_size: None, mode: Measured)
	fn sidechain_block_imported() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `528`
		//  Estimated: `3993`
		// Minimum execution time: 33_377_000 picoseconds.
		Weight::from_parts(34_051_000, 3993)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}

	fn add_enclave_identifier() -> Weight {
        Weight::from_parts(15_000_000, 3000)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
}
