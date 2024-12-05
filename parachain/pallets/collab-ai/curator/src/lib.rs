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
//! # Curator Pallet
//!
//! - [`Config`]
//! - [`Call`]
//!
//! ## Overview
//!
//! The Curator pallet handles the administration of general curator and proposed staking pool.
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

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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

		/// The minimum amount to be used as a deposit for a curator
		#[pallet::constant]
		type MinimumCuratorDeposit: Get<BalanceOf<Self>>;

		/// Origin from curator legal file verified by
		type CuratorJudgeOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}

	#[pallet::type_value]
	pub fn DefaultForCuratorIndex() -> CuratorIndex {
		1u128
	}

	/// The next curator index, the number of (public) curator that have been made so far + 1.
	#[pallet::storage]
	#[pallet::getter(fn public_curator_count)]
	pub type PublicCuratorCount<T> =
		StorageValue<_, CuratorIndex, ValueQuery, DefaultForCuratorIndex>;

	/// The public curator to index
	#[pallet::storage]
	#[pallet::getter(fn public_curator_to_index)]
	pub type PublicCuratorToIndex<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, CuratorIndex, OptionQuery>;

	/// Curator index to hash and update time. Info Hash is current used curator legal file hash.
	#[pallet::storage]
	#[pallet::getter(fn curator_index_to_info)]
	pub type CuratorIndexToInfo<T: Config> = StorageMap<
		_,
		Twox64Concat,
		CuratorIndex,
		(InfoHash, BlockNumberFor<T>, T::AccountId, CandidateStatus),
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CuratorRegisted {
			curator: T::AccountId,
			curator_index: CuratorIndex,
			info_hash: InfoHash,
		},
		CuratorUpdated {
			curator: T::AccountId,
			curator_index: CuratorIndex,
			info_hash: InfoHash,
		},
		CuratorCleaned {
			curator: T::AccountId,
			curator_index: CuratorIndex,
		},
		CuratorStatusUpdated {
			curator: T::AccountId,
			curator_index: CuratorIndex,
			status: CandidateStatus,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		CuratorAlreadyRegistered,
		CuratorNotRegistered,
		CuratorIndexNotExist,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Registing a curator legal info
		#[pallet::call_index(0)]
		#[pallet::weight({195_000_000})]
		#[transactional]
		pub fn regist_curator(origin: OriginFor<T>, info_hash: InfoHash) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure curator not existing yet
			ensure!(
				!PublicCuratorToIndex::<T>::contains_key(&who),
				Error::<T>::CuratorAlreadyRegistered
			);
			// New registed curator need to make a balance reserve
			T::Currency::reserve(&who, T::MinimumCuratorDeposit::get())?;

			// Update curator
			let current_block = frame_system::Pallet::<T>::block_number();
			let next_curator_index = PublicCuratorCount::<T>::get();

			PublicCuratorToIndex::<T>::insert(&who, next_curator_index);
			CuratorIndexToInfo::<T>::insert(
				next_curator_index,
				(info_hash, current_block, who.clone(), CandidateStatus::Unverified),
			);
			PublicCuratorCount::<T>::put(
				next_curator_index.checked_add(1u32.into()).ok_or(ArithmeticError::Overflow)?,
			);

			Self::deposit_event(Event::CuratorRegisted {
				curator: who,
				curator_index: next_curator_index,
				info_hash,
			});
			Ok(())
		}

		/// Updating a curator legal info
		#[pallet::call_index(1)]
		#[pallet::weight({195_000_000})]
		pub fn update_curator(origin: OriginFor<T>, info_hash: InfoHash) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure existing
			let curator_index =
				PublicCuratorToIndex::<T>::get(&who).ok_or(Error::<T>::CuratorNotRegistered)?;

			// Update curator
			// But if banned, then require extra reserve
			CuratorIndexToInfo::<T>::try_mutate_exists(
				curator_index,
				|maybe_info| -> Result<(), DispatchError> {
					let info = maybe_info.as_mut().ok_or(Error::<T>::CuratorIndexNotExist)?;

					if info.3 == CandidateStatus::Banned {
						T::Currency::reserve(&who, T::MinimumCuratorDeposit::get())?;
					}

					// Update hash
					info.0 = info_hash;
					// Update block number
					info.1 = frame_system::Pallet::<T>::block_number();
					Self::deposit_event(Event::CuratorUpdated {
						curator: who,
						curator_index,
						info_hash,
					});
					Ok(())
				},
			)?;
			Ok(())
		}

		/// Clean a curator legal info
		#[pallet::call_index(2)]
		#[pallet::weight({195_000_000})]
		#[transactional]
		pub fn clean_curator(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Ensure existing
			let curator_index =
				PublicCuratorToIndex::<T>::take(&who).ok_or(Error::<T>::CuratorNotRegistered)?;

			// Update curator
			// But if banned, then require extra reserve
			CuratorIndexToInfo::<T>::try_mutate_exists(
				curator_index,
				|maybe_info| -> Result<(), DispatchError> {
					let info = maybe_info.as_ref().ok_or(Error::<T>::CuratorIndexNotExist)?;

					if info.3 != CandidateStatus::Banned {
						let _ = T::Currency::unreserve(&who, T::MinimumCuratorDeposit::get());
					}

					// Delete item
					*maybe_info = None;
					Self::deposit_event(Event::CuratorCleaned { curator: who, curator_index });
					Ok(())
				},
			)?;
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight({195_000_000})]
		pub fn judge_curator_status(
			origin: OriginFor<T>,
			curator: T::AccountId,
			status: CandidateStatus,
		) -> DispatchResult {
			T::CuratorJudgeOrigin::ensure_origin(origin)?;
			let curator_index = PublicCuratorToIndex::<T>::get(curator.clone())
				.ok_or(Error::<T>::CuratorNotRegistered)?;
			CuratorIndexToInfo::<T>::try_mutate_exists(
				curator_index,
				|maybe_info| -> Result<(), DispatchError> {
					let info = maybe_info.as_mut().ok_or(Error::<T>::CuratorIndexNotExist)?;
					// Update block number
					info.1 = frame_system::Pallet::<T>::block_number();
					// Update status
					info.3 = status;

					Self::deposit_event(Event::CuratorStatusUpdated {
						curator,
						curator_index,
						status,
					});
					Ok(())
				},
			)?;
			Ok(())
		}
	}
}

/// Some sort of check on the origin is from curator.
impl<T: Config> CuratorQuery<T::AccountId> for Pallet<T> {
	fn is_curator(account: T::AccountId) -> bool {
		if let Some(curator_index) = PublicCuratorToIndex::<T>::get(&account) {
			if let Some(info) = CuratorIndexToInfo::<T>::get(curator_index) {
				if info.3 != CandidateStatus::Banned {
					return true;
				}
			}
		}

		false
	}

	fn is_verified_curator(account: T::AccountId) -> bool {
		if let Some(curator_index) = PublicCuratorToIndex::<T>::get(&account) {
			if let Some(info) = CuratorIndexToInfo::<T>::get(curator_index) {
				if info.3 == CandidateStatus::Verified {
					return true;
				}
			}
		}

		false
	}
}
