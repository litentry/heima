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

use frame_support::{
	dispatch::{DispatchResult, DispatchResultWithPostInfo},
	ensure,
	pallet_prelude::*,
	traits::Get,
};
use frame_system::pallet_prelude::*;
use heima_primitives::Identity;

pub use pallet::*;

mod custodial_wallet;
pub use custodial_wallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		// some extrinsics should only be called by origins from TEE
		type TEECallOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
		// origin to manage Relayer Admin
		type SetAdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}

	#[pallet::storage]
	#[pallet::getter(fn admin)]
	pub type Admin<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	// use `Identity` as key to keep the flexibility, however we do further check its type when
	// adding them
	#[pallet::storage]
	#[pallet::getter(fn relayer)]
	pub type Relayer<T: Config> = StorageMap<_, Blake2_128Concat, Identity, (), OptionQuery>;

	// `ValueQuery` is used as each field in CustodialWallet is optional already
	// not using Option<CustodialWallet> either as each field is set separately
	#[pallet::storage]
	#[pallet::getter(fn vault)]
	pub type Vault<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, CustodialWallet, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AdminSet { new_admin: Option<T::AccountId> },
		RelayerAdded { who: Identity },
		RelayerRemoved { who: Identity },
		BtcWalletGenerated { pub_key: PubKey33, account_id: T::AccountId },
		EthWalletGenerated { pub_key: PubKey33 },
		TonWalletGenerated { pub_key: PubKey32 },
		VaultRemoved { who: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		RequireAdminOrRoot,
		RelayerNotExist,
		UnsupportedRelayerType,
		BtcWalletAlreadyExist,
		EthWalletAlreadyExist,
		TonWalletAlreadyExist,
		VaultNotExist,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub admin: Option<T::AccountId>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { admin: None }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			if let Some(ref admin) = self.admin {
				Admin::<T>::put(admin);
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set the admin account
		///
		/// Weights should be 2 DB writes: 1 for mode and 1 for event
		#[pallet::call_index(0)]
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

		#[pallet::call_index(1)]
		#[pallet::weight({195_000_000})]
		pub fn add_relayer(origin: OriginFor<T>, account: Identity) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			ensure!(account.is_substrate() || account.is_evm(), Error::<T>::UnsupportedRelayerType);
			// we don't care if `account` already exists
			Relayer::<T>::insert(account.clone(), ());
			Self::deposit_event(Event::RelayerAdded { who: account });
			Ok(Pays::No.into())
		}

		#[pallet::call_index(2)]
		#[pallet::weight({195_000_000})]
		pub fn remove_relayer(
			origin: OriginFor<T>,
			account: Identity,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			ensure!(Relayer::<T>::contains_key(&account), Error::<T>::RelayerNotExist);
			Relayer::<T>::remove(account.clone());
			Self::deposit_event(Event::RelayerRemoved { who: account });
			Ok(Pays::No.into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight({195_000_000})]
		pub fn remove_vault(
			origin: OriginFor<T>,
			account: T::AccountId,
		) -> DispatchResultWithPostInfo {
			Self::ensure_admin_or_root(origin)?;
			ensure!(Vault::<T>::contains_key(&account), Error::<T>::VaultNotExist);
			Vault::<T>::remove(account.clone());
			Self::deposit_event(Event::VaultRemoved { who: account });
			Ok(Pays::No.into())
		}

		/// ---------------------------------------------------
		/// The following extrinsics are supposed to be called by TEE only
		/// ---------------------------------------------------
		#[pallet::call_index(30)]
		#[pallet::weight(({195_000_000}, DispatchClass::Normal, Pays::No))]
		pub fn btc_wallet_generated(
			origin: OriginFor<T>,
			pub_key: PubKey33,
		) -> DispatchResultWithPostInfo {
			let tee_account = T::TEECallOrigin::ensure_origin(origin)?;
			Vault::<T>::try_mutate(tee_account.clone(), |v| {
				ensure!(!v.has_btc(), Error::<T>::BtcWalletAlreadyExist);
				v.btc = Some(pub_key);
				Self::deposit_event(Event::BtcWalletGenerated { pub_key, account_id: tee_account });
				Ok(Pays::No.into())
			})
		}

		#[pallet::call_index(31)]
		#[pallet::weight(({195_000_000}, DispatchClass::Normal, Pays::No))]
		pub fn eth_wallet_generated(
			origin: OriginFor<T>,
			pub_key: PubKey33,
		) -> DispatchResultWithPostInfo {
			let tee_account = T::TEECallOrigin::ensure_origin(origin)?;
			Vault::<T>::try_mutate(tee_account, |v| {
				ensure!(!v.has_eth(), Error::<T>::EthWalletAlreadyExist);
				v.eth = Some(pub_key);
				Self::deposit_event(Event::EthWalletGenerated { pub_key });
				Ok(Pays::No.into())
			})
		}

		#[pallet::call_index(32)]
		#[pallet::weight(({195_000_000}, DispatchClass::Normal, Pays::No))]
		pub fn ton_wallet_generated(
			origin: OriginFor<T>,
			pub_key: PubKey32,
		) -> DispatchResultWithPostInfo {
			let tee_account = T::TEECallOrigin::ensure_origin(origin)?;
			Vault::<T>::try_mutate(tee_account, |v| {
				ensure!(!v.has_ton(), Error::<T>::TonWalletAlreadyExist);
				v.ton = Some(pub_key);
				Self::deposit_event(Event::TonWalletGenerated { pub_key });
				Ok(Pays::No.into())
			})
		}

		// TODO: placeholder
		#[pallet::call_index(33)]
		#[pallet::weight(({195_000_000}, DispatchClass::Normal, Pays::No))]
		pub fn task_complete(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let _ = T::TEECallOrigin::ensure_origin(origin)?;
			Ok(Pays::No.into())
		}
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
