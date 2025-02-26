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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use frame_support::{
	pallet_prelude::*,
	traits::{
		fungibles::{Balanced, Create, Mutate, Refund},
		tokens::{Balance, Fortitude, Precision, Preservation},
		AccountTouch,
	},
	PalletId,
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
use pallet_parachain_taking::{Round, RoundIndex, TotalSelected};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_parachain_staking::Config {
		/// Number of rounds of reward recording
		#[pallet::constant]
		type RewardPaymentRecordRound: Get<RoundIndex>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			let mut weight = T::DbWeight::get().reads_writes(1, 1); // Self::state()
			let round = <Round<T>>::get();
			// Clean outdated storage
			let reward_record_length = <RoundAccumulatedReward<T>>::iter_prefix(
				round.saturating_sub(T::RewardPaymentRecordRound::get()),
			)
			.collect::<Vec<_>>()
			.len();
			if reward_record_length > 0 {
				let _ = <RoundAccumulatedReward<T>>::clear_prefix(
					round,
					Some(<TotalSelected<T>>::get()),
					None,
				);
				weight =
					weight.saturating_add(T::DbWeight::get().reads_writes(reward_record_length, 0));
			}
			weight
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn round_accumulated_reward)]
	/// Snapshot of round accumulated reward
	/// Clean up after RewardPaymentDelay
	pub type RoundAccumulatedReward<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		RoundIndex,
		Twox64Concat,
		T::AccountId,
		BalanceOf<T>,
		ValueQuery,
	>;

	impl<T: Config>
		pallet_parachain_staking::TransactionFeeRewardResource<
			BalanceOf<T>,
			RoundIndex,
			T::AccountId,
		> for Pallet<T>
	{
		fn on_transaction_fee_reward_notify(
			amount: BalanceOf<T>,
			collator: T::AccountId,
		) -> Result<(), &str> {
			let round = <Round<T>>::get();
			let cr = <RoundAccumulatedReward<T>>::take(round, &collator);
			<RoundAccumulatedReward<T>>::insert(round, collator, cr.saturating_add(amount));
			Ok(())
		}

		fn query_round_fee_reward(round: RoundIndex, collator: T::AccountId) -> BalanceOf<T> {
			<RoundAccumulatedReward<T>>::get(round, collator)
		}
	}
}
