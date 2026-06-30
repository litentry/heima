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

//! bridge benchmark file

#![cfg(feature = "runtime-benchmarks")]
#![allow(clippy::type_complexity)]
#![allow(clippy::useless_vec)]
use super::*;
use crate::{BridgeChainId, Call, Event, Pallet as bridge};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_system::{Call as SystemCall, RawOrigin};
use sp_std::{boxed::Box, vec, vec::Vec};

const USER_SEED: u32 = 9966;

fn assert_last_event<T: Config>(generic_event: <T as frame_system::Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn make_proposal<T: Config>(remark: Vec<u8>) -> T::Proposal {
	SystemCall::<T>::remark { remark }.into()
}

benchmarks! {
	set_threshold{
		let i = 100u32;
	}:_(RawOrigin::Root,i)
	verify{
		assert_eq!(RelayerThreshold::<T>::get(),i);
	}

	whitelist_chain{
		let bridgechain_id = T::BridgeChainId::get().saturating_add(1);
	}:_(RawOrigin::Root,bridgechain_id)
	verify{
		assert!(ChainNonces::<T>::contains_key(bridgechain_id));
	}

	add_relayer{
		let relayer_id: T::AccountId = account("TEST_A", 0u32, USER_SEED);
	}:_(RawOrigin::Root,relayer_id.clone())
	verify{
		assert!(Relayers::<T>::contains_key(relayer_id));
	}

	remove_relayer{
		let relayer_id: T::AccountId = account("TEST_B", 0u32, USER_SEED);

		bridge::<T>::add_relayer(
			RawOrigin::Root.into(),
			relayer_id.clone()
		)?;
	}:_(RawOrigin::Root,relayer_id.clone())
	verify{
		  assert!(!Relayers::<T>::contains_key(relayer_id));
	}

	acknowledge_proposal{
		let relayer_id: T::AccountId = account("TEST_A", 0u32, USER_SEED);
		let prop_id:DepositNonce = 1;
		let src_id:BridgeChainId = T::BridgeChainId::get().saturating_add(1);
		let r_id:ResourceId = derive_resource_id(src_id, b"remark");

		let proposal = make_proposal::<T>(vec![]);
		let method = vec![0u8];

		bridge::<T>::add_relayer(
			RawOrigin::Root.into(),
			relayer_id.clone(),
		)?;

		bridge::<T>::whitelist_chain(
			RawOrigin::Root.into(),
			src_id,
		)?;

	}:_(RawOrigin::Signed(relayer_id),prop_id,src_id, r_id, Box::new(proposal))
	verify{
		assert_last_event::<T>(Event::ProposalSucceeded(src_id, prop_id).into());
	}

	reject_proposal{
		let relayer_id: T::AccountId = account("TEST_B", 1u32, USER_SEED+1);
		let prop_id:DepositNonce = 1;
		let src_id:BridgeChainId = T::BridgeChainId::get().saturating_add(1);
		let r_id:ResourceId = derive_resource_id(src_id, b"remark");

		let proposal = make_proposal::<T>(vec![]);

		let method = vec![0u8];

		bridge::<T>::add_relayer(
			RawOrigin::Root.into(),
			relayer_id.clone(),
		)?;

		bridge::<T>::whitelist_chain(
			RawOrigin::Root.into(),
			src_id,
		)?;

	}:_(RawOrigin::Signed(relayer_id),prop_id,src_id,r_id,Box::new(proposal))
	verify{
		assert_last_event::<T>(Event::ProposalRejected(src_id,prop_id).into());
	}

	eval_vote_state{
		// construct the bridge relayer and so on
		let relayer_id_a: T::AccountId = account("TEST_A", 0u32, USER_SEED);
		let relayer_id_b: T::AccountId = account("TEST_B", 1u32, USER_SEED+1);
		let relayer_id_c: T::AccountId = account("TEST_C", 2u32, USER_SEED-1);
		let prop_id:DepositNonce = 1;
		let src_id:BridgeChainId = T::BridgeChainId::get().saturating_add(1);
		let r_id:ResourceId = derive_resource_id(src_id, b"remark");

		let proposal = make_proposal::<T>(vec![]);
		let method = vec![0u8];

		bridge::<T>::add_relayer(
			RawOrigin::Root.into(),
			relayer_id_a,
		)?;

		bridge::<T>::add_relayer(
			RawOrigin::Root.into(),
			relayer_id_b.clone(),
		)?;

		bridge::<T>::add_relayer(
			RawOrigin::Root.into(),
			relayer_id_c.clone(),
		)?;

		bridge::<T>::whitelist_chain(
			RawOrigin::Root.into(),
			src_id,
		)?;

		bridge::<T>::reject_proposal(
			RawOrigin::Signed(relayer_id_b).into(),
			prop_id,
			src_id,
			r_id,
			Box::new(proposal.clone()),
		)?;

	}:_(RawOrigin::Signed(relayer_id_c),prop_id,src_id,Box::new(proposal))

}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
