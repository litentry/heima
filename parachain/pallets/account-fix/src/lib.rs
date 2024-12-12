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

//! A pallet for temporary fix of onchain accountInfo.
//! No storage for this pallet and it should be removed right after fixing.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	pallet_prelude::*,
	traits::{
		fungible::Mutate,
		tokens::{Fortitude, Precision, Preservation},
		Currency, ReservableCurrency, StorageVersion,
	},
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use sp_runtime::traits::StaticLookup;
use sp_std::vec::Vec;

type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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
	pub trait Config: frame_system::Config + pallet_balances::Config {
		type Currency: ReservableCurrency<Self::AccountId>;
		type IncConsumerOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type AddBalanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type BurnOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(195_000_000 * (who.len() as u64), 0))]
		pub fn inc_consumers(
			origin: OriginFor<T>,
			who: Vec<T::AccountId>,
		) -> DispatchResultWithPostInfo {
			let _ = T::IncConsumerOrigin::ensure_origin(origin)?;
			for i in &who {
				frame_system::Pallet::<T>::inc_consumers(i)?;
			}
			Ok(Pays::No.into())
		}

		/// add some balance of an existing account
		#[pallet::call_index(1)]
		#[pallet::weight({195_000_000})]
		pub fn add_balance(
			origin: OriginFor<T>,
			who: AccountIdLookupOf<T>,
			#[pallet::compact] add_free: BalanceOf<T>,
			#[pallet::compact] add_reserved: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let _ = T::AddBalanceOrigin::ensure_origin(origin)?;
			let who = T::Lookup::lookup(who)?;
			let add_total = add_free + add_reserved;

			// First we try to modify the account's balance to the forced balance.
			let _ = T::Currency::deposit_into_existing(&who, add_total)?;
			// Then do the reservation
			T::Currency::reserve(&who, add_reserved)?;

			Ok(Pays::No.into())
		}

		// burn balance from a given account
		#[pallet::call_index(2)]
		#[pallet::weight({195_000_000})]
		pub fn burn(
			origin: OriginFor<T>,
			who: T::AccountId,
			amount: <T as pallet_balances::Config>::Balance,
		) -> DispatchResultWithPostInfo {
			let _ = T::BurnOrigin::ensure_origin(origin)?;
			let _ = pallet_balances::Pallet::<T>::burn_from(
				&who,
				amount,
				Preservation::Expendable,
				Precision::Exact,
				Fortitude::Polite,
			)?;
			Ok(Pays::No.into())
		}
	}
}
