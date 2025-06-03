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

use crate::Pallet as IdentityManagement;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, BenchmarkError};
use frame_support::traits::EnsureOrigin;
use frame_system::RawOrigin;
#[allow(unused)]
use heima_primitives::{AccountId, ErrorDetail, IMPError, Identity};
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
		IdentityManagement::<T>::add_delegatee(RawOrigin::Root.into(), account.clone())?;
	}: _(RawOrigin::Root, account.clone())
	verify{
		assert!(!Delegatee::<T>::contains_key(account));
	}
	// Benchmark `link_identity`. There are no worst conditions. The benchmark showed that
	// execution time is constant irrespective of encrypted_data size.
	link_identity {
		let caller: T::AccountId =  frame_benchmarking::account("TEST_A", 0u32, USER_SEED);
		let shard = H256::from_slice(&TEST8_MRENCLAVE);
		let encrypted_did = vec![1u8; 2048];
		let encrypted_validation_data = vec![1u8; 2048];
		let encrypted_web3networks = vec![1u8; 2048];
	}: _(RawOrigin::Signed(caller.clone()), shard, caller.clone(), encrypted_did.clone(), encrypted_validation_data.clone(), encrypted_web3networks.clone())
	verify {
		assert_last_event::<T>(Event::LinkIdentityRequested{
			shard,
			account: caller,
			encrypted_identity: encrypted_did,
			encrypted_validation_data,
			encrypted_web3networks,
		}.into());
	}

	// Benchmark `deactivate_identity`. There are no worst conditions. The benchmark showed that
	// execution time is constant irrespective of encrypted_data size.
	deactivate_identity {
		let caller: T::AccountId =  frame_benchmarking::account("TEST_A", 0u32, USER_SEED);
		let shard = H256::from_slice(&TEST8_MRENCLAVE);
		let encrypted_did = vec![1u8; 2048];
		let encrypted_validation_data = vec![1u8; 2048];
		let encrypted_web3networks = vec![1u8; 2048];
		IdentityManagement::<T>::link_identity(RawOrigin::Signed(caller.clone()).into(), shard, caller.clone(), encrypted_did.clone(), encrypted_validation_data, encrypted_web3networks)?;
	}: _(RawOrigin::Signed(caller.clone()), shard, encrypted_did.clone())
	verify {
		assert_last_event::<T>(Event::DeactivateIdentityRequested{
			shard,
			account: caller,
			encrypted_identity: encrypted_did,
		}.into());
	}

	// Benchmark `activate_identity`. There are no worst conditions. The benchmark showed that
	// execution time is constant irrespective of encrypted_data size.
	activate_identity {
		let caller: T::AccountId =  frame_benchmarking::account("TEST_A", 0u32, USER_SEED);
		let shard = H256::from_slice(&TEST8_MRENCLAVE);
		let encrypted_did = vec![1u8; 2048];
		let encrypted_validation_data = vec![1u8; 2048];
		let encrypted_web3networks = vec![1u8; 2048];
		IdentityManagement::<T>::link_identity(RawOrigin::Signed(caller.clone()).into(), shard, caller.clone(), encrypted_did.clone(), encrypted_validation_data, encrypted_web3networks)?;
	}: _(RawOrigin::Signed(caller.clone()), shard, encrypted_did.clone())
	verify {
		assert_last_event::<T>(Event::ActivateIdentityRequested{
			shard,
			account: caller,
			encrypted_identity: encrypted_did,
		}.into());
	}

	// Benchmark `identity_linked`. There are no worst conditions. The benchmark showed that
	// execution time is constant irrespective of encrypted_data size.
	identity_linked {
		let id_graph_hash = H256::default();
		let req_ext_hash = H256::default();
		let call_origin = T::TEECallOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let prime_identity: Identity =  frame_benchmarking::account::<AccountId>("TEST_A", 0u32, USER_SEED).into();
	}: _<T::RuntimeOrigin>(call_origin, prime_identity.clone(), id_graph_hash, req_ext_hash)
	verify {
		assert_last_event::<T>(Event::IdentityLinked { prime_identity, id_graph_hash, req_ext_hash }.into());
	}

	// Benchmark `identity_deactivated`. There are no worst conditions. The benchmark showed that
	// execution time is constant irrespective of encrypted_data size.
	identity_deactivated {
		let id_graph_hash = H256::default();
		let req_ext_hash = H256::default();
		let call_origin = T::TEECallOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let prime_identity: Identity =  frame_benchmarking::account::<AccountId>("TEST_A", 0u32, USER_SEED).into();
	}: _<T::RuntimeOrigin>(call_origin, prime_identity.clone(), id_graph_hash, req_ext_hash)
	verify {
		assert_last_event::<T>(Event::IdentityDeactivated { prime_identity, id_graph_hash, req_ext_hash }.into());
	}

	// Benchmark `identity_activated`. There are no worst conditions. The benchmark showed that
	// execution time is constant irrespective of encrypted_data size.
	identity_activated {
		let id_graph_hash = H256::default();
		let req_ext_hash = H256::default();
		let call_origin = T::TEECallOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let prime_identity: Identity =  frame_benchmarking::account::<AccountId>("TEST_A", 0u32, USER_SEED).into();
	}: _<T::RuntimeOrigin>(call_origin, prime_identity.clone(), id_graph_hash, req_ext_hash)
	verify {
		assert_last_event::<T>(Event::IdentityActivated { prime_identity, id_graph_hash, req_ext_hash }.into());
	}

	// Benchmark `identity_networks_set`. There are no worst conditions. The benchmark showed that
	// execution time is constant irrespective of encrypted_data size.
	identity_networks_set {
		let id_graph_hash = H256::default();
		let req_ext_hash = H256::default();
		let call_origin = T::TEECallOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let prime_identity: Identity =  frame_benchmarking::account::<AccountId>("TEST_A", 0u32, USER_SEED).into();
	}: _<T::RuntimeOrigin>(call_origin, prime_identity.clone(), id_graph_hash, req_ext_hash)
	verify {
		assert_last_event::<T>(Event::IdentityNetworksSet { prime_identity, id_graph_hash, req_ext_hash }.into());
	}

	// Benchmark `some_error`. There are no worst conditions. The benchmark showed that
	// execution time is constant irrespective of encrypted_data size.
	some_error {
		let call_origin = T::TEECallOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		let prime_identity: Identity =  frame_benchmarking::account::<AccountId>("TEST_A", 0u32, USER_SEED).into();
		let detail = ErrorDetail::WrongWeb2Handle;
		let error = IMPError::LinkIdentityFailed(detail.clone());
		let req_ext_hash = H256::default();
	}: _<T::RuntimeOrigin>(call_origin, Some(prime_identity.clone()), error, req_ext_hash)
	verify {
		assert_last_event::<T>(Event::LinkIdentityFailed { prime_identity: Some(prime_identity), detail, req_ext_hash }.into())
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
	IdentityManagement,
	crate::benchmarking::tests::new_test_ext(),
	crate::mock::Test,
);
