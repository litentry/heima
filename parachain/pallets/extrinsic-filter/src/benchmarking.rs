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

//! Benchmarking setup for pallet-extrinsic-filter

use super::*;

#[allow(unused)]
use crate::Pallet as ExtrinsicFilter;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use sp_std::vec;

const MAX_BYTES: u32 = 1_024;

fn assert_last_event<T: Config>(generic_event: <T as frame_system::Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
	block_extrinsics {
		let p in 1 .. MAX_BYTES;
		let f in 1 .. MAX_BYTES;

		let pallet_name_bytes = vec![0u8; p as usize];
		let function_name_bytes = vec![0u8; f as usize];
	}: _(RawOrigin::Root, pallet_name_bytes.clone(), Some(function_name_bytes.clone()))
	verify {
		assert_eq!(
			ExtrinsicFilter::<T>::blocked_extrinsics((pallet_name_bytes.clone(), function_name_bytes.clone())),
			Some(())
		);
		assert_last_event::<T>(Event::ExtrinsicsBlocked {
			pallet_name_bytes,
			function_name_bytes: Some(function_name_bytes)
		}.into());
	}

	unblock_extrinsics {
		let p in 1 .. MAX_BYTES;
		let f in 1 .. MAX_BYTES;

		let pallet_name_bytes = vec![0u8; p as usize];
		let function_name_bytes = vec![0u8; f as usize];
		// block them
		assert!(ExtrinsicFilter::<T>::block_extrinsics(
			RawOrigin::Root.into(),
			pallet_name_bytes.clone(),
			Some(function_name_bytes.clone())
		).is_ok());
	}: _(RawOrigin::Root, pallet_name_bytes.clone(), Some(function_name_bytes.clone()))
	verify {
		assert_eq!(
			ExtrinsicFilter::<T>::blocked_extrinsics((pallet_name_bytes.clone(), function_name_bytes.clone())),
			None
		);
		assert_last_event::<T>(Event::ExtrinsicsUnblocked {
			pallet_name_bytes,
			function_name_bytes: Some(function_name_bytes)
		}.into());
	}
}

impl_benchmark_test_suite!(ExtrinsicFilter, crate::mock::new_test_ext(), crate::mock::Test,);
