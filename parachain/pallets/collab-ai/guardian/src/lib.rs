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
//
//! # Guardian Pallet
//!
//! - [`Config`]
//! - [`Call`]
//!
//! ## Overview
//!
//! The Guardian pallet handles the administration of general guardian and guardian voting.
//!
//!
#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{Currency, EnsureOrigin, Get, LockableCurrency, ReservableCurrency},
	transactional,
};
use frame_system::{
	ensure_signed,
	pallet_prelude::{BlockNumberFor, OriginFor},
};
pub use pallet::*;
use pallet_collab_ai_common::*;
use sp_runtime::ArithmeticError;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + Sized {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Currency type for this pallet.
		type Currency: ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId, Moment = BlockNumberFor<Self>>;

		/// The minimum amount to be used as a deposit for a guardian
		#[pallet::constant]
		type MinimumGuardianDeposit: Get<BalanceOf<Self>>;

		/// Origin from guardian legal file verified by
		type GuardianJudgeOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}

	#[pallet::type_value]
	pub fn DefaultForGuardianIndex() -> GuardianIndex {
		1u128
	}

	/// Next guardian index, the number of (public) guardian that have been made so far + 1.
	#[pallet::storage]
	#[pallet::getter(fn public_guardian_count)]
	pub type PublicGuardianCount<T> =
		StorageValue<_, GuardianIndex, ValueQuery, DefaultForGuardianIndex>;

	/// The public guardian to index
	#[pallet::storage]
	#[pallet::getter(fn public_guardian_to_index)]
	pub type PublicGuardianToIndex<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, GuardianIndex, OptionQuery>;

	/// Guardian index to hash and update time. Info Hash is current used guardian legal file hash.
	#[pallet::storage]
	#[pallet::getter(fn guardian_index_to_info)]
	pub type GuardianIndexToInfo<T: Config> = StorageMap<
		_,
		Twox64Concat,
		GuardianIndex,
		(InfoHash, BlockNumberFor<T>, T::AccountId, CandidateStatus),
		OptionQuery,
	>;

	/// Votings for guardian
	#[pallet::storage]
	#[pallet::getter(fn guardian_votes)]
	pub type GuardianVotes<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		GuardianIndex,
		GuardianVote,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		GuardianRegisted {
			guardian: T::AccountId,
			guardian_index: GuardianIndex,
			info_hash: InfoHash,
		},
		GuardianUpdated {
			guardian: T::AccountId,
			guardian_index: GuardianIndex,
			info_hash: InfoHash,
		},
		GuardianCleaned {
			guardian: T::AccountId,
			guardian_index: GuardianIndex,
		},
		GuardianStatusUpdated {
			guardian: T::AccountId,
			guardian_index: GuardianIndex,
			status: CandidateStatus,
		},
		VoteGuardian {
			voter: T::AccountId,
			guardian_index: GuardianIndex,
			guardian: T::AccountId,
			status: Option<GuardianVote>,
		},
		RemoveAllVote {
			voter: T::AccountId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		GuardianAlreadyRegistered,
		GuardianNotRegistered,
		GuardianIndexNotExist,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Registing a guardian legal info
		#[pallet::call_index(0)]
		#[pallet::weight({195_000_000})]
		#[transactional]
		pub fn regist_guardian(origin: OriginFor<T>, info_hash: InfoHash) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure guardian not existing yet
			ensure!(
				!PublicGuardianToIndex::<T>::contains_key(&who),
				Error::<T>::GuardianAlreadyRegistered
			);
			// New registed guardian need to make a balance reserve
			T::Currency::reserve(&who, T::MinimumGuardianDeposit::get())?;

			// Update guardian
			let current_block = frame_system::Pallet::<T>::block_number();
			let next_guardian_index = PublicGuardianCount::<T>::get();

			PublicGuardianToIndex::<T>::insert(&who, next_guardian_index);
			GuardianIndexToInfo::<T>::insert(
				next_guardian_index,
				(info_hash, current_block, who.clone(), CandidateStatus::Unverified),
			);
			PublicGuardianCount::<T>::put(
				next_guardian_index.checked_add(1u32.into()).ok_or(ArithmeticError::Overflow)?,
			);

			Self::deposit_event(Event::GuardianRegisted {
				guardian: who,
				guardian_index: next_guardian_index,
				info_hash,
			});
			Ok(())
		}

		/// Updating a guardian legal info
		#[pallet::call_index(1)]
		#[pallet::weight({195_000_000})]
		pub fn update_guardian(origin: OriginFor<T>, info_hash: InfoHash) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure existing
			let guardian_index =
				PublicGuardianToIndex::<T>::get(&who).ok_or(Error::<T>::GuardianNotRegistered)?;

			// Update guardian
			// But if banned, then require extra reserve
			GuardianIndexToInfo::<T>::try_mutate_exists(
				guardian_index,
				|maybe_info| -> Result<(), DispatchError> {
					let info = maybe_info.as_mut().ok_or(Error::<T>::GuardianIndexNotExist)?;

					if info.3 == CandidateStatus::Banned {
						T::Currency::reserve(&who, T::MinimumGuardianDeposit::get())?;
					}

					// Update hash
					info.0 = info_hash;
					// Update block number
					info.1 = frame_system::Pallet::<T>::block_number();
					Self::deposit_event(Event::GuardianUpdated {
						guardian: who,
						guardian_index,
						info_hash,
					});
					Ok(())
				},
			)?;
			Ok(())
		}

		/// Clean a guardian legal info
		#[pallet::call_index(2)]
		#[pallet::weight({195_000_000})]
		#[transactional]
		pub fn clean_guardian(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure existing
			let guardian_index =
				PublicGuardianToIndex::<T>::take(&who).ok_or(Error::<T>::GuardianNotRegistered)?;

			// Update guardian
			// But if banned, then require extra reserve
			GuardianIndexToInfo::<T>::try_mutate_exists(
				guardian_index,
				|maybe_info| -> Result<(), DispatchError> {
					let info = maybe_info.as_ref().ok_or(Error::<T>::GuardianIndexNotExist)?;

					if info.3 != CandidateStatus::Banned {
						let _ = T::Currency::unreserve(&who, T::MinimumGuardianDeposit::get());
					}

					// Delete item
					*maybe_info = None;
					Self::deposit_event(Event::GuardianCleaned { guardian: who, guardian_index });
					Ok(())
				},
			)?;
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight({195_000_000})]
		pub fn judge_guardian_status(
			origin: OriginFor<T>,
			guardian: T::AccountId,
			status: CandidateStatus,
		) -> DispatchResult {
			T::GuardianJudgeOrigin::ensure_origin(origin)?;
			let guardian_index = PublicGuardianToIndex::<T>::get(&guardian)
				.ok_or(Error::<T>::GuardianNotRegistered)?;
			GuardianIndexToInfo::<T>::try_mutate_exists(
				guardian_index,
				|maybe_info| -> Result<(), DispatchError> {
					let info = maybe_info.as_mut().ok_or(Error::<T>::GuardianIndexNotExist)?;
					// Update block number
					info.1 = frame_system::Pallet::<T>::block_number();
					// Update status
					info.3 = status;

					Self::deposit_event(Event::GuardianStatusUpdated {
						guardian,
						guardian_index,
						status,
					});
					Ok(())
				},
			)?;
			Ok(())
		}

		/// Anyone can vote for guardian
		/// However if voter is not participating the staking pool
		/// then its vote will never effecting guardian selection procedure
		#[pallet::call_index(4)]
		#[pallet::weight({195_000_000})]
		pub fn vote(
			origin: OriginFor<T>,
			guardian: T::AccountId,
			status: Option<GuardianVote>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// Ensure existing
			let guardian_index = PublicGuardianToIndex::<T>::get(&guardian)
				.ok_or(Error::<T>::GuardianNotRegistered)?;
			if let Some(i) = status {
				GuardianVotes::<T>::insert(&who, guardian_index, i);
			} else {
				GuardianVotes::<T>::remove(&who, guardian_index);
			}

			Self::deposit_event(Event::VoteGuardian {
				voter: who,
				guardian_index,
				guardian,
				status,
			});
			Ok(())
		}

		/// Remove vote to None
		#[pallet::call_index(5)]
		#[pallet::weight({195_000_000})]
		pub fn remove_all_votes(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let _ = GuardianVotes::<T>::clear_prefix(&who, u32::MAX, None);
			Self::deposit_event(Event::RemoveAllVote { voter: who });
			Ok(())
		}
	}
}

/// Some sort of check on the origin is from guardian.
impl<T: Config> GuardianQuery<T::AccountId> for Pallet<T> {
	fn is_guardian(account: T::AccountId) -> bool {
		if let Some(guardian_index) = PublicGuardianToIndex::<T>::get(&account) {
			if let Some(info) = GuardianIndexToInfo::<T>::get(guardian_index) {
				if info.3 != CandidateStatus::Banned {
					return true;
				}
			}
		}

		false
	}

	fn is_verified_guardian(account: T::AccountId) -> bool {
		if let Some(guardian_index) = PublicGuardianToIndex::<T>::get(&account) {
			if let Some(info) = GuardianIndexToInfo::<T>::get(guardian_index) {
				if info.3 == CandidateStatus::Verified {
					return true;
				}
			}
		}

		false
	}

	fn get_vote(voter: T::AccountId, guardian: T::AccountId) -> Option<GuardianVote> {
		// Ensure existing
		if let Some(guardian_index) = PublicGuardianToIndex::<T>::get(&guardian) {
			return GuardianVotes::<T>::get(&voter, guardian_index);
		}
		None
	}
}
