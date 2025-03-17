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
#![allow(clippy::type_complexity)]

use frame_support::{
	migration::storage_key_iter,
	pallet_prelude::*,
	traits::{Get, OnRuntimeUpgrade},
	Twox64Concat,
};
use sp_runtime::Saturating;
use sp_std::{convert::From, marker::PhantomData, vec::Vec};

use pallet_parachain_staking::{
	BalanceOf, Bond, CandidateInfo, CandidatePool, Delegations, TopDelegations, Total,
};
pub const DECIMAL_CONVERTOR: u128 = 1_000_000u128;

// Fix Parachain Staking Storage for missing migrating TopDelegations total
pub struct FixParachainStakingStorage<T>(PhantomData<T>);

impl<T> OnRuntimeUpgrade for FixParachainStakingStorage<T>
where
	T: frame_system::Config + pallet_parachain_staking::Config,
	BalanceOf<T>: From<u128>,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		// Does not matter
		Ok(Vec::<u8>::new())
	}

	// Staked is the snapshot of Total, and remove after used, so no fix
	// AtStake, will not be fixed, let's wait extra two round with reward = 0 to make this data
	// clean DelayedPayouts, will not fix, let's wait extra two round with reward = 0 to make this
	// data clean

	// TopDelegations require fix
	// CandidateInfo require fix
	// CandidatePool require fix
	// Total fix
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceParachainStakingStorage",
			"running migration to ParachainStaking TopDelegations"
		);
		let pallet_prefix: &[u8] = b"ParachainStaking";
		let storage_item_prefix: &[u8] = b"TopDelegations";
		let mut weight: Weight = frame_support::weights::Weight::zero();

		let mut candidates = <CandidatePool<T>>::get();
		// remove all bonds
		candidates.clear();

		// intitialized total
		let mut total: BalanceOf<T> = 0u128.into();

		for (account, mut delegations) in storage_key_iter::<
			T::AccountId,
			Delegations<T::AccountId, BalanceOf<T>>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.drain()
		{
			// Patching the missed total value converting
			// This storage is already correpted
			let mut collator_delegations_sum: BalanceOf<T> = 0u128.into();
			for delegation_bond in delegations.delegations.iter() {
				collator_delegations_sum += delegation_bond.amount;
			}
			delegations.total = collator_delegations_sum;

			// Get CandidateInfo of the same collator key
			let mut metadata = <CandidateInfo<T>>::get(&account).unwrap();
			// Self + delegation total
			metadata.total_counted = metadata.bond + delegations.total;

			// Fix TopDelegations
			<TopDelegations<T>>::insert(&account, delegations);

			// Bond use its owner value to determine if equal without checking its amount
			// We need to check amount later
			candidates.insert(Bond { owner: account.clone(), amount: metadata.total_counted });
			// Add total
			total = total.saturating_add(metadata.total_counted);

			// Fix CandidateInfo
			<CandidateInfo<T>>::insert(&account, metadata);

			weight += T::DbWeight::get().reads_writes(2, 2);
		}

		// Fix CandidatePool
		candidates.0.sort_by(|a, b| a.amount.cmp(&b.amount));
		<CandidatePool<T>>::put(candidates);
		// Fix Total
		<Total<T>>::put(total);

		weight
	}

	// Check Top Delegation total = sum, collator wise
	// Check CandidateInfo total count = self bond + sum of delegation, collator wise
	// Check CandidatePool =
	// Check Total = sum CandidateInfo total count
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		// Check Top Delegation total = sum
		// Check CandidateInfo total count = self bond + sum of delegation
		// Check Total = sum CandidateInfo total count
		let mut total: BalanceOf<T> = 0u128.into();
		for (account, delegations) in <TopDelegations<T>>::iter() {
			log::info!("TopDelegations Collator: {}", hex::encode(account.encode()));
			// Start calculating collator delegation sum
			let mut collator_delegations_sum: BalanceOf<T> = 0u128.into();

			for delegation_bond in delegations.delegations.iter() {
				collator_delegations_sum += delegation_bond.amount;
			}

			// Check Top Delegation total = sum, collator wise
			assert_eq!(collator_delegations_sum, delegations.total);

			let metadata = <CandidateInfo<T>>::get(account).unwrap();
			// Check CandidateInfo total count = self bond + sum of delegation
			assert_eq!(metadata.total_counted, metadata.bond + collator_delegations_sum);

			// Collator self + Collator delegations
			total += metadata.bond + collator_delegations_sum;

			log::info!("Delegations total: {:?}", delegations.total);
			log::info!("CandidateInfo Metadata: {:?}", metadata);
		}
		// Check Total = sum CandidateInfo total count
		assert_eq!(total, <Total<T>>::get());

		// It is hard to check CandidatePool without iterating vector
		// So we just check its sum = Total
		// Get all ordered_set of bond
		let ordered_set = <CandidatePool<T>>::get();
		let mut candidate_pool_sum: BalanceOf<T> = 0u128.into();

		// Make sure the number of Order set is correct
		assert_eq!(ordered_set.0.len(), 9);
		for bond in ordered_set.0.iter() {
			candidate_pool_sum += bond.amount;
		}

		log::info!("CandidatePool: {:?}", ordered_set);

		// Check CandidatePool element's amount total = total (self bond + sum of delegation)
		log::info!("Total: {:?}", total);
		assert_eq!(total, candidate_pool_sum);

		Ok(())
	}
}
