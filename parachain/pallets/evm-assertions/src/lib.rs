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
#![allow(clippy::useless_conversion)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;
use sp_std::vec::Vec;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::OriginFor;
	use scale_info::TypeInfo;
	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[derive(Encode, Decode, Clone, Default, Debug, PartialEq, Eq, TypeInfo)]
	pub struct Assertion {
		pub byte_code: Vec<u8>,
		pub secrets: Vec<Vec<u8>>,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Assertion identifier, usually H160
		type AssertionId: Default
			+ Copy
			+ PartialEq
			+ core::fmt::Debug
			+ parity_scale_codec::FullCodec
			+ TypeInfo;

		/// Only a member of the Developers Collective can deploy the contract
		type ContractDevOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Only TEE-Workers can call some extrinsics
		type TEECallOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}

	/// Map for storing assertion smart contract bytecode alongside with additional secrets
	/// Secrets are encrypted with worker's shielding key
	#[pallet::storage]
	#[pallet::getter(fn assertions)]
	pub type Assertions<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssertionId, Assertion, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AssertionCreated { id: T::AssertionId, byte_code: Vec<u8>, secrets: Vec<Vec<u8>> },
		AssertionStored { id: T::AssertionId },
		AssertionVoided { id: T::AssertionId },
	}

	#[pallet::error]
	pub enum Error<T> {
		AssertionExists,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight((T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
		pub fn create_assertion(
			origin: OriginFor<T>,
			id: T::AssertionId,
			byte_code: Vec<u8>,
			secrets: Vec<Vec<u8>>,
		) -> DispatchResultWithPostInfo {
			let _ = T::ContractDevOrigin::ensure_origin(origin)?;
			ensure!(!Assertions::<T>::contains_key(id), Error::<T>::AssertionExists);
			Assertions::<T>::insert(
				id,
				Assertion { byte_code: byte_code.clone(), secrets: secrets.clone() },
			);
			Self::deposit_event(Event::AssertionCreated { id, byte_code, secrets });
			Ok(Pays::No.into())
		}

		/// Only called by the Identity-Worker
		#[pallet::call_index(1)]
		#[pallet::weight((T::DbWeight::get().read, DispatchClass::Normal, Pays::No))]
		pub fn store_assertion(
			origin: OriginFor<T>,
			id: T::AssertionId,
		) -> DispatchResultWithPostInfo {
			let _ = T::TEECallOrigin::ensure_origin(origin)?;
			Self::deposit_event(Event::AssertionStored { id });
			Ok(Pays::No.into())
		}

		/// Only called by the Identity-Worker
		#[pallet::call_index(2)]
		#[pallet::weight((T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
		pub fn void_assertion(
			origin: OriginFor<T>,
			id: T::AssertionId,
		) -> DispatchResultWithPostInfo {
			let _ = T::TEECallOrigin::ensure_origin(origin)?;
			Assertions::<T>::remove(id);
			Self::deposit_event(Event::AssertionVoided { id });
			Ok(Pays::No.into())
		}
	}
}
