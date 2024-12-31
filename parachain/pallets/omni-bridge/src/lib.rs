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

use parity_scale_codec::Codec;
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_runtime::FixedPointOperand;
use frame_support::ensure;
use frame_system::ensure_root;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::ensure_signed;
    use frame_system::pallet_prelude::OriginFor;
    use scale_info::TypeInfo;
    use scale_info::prelude::vec::Vec;
    use sp_std::{fmt::Debug, prelude::*, vec};

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

        /// The units in which we record balances.
        type Balance: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Codec
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + MaxEncodedLen
            + TypeInfo
            + FixedPointOperand;

        // origin to manage Relayer Admin
        type SetAdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }

    #[pallet::storage]
    #[pallet::getter(fn admin)]
    pub type Admin<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn relayer)]
    pub type Relayer<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, (), OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Relayer added
        RelayerAdded(T::AccountId),
        /// Relayer removed
        RelayerRemoved(T::AccountId),

        /// Account paid in tokens, they will be paid out on the other side of the bridge.
        PaidIn(T::Balance, Vec<u8>),
        /// Tokens were paid out to the account after being paid in on the other side of the bridge.
        PaidOut(T::Balance, T::AccountId),

        /// Admins was set
        AdminSet { new_admin: Option<T::AccountId> },
    }

    #[pallet::error]
    pub enum Error<T> {
        RequireAdminOrRoot,
        UnknownRelayer,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight((T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
        pub fn pay_in(
            _origin: OriginFor<T>,
            balance: T::Balance,
            // contains recipient address
            call_data: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            // check user has amount
            // take tokens from user's account
            Self::deposit_event(Event::PaidIn(balance, call_data));
            // todo: should pay
            Ok(Pays::No.into())
        }

        #[pallet::call_index(1)]
        #[pallet::weight((T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
        pub fn pay_out(origin: OriginFor<T>, balance: T::Balance, _recipient: T::AccountId) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // check user is an relayer
            // add tokens to user account
            Self::deposit_event(Event::PaidOut(balance, who));
            Ok(Pays::No.into())
        }

        #[pallet::call_index(2)]
        #[pallet::weight((T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
        pub fn add_relayer(origin: OriginFor<T>, relayer: T::AccountId) -> DispatchResultWithPostInfo {
            Self::ensure_admin_or_root(origin)?;
            Relayer::<T>::insert(relayer.clone(), ());
            Self::deposit_event(Event::RelayerAdded(relayer));
            Ok(Pays::No.into())
        }

        #[pallet::call_index(3)]
        #[pallet::weight((T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
        pub fn remove_relayer(origin: OriginFor<T>, relayer: T::AccountId) -> DispatchResultWithPostInfo {
            Self::ensure_admin_or_root(origin)?;
            ensure!(Relayer::<T>::contains_key(&relayer), Error::<T>::UnknownRelayer);
            Relayer::<T>::remove(relayer.clone());
            Self::deposit_event(Event::RelayerRemoved(relayer));
            Ok(Pays::No.into())
        }

        #[pallet::call_index(4)]
        #[pallet::weight((2 * T::DbWeight::get().write, DispatchClass::Normal, Pays::No))]
        pub fn set_admin(
            origin: OriginFor<T>,
            new_admin: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            T::SetAdminOrigin::ensure_origin(origin)?;
            Admin::<T>::put(new_admin.clone());
            Self::deposit_event(Event::AdminSet { new_admin: Some(new_admin) });
            Ok(Pays::No.into())
        }
    }

    impl<T: Config> Pallet<T> {
        fn ensure_admin_or_root(origin: OriginFor<T>) -> DispatchResult {
            ensure!(
			ensure_root(origin.clone()).is_ok() || Some(ensure_signed(origin)?) == Self::admin(),
			Error::<T>::RequireAdminOrRoot
		);
            Ok(())
        }
    }
}