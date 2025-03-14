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

use frame_support::{migration::storage_key_iter, pallet_prelude::*, Twox64Concat};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_bounties::{Bounties, BountyIndex, BountyStatus};
use pallet_treasury::BalanceOf;
use parity_scale_codec::EncodeLike;
use sp_runtime::Saturating;

pub const DECIMAL_CONVERTOR: u128 = 1_000_000u128;

// We are creating the exact same struct from the bounties pallet because the fields are private in
// the upstream code
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Bounty<AccountId, Balance, BlockNumber> {
	/// The account proposing it.
	pub proposer: AccountId,
	/// The (total) amount that should be paid if the bounty is rewarded.
	pub value: Balance,
	/// The curator fee. Included in value.
	pub fee: Balance,
	/// The deposit of curator.
	pub curator_deposit: Balance,
	/// The amount held on deposit (reserved) for making this proposal.
	pub bond: Balance,
	/// The status of this bounty.
	pub status: BountyStatus<AccountId, BlockNumber>,
}

// This is important when we want to insert into the storage item
impl<AccountId, Balance, BlockNumber>
	EncodeLike<pallet_bounties::Bounty<AccountId, Balance, BlockNumber>>
	for Bounty<AccountId, Balance, BlockNumber>
where
	AccountId: EncodeLike<AccountId>,
	Balance: EncodeLike<Balance>,
	BlockNumber: EncodeLike<BlockNumber>,
{
}

pub struct ReplacePalletBountyStorage<T, I = ()>(PhantomData<(T, I)>);
impl<T, I: 'static> OnRuntimeUpgrade for ReplacePalletBountyStorage<T, I>
where
	T: pallet_bounties::Config<I> + pallet_treasury::Config,
	BalanceOf<T, I>: EncodeLike<BalanceOf<T, I>> + From<u128>,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		let pallet_prefix: &[u8] = b"Bounties";
		let storage_item_prefix: &[u8] = b"Bounties";
		let stored_data: Vec<_> = storage_key_iter::<
			BountyIndex,
			Bounty<T::AccountId, BalanceOf<T, I>, BlockNumberFor<T>>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.collect();

		let result: Vec<_> = stored_data
			.into_iter()
			.map(|(bounty_index, bounty)| {
				let mut new_bounty = bounty;
				new_bounty.value = new_bounty.value.saturating_mul(DECIMAL_CONVERTOR.into());
				new_bounty.fee = new_bounty.fee.saturating_mul(DECIMAL_CONVERTOR.into());
				new_bounty.curator_deposit =
					new_bounty.curator_deposit.saturating_mul(DECIMAL_CONVERTOR.into());
				new_bounty.bond = new_bounty.bond.saturating_mul(DECIMAL_CONVERTOR.into());

				(bounty_index, new_bounty)
			})
			.collect();

		log::info!(
			target: "ReplacePalletBountyStorage",
			"Finished performing pre upgrade checks"
		);

		Ok(result.encode())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplacePalletBountyStorage",
			"running migration to Bounties Bounties Storage Item"
		);
		let pallet_prefix: &[u8] = b"Bounties";
		let storage_item_prefix: &[u8] = b"Bounties";
		let mut weight: Weight = frame_support::weights::Weight::zero();

		for (bounty_index, mut bounty) in storage_key_iter::<
			BountyIndex,
			Bounty<T::AccountId, BalanceOf<T, I>, BlockNumberFor<T>>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.drain()
		{
			bounty.value = bounty.value.saturating_mul(DECIMAL_CONVERTOR.into());
			bounty.fee = bounty.fee.saturating_mul(DECIMAL_CONVERTOR.into());
			bounty.curator_deposit =
				bounty.curator_deposit.saturating_mul(DECIMAL_CONVERTOR.into());
			bounty.bond = bounty.bond.saturating_mul(DECIMAL_CONVERTOR.into());

			<Bounties<T, I>>::insert(bounty_index, bounty);

			weight += T::DbWeight::get().reads_writes(1, 1);
		}

		log::info!(
			target: "ReplacePalletBountyStorage",
			"Finished performing storage migrations"
		);

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_result = Vec::<(
			BountyIndex,
			Bounty<T::AccountId, BalanceOf<T, I>, BlockNumberFor<T>>,
		)>::decode(&mut &state[..])
		.map_err(|_| "Failed to decode Bounties")?;

		let pallet_prefix: &[u8] = b"Bounties";
		let storage_item_prefix: &[u8] = b"Bounties";
		let actual_result: Vec<_> = storage_key_iter::<
			BountyIndex,
			Bounty<T::AccountId, BalanceOf<T, I>, BlockNumberFor<T>>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.collect();

		for x in 0..actual_result.len() {
			assert_eq!(actual_result[x], expected_result[x])
		}

		log::info!(
			target: "ReplacePalletBountyStorage",
			"Finished performing post upgrade checks"
		);

		Ok(())
	}
}
