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
	traits::{Currency, Get, OnRuntimeUpgrade},
	Blake2_128Concat,
};
use sp_runtime::Saturating;
use sp_std::{marker::PhantomData, vec::Vec};

pub const DECIMAL_CONVERTOR: u32 = 1_000_000;

#[cfg(feature = "try-runtime")]
use parity_scale_codec::Encode;
#[cfg(feature = "try-runtime")]
use sp_std::collections::btree_map::BTreeMap;

use pallet_vesting::{MaxVestingSchedulesGet, Vesting, VestingInfo};
type BalanceOf<T> = <<T as pallet_vesting::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

pub struct ReplacePalletVestingStorage<T>(PhantomData<T>);

impl<T> ReplacePalletVestingStorage<T>
where
	T: pallet_vesting::Config,
{
	// pallet_vesting
	pub fn replace_vesting_vesting_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplacePalletVestingStorage",
			"Running migration to Vesting - Vesting"
		);

		let mut weight = frame_support::weights::Weight::zero();

		let pallet_prefix: &[u8] = b"Vesting";
		let storage_item_prefix: &[u8] = b"Vesting";

		for (account, mut vest_info) in storage_key_iter::<
			T::AccountId,
			BoundedVec<VestingInfo<BalanceOf<T>, T::BlockNumber>, MaxVestingSchedulesGet<T>>,
			Blake2_128Concat,
		>(pallet_prefix, storage_item_prefix)
		.drain()
		{
			for vest in vest_info.iter_mut() {
				*vest = VestingInfo::new(
					vest.locked().saturating_mul(DECIMAL_CONVERTOR.into()),
					vest.per_block().saturating_mul(DECIMAL_CONVERTOR.into()),
					vest.starting_block(),
				);
			}

			Vesting::<T>::insert(&account, vest_info);
			weight += T::DbWeight::get().reads_writes(1, 1);
		}

		weight
	}
}

#[cfg(feature = "try-runtime")]
impl<T> ReplacePalletVestingStorage<T>
where
	T: pallet_vesting::Config,
{
	// pallet_vesting
	pub fn pre_upgrade_vesting_vesting_storage() -> Result<Vec<u8>, &'static str> {
		let result: BTreeMap<
			T::AccountId,
			BoundedVec<VestingInfo<BalanceOf<T>, T::BlockNumber>, MaxVestingSchedulesGet<T>>,
		> = <Vesting<T>>::iter()
			.map(|(account, vest_vec)| {
				let mut new_vest_vec: BoundedVec<
					VestingInfo<BalanceOf<T>, T::BlockNumber>,
					MaxVestingSchedulesGet<T>,
				> = vest_vec;
				for vest in new_vest_vec.iter_mut() {
					*vest = VestingInfo::new(
						vest.locked().saturating_mul(DECIMAL_CONVERTOR.into()),
						vest.per_block().saturating_mul(DECIMAL_CONVERTOR.into()),
						vest.starting_block(),
					);
				}
				(account, new_vest_vec)
			})
			.collect();
		Ok(result.encode())
	}

	pub fn post_upgrade_vesting_vesting_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_state = BTreeMap::<
			T::AccountId,
			BoundedVec<VestingInfo<BalanceOf<T>, T::BlockNumber>, MaxVestingSchedulesGet<T>>,
		>::decode(&mut &state[..])
		.map_err(|_| "Failed to decode BoundedVec<VestingInfo>")?;
		for (account, actual_result) in <Vesting<T>>::iter() {
			let expected_result: BoundedVec<
				VestingInfo<BalanceOf<T>, T::BlockNumber>,
				MaxVestingSchedulesGet<T>,
			> = expected_state
				.get(&account)
				.ok_or("Not Expected BoundedVec<VestingInfo>")?
				.clone();
			assert_eq!(expected_result.encode(), actual_result.encode());
		}
		Ok(())
	}
}

impl<T> OnRuntimeUpgrade for ReplacePalletVestingStorage<T>
where
	T: pallet_vesting::Config,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		// pallet_vesting
		let vesting_vec = Self::pre_upgrade_vesting_vesting_storage()?;

		log::info!(
			target: "ReplacePalletVestingStorage",
			"Finished performing Vesting pre upgrade checks"
		);

		Ok((vesting_vec).encode())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let mut weight = frame_support::weights::Weight::zero();
		// pallet_vesting
		weight += Self::replace_vesting_vesting_storage();

		log::info!(
			target: "ReplacePalletVestingStorage",
			"Finished performing Vesting storage migration"
		);

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		let pre_vec: (Vec<u8>,) =
			Decode::decode(&mut &state[..]).map_err(|_| "Failed to decode Tuple")?;
		Self::post_upgrade_vesting_vesting_storage(pre_vec.0)?;

		log::info!(
			target: "ReplacePalletVestingStorage",
			"Finished performing Vesting post upgrade checks"
		);

		Ok(())
	}
}
