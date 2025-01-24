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
	Blake2_128Concat, Twox64Concat,
};
use sp_runtime::Saturating;
use sp_std::{convert::From, marker::PhantomData, vec::Vec};

use pallet_parachain_staking::{
	set::OrderedSet, BalanceOf, Bond, BottomDelegations, CandidateInfo, CandidateMetadata,
	CandidatePool, DelayedPayout, DelayedPayouts, DelegationAction, DelegationScheduledRequests,
	Delegations, Delegator, DelegatorState, ScheduledRequest, Staked, TopDelegations, Total,
};
pub const DECIMAL_CONVERTOR: u128 = 1_000_000u128;

#[cfg(feature = "try-runtime")]
use parity_scale_codec::Encode;
#[cfg(feature = "try-runtime")]
use sp_std::collections::btree_map::BTreeMap;
use storage::migration::get_storage_value;

// Replace Parachain Staking Storage for Decimal Change from 12 to 18
pub struct ReplaceParachainStakingStorage<T>(PhantomData<T>);
impl<T: pallet_parachain_staking::Config> ReplaceParachainStakingStorage<T>
where
	BalanceOf<T>: From<u128>,
{
	pub fn replace_delegator_state_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceParachainStakingStorage",
			"running migration to ParachainStaking DelegatorState"
		);
		let pallet_prefix: &[u8] = b"ParachainStaking";
		let storage_item_prefix: &[u8] = b"DelegatorState";
		let mut weight: Weight = frame_support::weights::Weight::zero();

		for (account, mut delegator) in storage_key_iter::<
			T::AccountId,
			Delegator<T::AccountId, BalanceOf<T>>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.drain()
		{
			delegator.total = delegator.total.saturating_mul(DECIMAL_CONVERTOR.into());
			delegator.less_total = delegator.less_total.saturating_mul(DECIMAL_CONVERTOR.into());
			let mut sorted_inner_vector = delegator.delegations.0;
			for elem in sorted_inner_vector.iter_mut() {
				elem.amount = elem.amount.saturating_mul(DECIMAL_CONVERTOR.into());
			}
			delegator.delegations = OrderedSet::from(sorted_inner_vector);

			<DelegatorState<T>>::insert(&account, delegator);
			weight += T::DbWeight::get().reads_writes(1, 1);
		}

		weight
	}

	pub fn replace_candidate_info_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceParachainStakingStorage",
			"running migration to ParachainStaking CandidateInfo"
		);
		let pallet_prefix: &[u8] = b"ParachainStaking";
		let storage_item_prefix: &[u8] = b"CandidateInfo";
		let mut weight: Weight = frame_support::weights::Weight::zero();

		for (account, mut metadata) in storage_key_iter::<
			T::AccountId,
			CandidateMetadata<BalanceOf<T>>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.drain()
		{
			metadata.bond = metadata.bond.saturating_mul(DECIMAL_CONVERTOR.into());
			metadata.total_counted =
				metadata.total_counted.saturating_mul(DECIMAL_CONVERTOR.into());
			metadata.lowest_top_delegation_amount =
				metadata.lowest_top_delegation_amount.saturating_mul(DECIMAL_CONVERTOR.into());
			metadata.highest_bottom_delegation_amount = metadata
				.highest_bottom_delegation_amount
				.saturating_mul(DECIMAL_CONVERTOR.into());
			metadata.lowest_bottom_delegation_amount = metadata
				.lowest_bottom_delegation_amount
				.saturating_mul(DECIMAL_CONVERTOR.into());

			if let Some(mut i) = metadata.request {
				i.amount = i.amount.saturating_mul(DECIMAL_CONVERTOR.into());
			}
			<CandidateInfo<T>>::insert(&account, metadata);
			weight += T::DbWeight::get().reads_writes(1, 1);
		}

		weight
	}

	pub fn replace_delegation_scheduled_requests_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceParachainStakingStorage",
			"running migration to ParachainStaking DelegationScheduledRequests"
		);
		let pallet_prefix: &[u8] = b"ParachainStaking";
		let storage_item_prefix: &[u8] = b"DelegationScheduledRequests";
		let mut weight: Weight = frame_support::weights::Weight::zero();

		for (account, mut scheduled_requests) in storage_key_iter::<
			T::AccountId,
			Vec<ScheduledRequest<T::AccountId, BalanceOf<T>>>,
			Blake2_128Concat,
		>(pallet_prefix, storage_item_prefix)
		.drain()
		{
			for scheduled_request in scheduled_requests.iter_mut() {
				match scheduled_request.action {
					DelegationAction::Revoke(n) => {
						scheduled_request.action =
							DelegationAction::Revoke(n.saturating_mul(DECIMAL_CONVERTOR.into()));
					},
					DelegationAction::Decrease(n) => {
						scheduled_request.action =
							DelegationAction::Decrease(n.saturating_mul(DECIMAL_CONVERTOR.into()));
					},
				}
			}
			<DelegationScheduledRequests<T>>::insert(&account, scheduled_requests);
			weight += T::DbWeight::get().reads_writes(1, 1);
		}
		weight
	}

	pub fn replace_top_delegations_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceParachainStakingStorage",
			"running migration to ParachainStaking TopDelegations"
		);
		let pallet_prefix: &[u8] = b"ParachainStaking";
		let storage_item_prefix: &[u8] = b"TopDelegations";
		let mut weight: Weight = frame_support::weights::Weight::zero();

		for (account, mut delegations) in storage_key_iter::<
			T::AccountId,
			Delegations<T::AccountId, BalanceOf<T>>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.drain()
		{
			for delegation_bond in delegations.delegations.iter_mut() {
				delegation_bond.amount =
					delegation_bond.amount.saturating_mul(DECIMAL_CONVERTOR.into());
			}

			<TopDelegations<T>>::insert(&account, delegations);
			weight += T::DbWeight::get().reads_writes(1, 1);
		}
		weight
	}

	pub fn replace_bottom_delegations_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceParachainStakingStorage",
			"running migration to ParachainStaking BottomDelegations"
		);
		let pallet_prefix: &[u8] = b"ParachainStaking";
		let storage_item_prefix: &[u8] = b"BottomDelegations";
		let mut weight: Weight = frame_support::weights::Weight::zero();

		for (account, mut delegations) in storage_key_iter::<
			T::AccountId,
			Delegations<T::AccountId, BalanceOf<T>>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.drain()
		{
			for delegation_bond in delegations.delegations.iter_mut() {
				delegation_bond.amount =
					delegation_bond.amount.saturating_mul(DECIMAL_CONVERTOR.into());
			}

			<BottomDelegations<T>>::insert(&account, delegations);
			weight += T::DbWeight::get().reads_writes(1, 1);
		}

		weight
	}

	pub fn replace_total_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceParachainStakingStorage",
			"running migration to ParachainStaking Total"
		);
		let pallet_prefix: &[u8] = b"ParachainStaking";
		let storage_item_prefix: &[u8] = b"Total";
		let stored_data =
			get_storage_value::<BalanceOf<T>>(pallet_prefix, storage_item_prefix, b"")
				.expect("Storage query fails: ParachainStaking Total");
		<Total<T>>::put(stored_data.saturating_mul(DECIMAL_CONVERTOR.into()));
		let weight = T::DbWeight::get();
		frame_support::weights::Weight::from_parts(0, weight.write + weight.read)
	}

	pub fn replace_candidate_pool_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceParachainStakingStorage",
			"running migration to ParachainStaking CandidatePool"
		);
		let pallet_prefix: &[u8] = b"ParachainStaking";
		let storage_item_prefix: &[u8] = b"CandidatePool";
		let mut stored_data = get_storage_value::<OrderedSet<Bond<T::AccountId, BalanceOf<T>>>>(
			pallet_prefix,
			storage_item_prefix,
			b"",
		)
		.expect("Storage query fails: ParachainStaking CandidatePool");
		for bond in stored_data.0.iter_mut() {
			bond.amount = bond.amount.saturating_mul(DECIMAL_CONVERTOR.into());
		}
		<CandidatePool<T>>::put(stored_data);
		let weight = T::DbWeight::get();
		frame_support::weights::Weight::from_parts(0, weight.write + weight.read)
	}

	pub fn replace_delayed_payouts_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceParachainStakingStorage",
			"running migration to ParachainStaking DelayedPayouts"
		);
		let pallet_prefix: &[u8] = b"ParachainStaking";
		let storage_item_prefix: &[u8] = b"DelayedPayouts";
		let mut weight: Weight = frame_support::weights::Weight::zero();

		for (round, mut delayed_payout) in storage_key_iter::<
			u32,
			DelayedPayout<BalanceOf<T>>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.drain()
		{
			delayed_payout.round_issuance =
				delayed_payout.round_issuance.saturating_mul(DECIMAL_CONVERTOR.into());
			delayed_payout.total_staking_reward =
				delayed_payout.total_staking_reward.saturating_mul(DECIMAL_CONVERTOR.into());

			<DelayedPayouts<T>>::insert(round, delayed_payout);
			weight += T::DbWeight::get().reads_writes(1, 1);
		}
		weight
	}

	pub fn replace_staked_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceParachainStakingStorage",
			"running migration to ParachainStaking Staked"
		);
		let pallet_prefix: &[u8] = b"ParachainStaking";
		let storage_item_prefix: &[u8] = b"Staked";

		let mut weight: Weight = frame_support::weights::Weight::zero();

		for (round, staked) in
			storage_key_iter::<u32, BalanceOf<T>, Twox64Concat>(pallet_prefix, storage_item_prefix)
				.drain()
		{
			<Staked<T>>::insert(round, staked.saturating_mul(DECIMAL_CONVERTOR.into()));
			weight += T::DbWeight::get().reads_writes(1, 1);
		}
		weight
	}
}

#[cfg(feature = "try-runtime")]
impl<T: pallet_parachain_staking::Config> ReplaceParachainStakingStorage<T>
where
	BalanceOf<T>: From<u128>,
{
	pub fn pre_upgrade_delegator_state_storage() -> Result<Vec<u8>, &'static str> {
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
		for (account, actual_result) in <DelegatorState<T>>::iter() {
			let expected_result: Delegator<T::AccountId, BalanceOf<T>> =
				expected_state.get(&account).ok_or("Not Expected Delegator")?.clone();
			assert_eq!(expected_result, actual_result);
		}
		Ok(())
	}
	pub fn pre_upgrade_candidate_info_storage() -> Result<Vec<u8>, &'static str> {
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
		for (account, actual_result) in <CandidateInfo<T>>::iter() {
			let expected_result: CandidateMetadata<BalanceOf<T>> =
				expected_state.get(&account).ok_or("Not Expected CandidateMetadata")?.clone();
			// Can not compare CandidateMetadata so compare its encode
			assert_eq!(expected_result.encode(), actual_result.encode());
		}
		Ok(())
	}
	pub fn pre_upgrade_delegation_scheduled_requests_storage() -> Result<Vec<u8>, &'static str> {
		let result: BTreeMap<T::AccountId, Vec<ScheduledRequest<T::AccountId, BalanceOf<T>>>> =
			<DelegationScheduledRequests<T>>::iter()
				.map(|(account, state)| {
					let mut new_scheduled_requests: Vec<
						ScheduledRequest<T::AccountId, BalanceOf<T>>,
					> = state;
					for scheduled_request in new_scheduled_requests.iter_mut() {
						match scheduled_request.action {
							DelegationAction::Revoke(n) => {
								scheduled_request.action = DelegationAction::Revoke(
									n.saturating_mul(DECIMAL_CONVERTOR.into()),
								);
							},
							DelegationAction::Decrease(n) => {
								scheduled_request.action = DelegationAction::Decrease(
									n.saturating_mul(DECIMAL_CONVERTOR.into()),
								);
							},
						}
					}

					(account, new_scheduled_requests)
				})
				.collect();
		Ok(result.encode())
	}
	pub fn post_upgrade_delegation_scheduled_requests_storage(
		state: Vec<u8>,
	) -> Result<(), &'static str> {
		let expected_state = BTreeMap::<
			T::AccountId,
			Vec<ScheduledRequest<T::AccountId, BalanceOf<T>>>,
		>::decode(&mut &state[..])
		.map_err(|_| "Failed to decode Vec<ScheduledRequest>")?;
		for (account, actual_result) in <DelegationScheduledRequests<T>>::iter() {
			let expected_result: Vec<ScheduledRequest<T::AccountId, BalanceOf<T>>> = expected_state
				.get(&account)
				.ok_or("Not Expected Vec<ScheduledRequest>")?
				.clone();
			// Can not compare Vec<ScheduledRequest> so compare its encode
			assert_eq!(expected_result.encode(), actual_result.encode());
		}
		Ok(())
	}
	pub fn pre_upgrade_top_delegations_storage() -> Result<Vec<u8>, &'static str> {
		let result: BTreeMap<T::AccountId, Delegations<T::AccountId, BalanceOf<T>>> =
			<TopDelegations<T>>::iter()
				.map(|(account, state)| {
					let mut new_delegations: Delegations<T::AccountId, BalanceOf<T>> = state;

					for delegation_bond in new_delegations.delegations.iter_mut() {
						delegation_bond.amount =
							delegation_bond.amount.saturating_mul(DECIMAL_CONVERTOR.into());
					}

					(account, new_delegations)
				})
				.collect();
		Ok(result.encode())
	}
	pub fn post_upgrade_top_delegations_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_state =
			BTreeMap::<T::AccountId, Delegations<T::AccountId, BalanceOf<T>>>::decode(
				&mut &state[..],
			)
			.map_err(|_| "Failed to decode Delegations")?;
		for (account, actual_result) in <TopDelegations<T>>::iter() {
			let expected_result: Delegations<T::AccountId, BalanceOf<T>> =
				expected_state.get(&account).ok_or("Not Expected Delegations")?.clone();
			assert_eq!(expected_result.encode(), actual_result.encode());
		}
		Ok(())
	}
	pub fn pre_upgrade_bottom_delegations_storage() -> Result<Vec<u8>, &'static str> {
		let result: BTreeMap<T::AccountId, Delegations<T::AccountId, BalanceOf<T>>> =
			<BottomDelegations<T>>::iter()
				.map(|(account, state)| {
					let mut new_delegations: Delegations<T::AccountId, BalanceOf<T>> = state;

					for delegation_bond in new_delegations.delegations.iter_mut() {
						delegation_bond.amount =
							delegation_bond.amount.saturating_mul(DECIMAL_CONVERTOR.into());
					}

					(account, new_delegations)
				})
				.collect();
		Ok(result.encode())
	}
	pub fn post_upgrade_bottom_delegations_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_state =
			BTreeMap::<T::AccountId, Delegations<T::AccountId, BalanceOf<T>>>::decode(
				&mut &state[..],
			)
			.map_err(|_| "Failed to decode Delegations")?;
		for (account, actual_result) in <BottomDelegations<T>>::iter() {
			let expected_result: Delegations<T::AccountId, BalanceOf<T>> =
				expected_state.get(&account).ok_or("Not Expected Delegations")?.clone();
			assert_eq!(expected_result.encode(), actual_result.encode());
		}
		Ok(())
	}
	pub fn pre_upgrade_total_storage() -> Result<Vec<u8>, &'static str> {
		Ok(<Total<T>>::get().saturating_mul(DECIMAL_CONVERTOR.into()).encode())
	}
	pub fn post_upgrade_total_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_state = BalanceOf::<T>::decode(&mut &state[..])
			.map_err(|_| "Failed to decode Total Balance")?;
		let actual_state = <Total<T>>::get();
		assert_eq!(expected_state, actual_state);
		Ok(())
	}
	pub fn pre_upgrade_candidate_pool_storage() -> Result<Vec<u8>, &'static str> {
		let result: BTreeMap<T::AccountId, BalanceOf<T>> = <CandidatePool<T>>::get()
			.0
			.iter()
			.map(|bond| {
				let mut new_bond: Bond<T::AccountId, BalanceOf<T>> = bond.clone();
				new_bond.amount = new_bond.amount.saturating_mul(DECIMAL_CONVERTOR.into());
				(new_bond.owner, new_bond.amount)
			})
			.collect();
		Ok(result.encode())
	}
	pub fn post_upgrade_candidate_pool_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_state = BTreeMap::<T::AccountId, BalanceOf<T>>::decode(&mut &state[..])
			.map_err(|_| "Failed to decode Candidate Pool Bond (owner, amount)")?;
		let actual_state: BTreeMap<T::AccountId, BalanceOf<T>> = <CandidatePool<T>>::get()
			.0
			.iter()
			.map(|bond| (bond.owner.clone(), bond.amount))
			.collect();
		assert_eq!(expected_state.encode(), actual_state.encode());
		Ok(())
	}
	pub fn pre_upgrade_delayed_payouts_storage() -> Result<Vec<u8>, &'static str> {
		let result: BTreeMap<u32, DelayedPayout<BalanceOf<T>>> = <DelayedPayouts<T>>::iter()
			.map(|(round, state)| {
				let mut new_delayed_payout: DelayedPayout<BalanceOf<T>> = state;

				new_delayed_payout.round_issuance =
					new_delayed_payout.round_issuance.saturating_mul(DECIMAL_CONVERTOR.into());
				new_delayed_payout.total_staking_reward = new_delayed_payout
					.total_staking_reward
					.saturating_mul(DECIMAL_CONVERTOR.into());

				(round, new_delayed_payout)
			})
			.collect();
		Ok(result.encode())
	}
	pub fn post_upgrade_delayed_payouts_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_state = BTreeMap::<u32, DelayedPayout<BalanceOf<T>>>::decode(&mut &state[..])
			.map_err(|_| "Failed to decode Delayed Payouts")?;
		for (round, actual_result) in <DelayedPayouts<T>>::iter() {
			let expected_result: DelayedPayout<BalanceOf<T>> =
				expected_state.get(&round).ok_or("Not Expected DelayedPayout")?.clone();
			assert_eq!(expected_result.encode(), actual_result.encode());
		}
		Ok(())
	}
	pub fn pre_upgrade_staked_storage() -> Result<Vec<u8>, &'static str> {
		let result: BTreeMap<u32, BalanceOf<T>> = <Staked<T>>::iter()
			.map(|(round, state)| {
				let new_staked: BalanceOf<T> = state;
				(round, new_staked.saturating_mul(DECIMAL_CONVERTOR.into()))
			})
			.collect();
		Ok(result.encode())
	}
	pub fn post_upgrade_staked_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_state = BTreeMap::<u32, BalanceOf<T>>::decode(&mut &state[..])
			.map_err(|_| "Failed to decode Staked")?;
		for (round, actual_result) in <Staked<T>>::iter() {
			let expected_result: BalanceOf<T> =
				*expected_state.get(&round).ok_or("Not Expected DelayedPayout")?;
			assert_eq!(expected_result.encode(), actual_result.encode());
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
		let delegation_scheduled_requests_vec =
			Self::pre_upgrade_delegation_scheduled_requests_storage()?;
		let top_delegations_vec = Self::pre_upgrade_top_delegations_storage()?;
		let bottom_delegations_vec = Self::pre_upgrade_bottom_delegations_storage()?;
		let total_vec = Self::pre_upgrade_total_storage()?;
		let candidate_pool_vec = Self::pre_upgrade_candidate_pool_storage()?;
		let delayed_payouts_vec = Self::pre_upgrade_delayed_payouts_storage()?;
		let staked_vec = Self::pre_upgrade_staked_storage()?;
		Ok((
			delegator_state_vec,
			candidate_info_vec,
			delegation_scheduled_requests_vec,
			top_delegations_vec,
			bottom_delegations_vec,
			total_vec,
			candidate_pool_vec,
			delayed_payouts_vec,
			staked_vec,
		)
			.encode())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let mut weight = frame_support::weights::Weight::from_parts(0, 0);
		weight += Self::replace_delegator_state_storage();
		weight += Self::replace_candidate_info_storage();
		weight += Self::replace_delegation_scheduled_requests_storage();
		weight += Self::replace_top_delegations_storage();
		weight += Self::replace_bottom_delegations_storage();
		weight += Self::replace_total_storage();
		weight += Self::replace_candidate_pool_storage();

		// No need for AtStake Migration since this is a snapshot, everything is good as long as it
		// will not change proportion AtStake

		weight += Self::replace_delayed_payouts_storage();
		// Staked Storage holds limited amount of recent rounds only, should not cause large PoV
		weight += Self::replace_staked_storage();

		// No need since all balance related config is Zero
		// InflationConfig

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		let pre_vec: (
			Vec<u8>,
			Vec<u8>,
			Vec<u8>,
			Vec<u8>,
			Vec<u8>,
			Vec<u8>,
			Vec<u8>,
			Vec<u8>,
			Vec<u8>,
		) = Decode::decode(&mut &state[..]).map_err(|_| "Failed to decode Tuple")?;
		Self::post_upgrade_delegator_state_storage(pre_vec.0)?;
		Self::post_upgrade_candidate_info_storage(pre_vec.1)?;
		Self::post_upgrade_delegation_scheduled_requests_storage(pre_vec.2)?;
		Self::post_upgrade_top_delegations_storage(pre_vec.3)?;
		Self::post_upgrade_bottom_delegations_storage(pre_vec.4)?;
		Self::post_upgrade_total_storage(pre_vec.5)?;
		Self::post_upgrade_candidate_pool_storage(pre_vec.6)?;
		Self::post_upgrade_delayed_payouts_storage(pre_vec.7)?;
		Self::post_upgrade_staked_storage(pre_vec.8)?;
		Ok(())
	}
}
