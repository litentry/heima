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
use frame_support::traits::{Get, OnRuntimeUpgrade};
use sp_std::{marker::PhantomData, vec::Vec};

use frame_support::{
	migration::{get_storage_value, storage_key_iter},
	pallet_prelude::*,
	Twox64Concat,
};
use pallet_treasury::{BalanceOf, Deactivated, ProposalIndex, Proposals};
use parity_scale_codec::EncodeLike;
use sp_runtime::Saturating;

pub const DECIMAL_CONVERTOR: u128 = 1_000_000u128;

// We are recreating the proposal struct with public fields
#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub struct Proposal<AccountId, Balance> {
	/// The account proposing it.
	pub proposer: AccountId,
	/// The (total) amount that should be paid if the proposal is accepted.
	pub value: Balance,
	/// The account to whom the payment should be made if the proposal is accepted.
	pub beneficiary: AccountId,
	/// The amount held on deposit (reserved) for making this proposal.
	pub bond: Balance,
}

// This is important when we want to insert into the storage item
impl<AccountId, Balance> EncodeLike<pallet_treasury::Proposal<AccountId, Balance>>
	for Proposal<AccountId, Balance>
where
	AccountId: EncodeLike<AccountId>,
	Balance: EncodeLike<Balance>,
{
}

pub struct ReplaceTreasuryStorage<T, I = ()>(PhantomData<(T, I)>);

impl<T, I: 'static> ReplaceTreasuryStorage<T, I>
where
	T: pallet_treasury::Config<I>,
	BalanceOf<T, I>: EncodeLike<BalanceOf<T, I>> + From<u128>,
{
	fn replace_proposals_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceTreasuryStorage",
			"running migration to Treasury Proposals Storage Item"
		);
		let pallet_prefix: &[u8] = b"Treasury";
		let storage_item_prefix: &[u8] = b"Proposals";
		let mut weight: Weight = frame_support::weights::Weight::zero();

		for (proposal_index, mut proposal) in storage_key_iter::<
			ProposalIndex,
			Proposal<T::AccountId, BalanceOf<T, I>>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.drain()
		{
			proposal.value = proposal.value.saturating_mul(DECIMAL_CONVERTOR.into());
			proposal.bond = proposal.bond.saturating_mul(DECIMAL_CONVERTOR.into());

			<Proposals<T, I>>::insert(proposal_index, proposal);

			weight += T::DbWeight::get().reads_writes(1, 1);
		}
		weight
	}

	fn replace_deactivated_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceTreasuryStorage",
			"running migration to Treasury Deactivated Storage Item"
		);
		let pallet_prefix: &[u8] = b"Treasury";
		let storage_item_prefix: &[u8] = b"Deactivated";
		let stored_data =
			get_storage_value::<BalanceOf<T, I>>(pallet_prefix, storage_item_prefix, b"")
				.expect("Storage query fails: Treasury Deactivated");

		<Deactivated<T, I>>::put(stored_data.saturating_mul(DECIMAL_CONVERTOR.into()));

		let weight = T::DbWeight::get();
		frame_support::weights::Weight::from_parts(0, weight.write + weight.read)
	}
}

#[cfg(feature = "try-runtime")]
impl<T, I: 'static> ReplaceTreasuryStorage<T, I>
where
	T: pallet_treasury::Config<I>,
	BalanceOf<T, I>: EncodeLike<BalanceOf<T, I>> + From<u128>,
{
	fn pre_upgrade_proposals_storage() -> Result<Vec<u8>, &'static str> {
		let pallet_prefix: &[u8] = b"Treasury";
		let storage_item_prefix: &[u8] = b"Proposals";
		let stored_data: Vec<_> = storage_key_iter::<
			ProposalIndex,
			Proposal<T::AccountId, BalanceOf<T, I>>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.collect();

		let result: Vec<_> = stored_data
			.into_iter()
			.map(|(proposal_index, proposal)| {
				let mut new_proposal = proposal;
				new_proposal.value = new_proposal.value.saturating_mul(DECIMAL_CONVERTOR.into());
				new_proposal.bond = new_proposal.bond.saturating_mul(DECIMAL_CONVERTOR.into());

				(proposal_index, new_proposal)
			})
			.collect();

		Ok(result.encode())
	}

	fn post_upgrade_proposals_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_result =
			Vec::<(ProposalIndex, Proposal<T::AccountId, BalanceOf<T, I>>)>::decode(
				&mut &state[..],
			)
			.map_err(|_| "Failed to decode Bounties")?;

		let pallet_prefix: &[u8] = b"Treasury";
		let storage_item_prefix: &[u8] = b"Proposals";
		let actual_result: Vec<_> = storage_key_iter::<
			ProposalIndex,
			Proposal<T::AccountId, BalanceOf<T, I>>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.collect();

		for x in 0..actual_result.len() {
			assert_eq!(actual_result[x], expected_result[x])
		}

		Ok(())
	}

	fn pre_upgrade_deactivated_storage() -> Result<Vec<u8>, &'static str> {
		Ok(<Deactivated<T, I>>::get().saturating_mul(DECIMAL_CONVERTOR.into()).encode())
	}

	fn post_upgrade_deactivated_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_state = BalanceOf::<T, I>::decode(&mut &state[..])
			.map_err(|_| "Failed to decode Total Balance")?;
		let actual_state = <Deactivated<T, I>>::get();
		assert_eq!(expected_state, actual_state);
		Ok(())
	}
}

impl<T, I: 'static> OnRuntimeUpgrade for ReplaceTreasuryStorage<T, I>
where
	T: pallet_treasury::Config<I>,
	BalanceOf<T, I>: EncodeLike<BalanceOf<T, I>> + From<u128>,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		let proposals_state_vec = Self::pre_upgrade_proposals_storage()?;
		let deactivated_state_vec = Self::pre_upgrade_deactivated_storage()?;

		log::info!(
			target: "ReplaceTreasuryStorage",
			"Finished performing post upgrade checks"
		);
		Ok((proposals_state_vec, deactivated_state_vec).encode())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let mut weight = frame_support::weights::Weight::from_parts(0, 0);
		weight += Self::replace_proposals_storage();
		weight += Self::replace_deactivated_storage();

		log::info!(
			target: "ReplaceTreasuryStorage",
			"Finished performing storage migration"
		);

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		let pre_vec: (Vec<u8>, Vec<u8>) =
			Decode::decode(&mut &state[..]).map_err(|_| "Failed to decode Tuple")?;
		Self::post_upgrade_proposals_storage(pre_vec.0)?;
		Self::post_upgrade_deactivated_storage(pre_vec.1)?;
		log::info!(target: "ReplaceTreasuryStorage", "Finished performing post upgrade checks");
		Ok(())
	}
}
