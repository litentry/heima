// Copyright 2020-2021 Litentry Technologies GmbH.
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
//! Inspired by Acala and Moonbeam implementations.
//!
//! This pallet is used to bind to frame_system::BaseCallFilter in runtime,
//! aiming to provide 3 modes:
//! - `safe mode`:   only core systems are running
//! - `normal mode`: the normal status when parachain is running
//! - `test mode`:   all extrinsics are allowed (useful for testing)
//!
//! On top of it, it should be possible to selectively block certain extrinsic
//! or all extrinsics in certain pallet (blacklist feature).
//!
//! Currenly, no “whitelisting” is implemented, as the only usecase for whitelisting
//! appears to be testing, which could be covered by `test mode` + optional blacklisting.
//! Moreover, whitelisting would bring about more state transitions and
//! increase the complexity of this pallet.
//!
//! Setting the mode and blocking extrinsics should only come from a priviledged origin.

#![cfg_attr(not(feature = "std"), no_std)]

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;
// pub mod weights;

use frame_support::{
	dispatch::{CallMetadata, GetCallMetadata},
	pallet_prelude::*,
	traits::{Contains, StorageVersion},
	transactional,
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use sp_std::{prelude::*, vec::Vec};

use scale_info::TypeInfo;

#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, Debug, TypeInfo)]
pub enum OperationalMode {
	/// when parachain runs normally
	Normal,
	/// when parachain traps in contigency situation
	Safe,
	/// when parachain is used for testing purpose
	Test,
}

impl Default for OperationalMode {
	fn default() -> Self {
		OperationalMode::Normal
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	/// A wildcard to represents all extrinsics in one pallet
	/// const WILDCARD_FUNCTION_NAME: &str = "*";

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The priviledged origin to perform all operations
		type UpdateOrigin: EnsureOrigin<Self::Origin>;

		/// Filters that will be specified in runtime binding
		type NormalModeFilter: Contains<Self::Call>;
		type SafeModeFilter: Contains<Self::Call>;
		type TestModeFilter: Contains<Self::Call>;
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
		/// a mode was just sent
		ModeSet { mode: OperationalMode },
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
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set the mode
		/// Weights should be 2 DB writes: 1 for mode and 1 for event
		#[pallet::weight(2 * T::DbWeight::get().write)]
		pub fn set_mode(origin: OriginFor<T>, mode: OperationalMode) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;

			// we don't care which mode we are currently in,
			// just set to the desired mode
			Mode::<T>::put(mode);

			Self::deposit_event(Event::ModeSet { mode });

			Ok(().into())
		}

		/// block the given extrinsics
		/// (pallet_name_bytes, function_name_bytes) can uniquely identify an extrinsic
		/// if function_name_bytes is None, all extrinsics in `pallet_name_bytes` will be blocked
		#[pallet::weight(10_000)]
		#[transactional]
		pub fn block_extrinsics(
			origin: OriginFor<T>,
			pallet_name_bytes: Vec<u8>,
			function_name_bytes: Option<Vec<u8>>,
		) -> DispatchResultWithPostInfo {
			T::UpdateOrigin::ensure_origin(origin)?;
			let pallet_name_string = sp_std::str::from_utf8(&pallet_name_bytes)
				.map_err(|_| Error::<T>::CannotConvertToString)?;
			// we disallow blocking this pallet
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
					}
				},
			);
			Ok(().into())
		}

		/// unblock the given extrinsics
		/// (pallet_name_bytes, function_name_bytes) can uniquely identify an extrinsic
		/// if function_name_bytes is None, all extrinsics in `pallet_name_bytes` will be unblocked
		#[pallet::weight(10_000)]
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
			}

			Ok(().into())
		}
	}

	impl<T: Config> Contains<T::Call> for Pallet<T>
	where
		<T as frame_system::Config>::Call: GetCallMetadata,
	{
		fn contains(call: &T::Call) -> bool {
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
				))
			}

			false
		}
	}
}
