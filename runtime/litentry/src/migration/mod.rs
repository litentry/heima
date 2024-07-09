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
use frame_support::{
	migration::{clear_storage_prefix, storage_key_iter},
	pallet_prelude::*,
	traits::{Get, OnRuntimeUpgrade},
	Twox64Concat,
};
use sp_runtime::Saturating;
use sp_std::{
	convert::{From, TryInto},
	marker::PhantomData,
	vec::Vec,
};

use pallet_parachain_staking::{
	set::OrderedSet, BalanceOf, CandidateInfo, CandidateMetadata, Delegator, DelegatorState,
};
pub const DECIMAL_CONVERTOR: u128 = 1_000_000u128;

#[cfg(feature = "try-runtime")]
use parity_scale_codec::Encode;
#[cfg(feature = "try-runtime")]
use sp_std::collections::btree_map::BTreeMap;

// Replace Parachain Staking Storage for Decimal Change from 12 to 18
pub struct ReplaceParachainStakingStorage<T>(PhantomData<T>);
impl<T: pallet_parachain_staking::Config> ReplaceParachainStakingStorage<T>
where
	BalanceOf<T>: From<u128>,
{
	pub fn replace_delegator_state_storage() -> frame_support::weights::Weight {
		// DelegatorState
		let pallet_prefix: &[u8] = b"ParachainStaking";
		let storage_item_prefix: &[u8] = b"DelegatorState";
		// Read all the data into memory.
		// https://crates.parity.io/frame_support/storage/migration/fn.storage_key_iter.html
		let stored_data: Vec<_> = storage_key_iter::<
			T::AccountId,
			Delegator<T::AccountId, BalanceOf<T>>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.collect();
		let migrated_count = frame_support::weights::Weight::from_parts(
			0,
			stored_data
				.len()
				.try_into()
				.expect("There are between 0 and 2**64 mappings stored."),
		);
		// Now remove the old storage
		// https://crates.parity.io/frame_support/storage/migration/fn.clear_storage_prefix.html
		clear_storage_prefix(pallet_prefix, storage_item_prefix, &[], None, None);
		// Assert that old storage is empty
		assert!(storage_key_iter::<
			T::AccountId,
			Delegator<T::AccountId, BalanceOf<T>>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.next()
		.is_none());
		for (account, state) in stored_data {
			let mut new_delegator: Delegator<T::AccountId, BalanceOf<T>> = state;
			new_delegator.total = new_delegator.total.saturating_mul(DECIMAL_CONVERTOR.into());
			new_delegator.less_total =
				new_delegator.less_total.saturating_mul(DECIMAL_CONVERTOR.into());
			let mut sorted_inner_vector = new_delegator.delegations.0;
			for elem in sorted_inner_vector.iter_mut() {
				elem.amount = elem.amount.saturating_mul(DECIMAL_CONVERTOR.into());
			}
			new_delegator.delegations = OrderedSet::from(sorted_inner_vector);

			<DelegatorState<T>>::insert(&account, new_delegator)
		}
		let weight = T::DbWeight::get();
		migrated_count.saturating_mul(weight.write + weight.read)
	}

	pub fn replace_candidate_info_storage() -> frame_support::weights::Weight {
		// CandidateInfo
		let pallet_prefix: &[u8] = b"ParachainStaking";
		let storage_item_prefix: &[u8] = b"CandidateInfo";
		// Read all the data into memory.
		// https://crates.parity.io/frame_support/storage/migration/fn.storage_key_iter.html
		let stored_data: Vec<_> = storage_key_iter::<
			T::AccountId,
			CandidateMetadata<BalanceOf<T>>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.collect();
		let migrated_count = frame_support::weights::Weight::from_parts(
			0,
			stored_data
				.len()
				.try_into()
				.expect("There are between 0 and 2**64 mappings stored."),
		);
		// Now remove the old storage
		// https://crates.parity.io/frame_support/storage/migration/fn.clear_storage_prefix.html
		clear_storage_prefix(pallet_prefix, storage_item_prefix, &[], None, None);
		// Assert that old storage is empty
		assert!(storage_key_iter::<T::AccountId, CandidateMetadata<BalanceOf<T>>, Twox64Concat>(
			pallet_prefix,
			storage_item_prefix
		)
		.next()
		.is_none());
		for (account, state) in stored_data {
			let mut new_metadata: CandidateMetadata<BalanceOf<T>> = state;
			new_metadata.bond = new_metadata.bond.saturating_mul(DECIMAL_CONVERTOR.into());
			new_metadata.total_counted =
				new_metadata.total_counted.saturating_mul(DECIMAL_CONVERTOR.into());
			new_metadata.lowest_top_delegation_amount = new_metadata
				.lowest_top_delegation_amount
				.saturating_mul(DECIMAL_CONVERTOR.into());
			new_metadata.highest_bottom_delegation_amount = new_metadata
				.highest_bottom_delegation_amount
				.saturating_mul(DECIMAL_CONVERTOR.into());
			new_metadata.lowest_bottom_delegation_amount = new_metadata
				.lowest_bottom_delegation_amount
				.saturating_mul(DECIMAL_CONVERTOR.into());

			if let Some(mut i) = new_metadata.request {
				i.amount = i.amount.saturating_mul(DECIMAL_CONVERTOR.into());
			}
			<CandidateInfo<T>>::insert(&account, new_metadata)
		}
		let weight = T::DbWeight::get();
		migrated_count.saturating_mul(weight.write + weight.read)
	}
}

#[cfg(feature = "try-runtime")]
impl<T: pallet_parachain_staking::Config> ReplaceParachainStakingStorage<T>
where
	BalanceOf<T>: From<u128>,
{
	pub fn pre_upgrade_delegator_state_storage() -> Result<Vec<u8>, &'static str> {
		// get DelegatorState to check consistency
		let result: BTreeMap<T::AccountId, Delegator<T::AccountId, BalanceOf<T>>> =
			<DelegatorState<T>>::iter()
				.map(|(account, state)| {
					let mut new_delegator: Delegator<T::AccountId, BalanceOf<T>> = state;
					new_delegator.total =
						new_delegator.total.saturating_mul(DECIMAL_CONVERTOR.into());
					new_delegator.less_total =
						new_delegator.less_total.saturating_mul(DECIMAL_CONVERTOR.into());
					let mut sorted_inner_vector = new_delegator.delegations.0;
					for elem in sorted_inner_vector.iter_mut() {
						elem.amount = elem.amount.saturating_mul(DECIMAL_CONVERTOR.into());
					}
					new_delegator.delegations = OrderedSet::from(sorted_inner_vector);

					(account, new_delegator)
				})
				.collect();
		Ok(result.encode())
	}
	pub fn post_upgrade_delegator_state_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_state =
			BTreeMap::<T::AccountId, Delegator<T::AccountId, BalanceOf<T>>>::decode(
				&mut &state[..],
			)
			.map_err(|_| "Failed to decode Delegator")?;
		// check DelegatorState are the same as the expected
		for (account, actual_result) in <DelegatorState<T>>::iter() {
			let expected_result: Delegator<T::AccountId, BalanceOf<T>> =
				*(expected_state.get(&account).ok_or("Not Expected Delegator")?);
			assert_eq!(expected_result, actual_result);
		}
		Ok(())
	}
	pub fn pre_upgrade_candidate_info_storage() -> Result<Vec<u8>, &'static str> {
		// get DelegatorState to check consistency
		let result: BTreeMap<T::AccountId, CandidateMetadata<BalanceOf<T>>> =
			<CandidateInfo<T>>::iter()
				.map(|(account, state)| {
					let mut new_metadata: CandidateMetadata<BalanceOf<T>> = state;
					new_metadata.bond = new_metadata.bond.saturating_mul(DECIMAL_CONVERTOR.into());
					new_metadata.total_counted =
						new_metadata.total_counted.saturating_mul(DECIMAL_CONVERTOR.into());
					new_metadata.lowest_top_delegation_amount = new_metadata
						.lowest_top_delegation_amount
						.saturating_mul(DECIMAL_CONVERTOR.into());
					new_metadata.highest_bottom_delegation_amount = new_metadata
						.highest_bottom_delegation_amount
						.saturating_mul(DECIMAL_CONVERTOR.into());
					new_metadata.lowest_bottom_delegation_amount = new_metadata
						.lowest_bottom_delegation_amount
						.saturating_mul(DECIMAL_CONVERTOR.into());

					if let Some(mut i) = new_metadata.request {
						i.amount = i.amount.saturating_mul(DECIMAL_CONVERTOR.into());
					}

					(account, new_metadata)
				})
				.collect();
		Ok(result.encode())
	}
	pub fn post_upgrade_candidate_info_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_state =
			BTreeMap::<T::AccountId, CandidateMetadata<BalanceOf<T>>>::decode(&mut &state[..])
				.map_err(|_| "Failed to decode CandidateMetadata")?;
		// check CandidateInfo are the same as the expected
		for (account, actual_result) in <CandidateInfo<T>>::iter() {
			let expected_result: CandidateMetadata<BalanceOf<T>> =
				*(expected_state.get(&account).ok_or("Not Expected CandidateMetadata")?);
			assert_eq!(expected_result, actual_result);
		}
		Ok(())
	}
}

impl<T> OnRuntimeUpgrade for ReplaceParachainStakingStorage<T>
where
	T: frame_system::Config + pallet_parachain_staking::Config,
	BalanceOf<T>: From<u128>,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		let delegator_state_vec = Self::pre_upgrade_delegator_state_storage()?;
		let candidate_info_vec = Self::pre_upgrade_candidate_info_storage()?;
		Ok((delegator_state_vec, candidate_info_vec).encode())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let mut weight = frame_support::weights::Weight::from_parts(0, 0);
		weight += Self::replace_delegator_state_storage();
		weight += Self::replace_candidate_info_storage();

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		// let pre_vec = (Vec<u8>, Vec<u8>)::decode(&mut &state);
		let pre_vec: (Vec<u8>, Vec<u8>) =
			Decode::decode(&mut &state[..]).map_err(|_| "Failed to decode Tuple")?;
		// let pre_vec: (Vec<u8>, Vec<u8>) = state.decode();
		let _ = Self::post_upgrade_delegator_state_storage(pre_vec.0)?;
		let _ = Self::post_upgrade_candidate_info_storage(pre_vec.1)?;
		Ok(())
	}
}
