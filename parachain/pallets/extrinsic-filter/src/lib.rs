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

//! A pallet for dynamically filtering the extrinsics
//!
//! Inspired by:
//! - Acala `transaction-pause`
//! - Moonbeam `maintenance-mode`
//!
//! This pallet is used to bind to frame_system::BaseCallFilter in runtime,
//! aiming to provide 3 modes:
//! - `safe mode`:   only core extrinsics are allowed
//! - `normal mode`: the normal status when parachain is running
//! - `test mode`:   all extrinsics are allowed
//!
//! On top of it, it should be possible to selectively block certain extrinsic
//! or all extrinsics in certain pallet (blacklisting).
//!
//! Currenly, this pallet is mainly served as security guard.
//! Therefore no “whitelisting” is supported, as the only usecase for whitelisting
//! appears to be testing, which could be covered by `test mode` + optional blacklisting.
//! Moreover, whitelisting would bring about more state transitions and
//! increase the complexity of this pallet.
//!
//! The dispatchables `block_extrinsics` and `unblock_extrinsics` MUST be called on pair.
//! For exmaple:
//!   1. block_extrinsics(pallet_A, fn_A)
//!   2. block_extrinsics(pallet_A, None)
//! to completely unblock fn_A, you need to call:
//!   3. unblock_extrinsics(pallet_A, fn_A)
//!   4. unblock_extrinsics(pallet_A, None)
//! the order of 3 and 4 doesn't matter though.
//!
//! We disallow(nullify) blocking a single extrinsic and unblocking it by unblocking its
//! belonging pallet, it means:
//!   1. block_extrinsics(pallet_A, fn_A)
//! and then
//!   2. unlock_extrinsics(pallet_A, None)
//!   will not work, and vice versa.
//!
//! The reasons:
//! - simplicity
//! - whitelisting is not supported
//!
//! All dispatchables in this pallet must come from a priviledged origin.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

use frame_support::{
	pallet_prelude::*,
	traits::{CallMetadata, Contains, GetCallMetadata, StorageVersion},
	transactional,
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use sp_std::prelude::*;

use scale_info::TypeInfo;
pub use weights::WeightInfo;

#[derive(PartialEq, Eq, Clone, Copy, Default, Encode, Decode, Debug, TypeInfo)]
pub enum OperationalMode {
	/// when parachain runs normally
	#[default]
	#[codec(index = 0)]
	Normal,
	/// when parachain traps in contigency situation
	#[codec(index = 1)]
	Safe,
	/// when parachain is used for testing purpose
	#[codec(index = 2)]
	Test,
}

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
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The priviledged origin to perform all operations
		type UpdateOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Filters that will be specified in runtime binding
		type NormalModeFilter: Contains<Self::RuntimeCall>;
		type SafeModeFilter: Contains<Self::RuntimeCall>;
		type TestModeFilter: Contains<Self::RuntimeCall>;

		/// Weights
		type WeightInfo: WeightInfo;
	}

	/// current mode, ValueQuery as it can't be None
	#[pallet::storage]
	#[pallet::getter(fn mode)]
	pub type Mode<T: Config> = StorageValue<_, OperationalMode, ValueQuery>;

	/// a tuple (pallet_name_bytes, Option<function_name_bytes>) to represent blocked extrinsics
	/// if `Option<function_name_bytes>` is None, then all extrinsics in `pallet_name_bytes` are
	/// blocked
	#[pallet::storage]
	#[pallet::getter(fn blocked_extrinsics)]
	pub type BlockedExtrinsics<T: Config> =
		StorageMap<_, Twox64Concat, (Vec<u8>, Vec<u8>), (), OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// a new mode was just sent
		ModeSet { new_mode: OperationalMode },
		/// some extrinsics are blocked
		ExtrinsicsBlocked { pallet_name_bytes: Vec<u8>, function_name_bytes: Option<Vec<u8>> },
		/// some extrinsics are unblocked
		ExtrinsicsUnblocked { pallet_name_bytes: Vec<u8>, function_name_bytes: Option<Vec<u8>> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error when a given extrinsic cannot be blocked (e.g. this pallet)
		CannotBlock,
		/// Error during conversion bytes to utf8 string
		CannotConvertToString,
		/// Error when trying to block extrinsic more than once
		ExtrinsicAlreadyBlocked,
		/// Error when trying to unblock a non-existent extrinsic
		ExtrinsicNotBlocked,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set the mode
		///
		/// The storage of `BlockedExtrinsics` is unaffected.
		/// The reason is we'd rather have this pallet behave conservatively:
		/// having extra blocked extrinsics is better than having unexpected whitelisted extrinsics.
		/// See the test `set_mode_should_not_clear_blocked_extrinsics()`
		///
		/// Weights should be 2 DB writes: 1 for mode and 1 for event
		#[pallet::call_index(0)]
		#[pallet::weight(2 * T::DbWeight::get().write)]
		pub fn set_mode(origin: OriginFor<T>, mode: OperationalMode) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;

			// we don't care which mode we are currently in,
			// just set to the desired mode
			Mode::<T>::put(mode);

			Self::deposit_event(Event::ModeSet { new_mode: mode });

			Ok(().into())
		}

		/// block the given extrinsics
		/// (pallet_name_bytes, function_name_bytes) can uniquely identify an extrinsic
		/// if function_name_bytes is None, all extrinsics in `pallet_name_bytes` will be blocked
		#[pallet::call_index(1)]
		#[pallet::weight({10_000})]
		#[transactional]
		pub fn block_extrinsics(
			origin: OriginFor<T>,
			pallet_name_bytes: Vec<u8>,
			function_name_bytes: Option<Vec<u8>>,
		) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			let pallet_name_string = sp_std::str::from_utf8(&pallet_name_bytes)
				.map_err(|_| Error::<T>::CannotConvertToString)?;
			// we disallow blocking this pallet itself
			ensure!(
				pallet_name_string != <Self as PalletInfoAccess>::name(),
				Error::<T>::CannotBlock
			);

			BlockedExtrinsics::<T>::mutate(
				(pallet_name_bytes.clone(), function_name_bytes.clone().unwrap_or_default()),
				|v| {
					if v.is_none() {
						*v = Some(());
						Self::deposit_event(Event::ExtrinsicsBlocked {
							pallet_name_bytes,
							function_name_bytes,
						});
						// do not pay the fee upon successful block
						Ok(Pays::No.into())
					} else {
						Err(Error::<T>::ExtrinsicAlreadyBlocked.into())
					}
				},
			)
		}

		/// unblock the given extrinsics
		/// (pallet_name_bytes, function_name_bytes) can uniquely identify an extrinsic
		/// if function_name_bytes is None, all extrinsics in `pallet_name_bytes` will be unblocked
		#[pallet::call_index(2)]
		#[pallet::weight({10_000})]
		#[transactional]
		pub fn unblock_extrinsics(
			origin: OriginFor<T>,
			pallet_name_bytes: Vec<u8>,
			function_name_bytes: Option<Vec<u8>>,
		) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			if BlockedExtrinsics::<T>::take((
				pallet_name_bytes.clone(),
				function_name_bytes.clone().unwrap_or_default(),
			))
			.is_some()
			{
				Self::deposit_event(Event::ExtrinsicsUnblocked {
					pallet_name_bytes,
					function_name_bytes,
				});
			} else {
				return Err(Error::<T>::ExtrinsicNotBlocked.into());
			}
			// do not pay the fee upon successful unblock
			Ok(Pays::No.into())
		}
	}

	impl<T: Config> Contains<T::RuntimeCall> for Pallet<T>
	where
		<T as frame_system::Config>::RuntimeCall: GetCallMetadata,
	{
		fn contains(call: &T::RuntimeCall) -> bool {
			let allowed_by_mode = match Self::mode() {
				OperationalMode::Normal => T::NormalModeFilter::contains(call),
				OperationalMode::Safe => T::SafeModeFilter::contains(call),
				OperationalMode::Test => T::TestModeFilter::contains(call),
			};
			// if allowed_by_mode is true, continue to check BlockedExtrinsics
			// remember we only have blacklisting, no whitelisting
			if allowed_by_mode {
				let CallMetadata { function_name, pallet_name } = call.get_call_metadata();
				return !BlockedExtrinsics::<T>::contains_key((
					pallet_name.as_bytes(),
					function_name.as_bytes(),
				)) && !BlockedExtrinsics::<T>::contains_key((
					pallet_name.as_bytes(),
					Vec::<u8>::default(),
				));
			}

			false
		}
	}
}
