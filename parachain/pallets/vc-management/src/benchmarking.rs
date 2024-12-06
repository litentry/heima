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

//! Benchmarking setup for pallet-identity-management

use super::*;

use crate::Pallet as VCManagement;
use core_primitives::{AccountId, Assertion, ErrorDetail, Identity, VCMPError};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, BenchmarkError};
use frame_support::traits::EnsureOrigin;
use frame_system::RawOrigin;
use sp_core::H256;
use sp_std::vec;

use pallet_teebag::test_util::TEST8_MRENCLAVE;
const USER_SEED: u32 = 9966;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
	// Benchmark `add_delegatee`. There are no worst conditions. The benchmark showed that
	// execution time is constant irrespective of encrypted_data size.
	add_delegatee {
		let account: T::AccountId =  frame_benchmarking::account("TEST_A", 0u32, USER_SEED);
	}: _(RawOrigin::Root, account.clone())
	verify{
		assert!(Delegatee::<T>::contains_key(account));
	}
	// Benchmark `remove_delegatee`. There are no worst conditions. The benchmark showed that
	// execution time is constant irrespective of encrypted_data size.
	remove_delegatee {
		let account: T::AccountId =  frame_benchmarking::account("TEST_A", 0u32, USER_SEED);
		VCManagement::<T>::add_delegatee(RawOrigin::Root.into(), account.clone())?;
	}: _(RawOrigin::Root, account.clone())
	verify{
		assert!(!Delegatee::<T>::contains_key(account));
	}

	// Benchmark `request_vc`. There are no worst conditions. The benchmark showed that
	// execution time is constant irrespective of encrypted_data size.
	request_vc {
		let account: T::AccountId =  frame_benchmarking::account("TEST_A", 0u32, USER_SEED);
		let shard = H256::from_slice(&TEST8_MRENCLAVE);
		let assertion = Assertion::A1;
	}: _(RawOrigin::Signed(account.clone()), shard, assertion.clone())
	verify{
		assert_last_event::<T>(Event::VCRequested{ account, shard, assertion }.into());
	}

	// Benchmark `vc_issued`. There are no worst conditions. The benchmark showed that
	// execution time is constant irrespective of encrypted_data size.
	vc_issued {
		let call_origin = T::TEECallOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let identity: Identity =  frame_benchmarking::account::<AccountId>("TEST_A", 0u32, USER_SEED).into();
		let assertion = Assertion::A1;
		let id_graph_hash = H256::default();
		let req_ext_hash = H256::default();
	}: _<T::RuntimeOrigin>(call_origin, identity.clone(), assertion.clone(), id_graph_hash, req_ext_hash)
	verify{
		assert_last_event::<T>(Event::VCIssued{ identity, assertion, id_graph_hash, req_ext_hash }.into());
	}

	// Benchmark `some_error`. There are no worst conditions. The benchmark showed that
	// execution time is constant irrespective of encrypted_data size.
	some_error {
		let call_origin = T::TEECallOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let identity: Identity =  frame_benchmarking::account::<AccountId>("TEST_A", 0u32, USER_SEED).into();
		let detail = ErrorDetail::WrongWeb2Handle;
		let assertion = Assertion::A1;
		let error = VCMPError::RequestVCFailed(assertion.clone(), detail.clone());
		let req_ext_hash = H256::default();
	}: _<T::RuntimeOrigin>(call_origin, Some(identity.clone()), error, req_ext_hash)
	verify {
		assert_last_event::<T>(Event::RequestVCFailed { identity: Some(identity), assertion, detail, req_ext_hash }.into())
	}

	// Benchmark `set_admin`. There are no worst conditions. The benchmark showed that
	// execution time is constant irrespective of encrypted_data size.
	set_admin {
		let old_admin: T::AccountId =  frame_benchmarking::account("TEST_A", 0u32, USER_SEED);
		VCManagement::<T>::set_admin(RawOrigin::Root.into(), old_admin.clone())?;
		let new_schema_admin: T::AccountId =  frame_benchmarking::account("TEST_A", 0u32, USER_SEED + 1);
	}: _(RawOrigin::Root, new_schema_admin.clone())
	verify {
		assert_last_event::<T>(Event::AdminChanged { old_admin: Some(old_admin), new_admin: Some(new_schema_admin) }.into())
	}

	// Benchmark `add_schema`. There are no worst conditions. The benchmark showed that
	// execution time is constant irrespective of encrypted_data size.
	add_schema {
		let account: T::AccountId =  frame_benchmarking::account("TEST_A", 0u32, USER_SEED);
		VCManagement::<T>::set_admin(RawOrigin::Root.into(), account.clone())?;
		let id: Vec<u8> = vec![1, 2, 3, 4];
		let content: Vec<u8> = vec![5, 6, 7, 8];
		let shard = H256::from_slice(&TEST8_MRENCLAVE);
	}: _(RawOrigin::Signed(account.clone()), shard, id, content)
	verify {
		assert_last_event::<T>(Event::SchemaIssued { account, shard, index: 0 }.into())
	}

	// Benchmark `disable_schema`. There are no worst conditions. The benchmark showed that
	// execution time is constant irrespective of encrypted_data size.
	disable_schema {
		let account: T::AccountId =  frame_benchmarking::account("TEST_A", 0u32, USER_SEED);
		VCManagement::<T>::set_admin(RawOrigin::Root.into(), account.clone())?;
		let id: Vec<u8> = vec![1, 2, 3, 4];
		let content: Vec<u8> = vec![5, 6, 7, 8];
		let shard = H256::from_slice(&TEST8_MRENCLAVE);
		VCManagement::<T>::add_schema(RawOrigin::Signed(account.clone()).into(), shard, id, content)?;
	}: _(RawOrigin::Signed(account.clone()), shard, 0)
	verify {
		assert_last_event::<T>(Event::SchemaDisabled { account, shard, index: 0 }.into())
	}

	// Benchmark `activate_schema`. There are no worst conditions. The benchmark showed that
	// execution time is constant irrespective of encrypted_data size.
	activate_schema {
		let account: T::AccountId =  frame_benchmarking::account("TEST_A", 0u32, USER_SEED);
		VCManagement::<T>::set_admin(RawOrigin::Root.into(), account.clone())?;
		let id: Vec<u8> = vec![1, 2, 3, 4];
		let content: Vec<u8> = vec![5, 6, 7, 8];
		let shard = H256::from_slice(&TEST8_MRENCLAVE);
		VCManagement::<T>::add_schema(RawOrigin::Signed(account.clone()).into(), shard, id, content)?;
		VCManagement::<T>::disable_schema(RawOrigin::Signed(account.clone()).into(), shard, 0)?;
	}: _(RawOrigin::Signed(account.clone()), shard, 0)
	verify {
		assert_last_event::<T>(Event::SchemaActivated { account, shard, index: 0 }.into())
	}

	// Benchmark `revoke_schema`. There are no worst conditions. The benchmark showed that
	// execution time is constant irrespective of encrypted_data size.
	revoke_schema {
		let account: T::AccountId =  frame_benchmarking::account("TEST_A", 0u32, USER_SEED);
		VCManagement::<T>::set_admin(RawOrigin::Root.into(), account.clone())?;
		let id: Vec<u8> = vec![1, 2, 3, 4];
		let content: Vec<u8> = vec![5, 6, 7, 8];
		let shard = H256::from_slice(&TEST8_MRENCLAVE);
		VCManagement::<T>::add_schema(RawOrigin::Signed(account.clone()).into(), shard, id, content)?;
	}: _(RawOrigin::Signed(account.clone()), shard, 0)
	verify {
		assert_last_event::<T>(Event::SchemaRevoked { account, shard, index: 0 }.into())
	}

	// Benchmark `on_vc_issued`. There are no worst conditions. The benchmark showed that
	// execution time is constant irrespective of encrypted_data size.
	on_vc_issued {
		let call_origin = T::TEECallOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let identity: Identity =  frame_benchmarking::account::<AccountId>("TEST_A", 0u32, USER_SEED).into();
		let assertion = Assertion::A1;
		let omni_account: T::AccountId =  frame_benchmarking::account("TEST_OMNI_ACCOUNT", 0u32, USER_SEED);
		let req_ext_hash = H256::default();
	}: _<T::RuntimeOrigin>(call_origin, identity.clone(), assertion.clone(), omni_account.clone(), req_ext_hash)
	verify{
		assert_last_event::<T>(Event::VCIssuedNew{ identity, assertion, omni_account, req_ext_hash }.into());
	}
}

#[cfg(test)]
mod tests {
	use crate::mock::Test;
	use sp_runtime::BuildStorage;
	pub fn new_test_ext() -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
		t.into()
	}
}

impl_benchmark_test_suite!(
	VCManagement,
	crate::benchmarking::tests::new_test_ext(),
	crate::mock::Test,
);
