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

//! Pallet for converting among AIUSD and other stable token.
//!
#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{
	pallet_prelude::*,
	traits::{
		tokens::{
			fungibles::{Inspect as FsInspect, Mutate as FsMutate},
			Fortitude, Precision, Preservation,
		},
		StorageVersion,
	},
	transactional,
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use sp_runtime::traits::{CheckedDiv, CheckedMul};
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	pub type InspectFungibles<T> = pallet_assets::Pallet<T>;
	/// Balance type alias for balances of assets that implement the `fungibles` trait.
	pub type AssetBalanceOf<T> =
		<InspectFungibles<T> as FsInspect<<T as frame_system::Config>::AccountId>>::Balance;
	/// Type alias for Asset IDs.
	pub type AssetIdOf<T> =
		<InspectFungibles<T> as FsInspect<<T as frame_system::Config>::AccountId>>::AssetId;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_assets::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		// This is not a treasury account
		// Balance of all types in this account record the
		// available stable coin for AIUSD to switch back
		type ConvertingPool: Get<Self::AccountId>;

		// Declare the asset id of AIUSD
		type AIUSDAssetId: Get<AssetIdOf<Self>>;

		/// The privileged origin to enable/disable AIUSD switch
		type ManagerOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}

	// Asset Id => ratio of system exchange rate for AIUSD to other token in 10^18
	// e.g.
	// (1 USDT) = 10^6 in system
	// (1 AIUSD) = 10^18 in system
	// Value of StorageMap n = 10^(-12) * 10^(18) = 10^(6),
	// which means (1 AIUSD) * n = (10^18) * (1 USDT) in system balance when converting.
	#[pallet::storage]
	#[pallet::getter(fn enabled_tokens)]
	pub type EnabledTokens<T: Config> =
		StorageMap<_, Twox64Concat, AssetIdOf<T>, AssetBalanceOf<T>, OptionQuery>;

	#[pallet::error]
	pub enum Error<T> {
		InvalidAssetId,
		AssetNotEnabled,
		Overflow,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// AIUSD minted with other eligible stable token locked
		AIUSDCreated {
			beneficiary: T::AccountId,
			aiusd_amount: AssetBalanceOf<T>,
			asset_id: AssetIdOf<T>,
			asset_amount: AssetBalanceOf<T>,
		},
		/// AIUSD burned with other eligible stable token released
		AIUSDDestroyed {
			beneficiary: T::AccountId,
			aiusd_amount: AssetBalanceOf<T>,
			asset_id: AssetIdOf<T>,
			asset_amount: AssetBalanceOf<T>,
		},
		AssetEnabled {
			asset_id: AssetIdOf<T>,
			decimal_ratio: AssetBalanceOf<T>,
		},
		AssetDisabled {
			asset_id: AssetIdOf<T>,
		},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Lock source_asset_id token and mint AIUSD
		#[pallet::call_index(0)]
		#[pallet::weight({195_000_000})]
		#[transactional]
		pub fn mint_aiusd(
			origin: OriginFor<T>,
			source_asset_id: AssetIdOf<T>,
			aiusd_amount: AssetBalanceOf<T>,
		) -> DispatchResult {
			let beneficiary = ensure_signed(origin)?;
			if let Some(ratio) = EnabledTokens::<T>::get(&source_asset_id) {
				let aiusd_id = T::AIUSDAssetId::get();
				ensure!(
					InspectFungibles::<T>::asset_exists(aiusd_id.clone())
						&& InspectFungibles::<T>::asset_exists(source_asset_id.clone()),
					Error::<T>::InvalidAssetId
				);

				let aiusd_minted_amount: AssetBalanceOf<T> =
					<InspectFungibles<T>>::mint_into(aiusd_id, &beneficiary, aiusd_amount)?;

				// Maybe it is better to save decimal of AIUSD somewhere
				let aiusd_decimal_unit_expression: T::Balance =
					10u128.pow(18).try_into().or(Err(Error::<T>::Overflow))?;
				let asset_target_transfer_amount = aiusd_minted_amount
					.checked_mul(&ratio)
					.ok_or(Error::<T>::Overflow)?
					.checked_div(&aiusd_decimal_unit_expression)
					.ok_or(Error::<T>::Overflow)?;
				let asset_actual_transfer_amount: AssetBalanceOf<T> =
					<InspectFungibles<T> as FsMutate<<T as frame_system::Config>::AccountId>>::transfer(
						source_asset_id.clone(),
						&beneficiary,
						&T::ConvertingPool::get(),
						asset_target_transfer_amount,
						Preservation::Expendable,
					)?;

				Self::deposit_event(Event::AIUSDCreated {
					beneficiary,
					aiusd_amount: aiusd_minted_amount,
					asset_id: source_asset_id,
					asset_amount: asset_actual_transfer_amount,
				});

				Ok(())
			} else {
				Err(Error::<T>::AssetNotEnabled.into())
			}
		}

		// Burn aiusd and get source_asset_id token released
		// Failed if pool does not have enough token of one type
		#[pallet::call_index(1)]
		#[pallet::weight({195_000_000})]
		#[transactional]
		pub fn burn_aiusd(
			origin: OriginFor<T>,
			source_asset_id: AssetIdOf<T>,
			aiusd_amount: AssetBalanceOf<T>,
		) -> DispatchResult {
			let beneficiary = ensure_signed(origin)?;
			if let Some(ratio) = EnabledTokens::<T>::get(&source_asset_id) {
				let aiusd_id = T::AIUSDAssetId::get();
				ensure!(
					InspectFungibles::<T>::asset_exists(aiusd_id.clone())
						&& InspectFungibles::<T>::asset_exists(source_asset_id.clone()),
					Error::<T>::InvalidAssetId
				);
				// It will fail if insufficient fund
				let aiusd_destroyed_amount: AssetBalanceOf<T> = <InspectFungibles<T>>::burn_from(
					aiusd_id,
					&beneficiary,
					aiusd_amount,
					Preservation::Expendable,
					Precision::Exact,
					Fortitude::Polite,
				)?;

				// Maybe it is better to save decimal of AIUSD somewhere
				let aiusd_decimal_unit_expression: T::Balance =
					10u128.pow(18).try_into().or(Err(Error::<T>::Overflow))?;
				let asset_target_transfer_amount = aiusd_destroyed_amount
					.checked_mul(&ratio)
					.ok_or(Error::<T>::Overflow)?
					.checked_div(&aiusd_decimal_unit_expression)
					.ok_or(Error::<T>::Overflow)?;
				let asset_actual_transfer_amount: AssetBalanceOf<T> =
					<InspectFungibles<T> as FsMutate<<T as frame_system::Config>::AccountId>>::transfer(
						source_asset_id.clone(),
						&T::ConvertingPool::get(),
						&beneficiary,
						asset_target_transfer_amount,
						Preservation::Expendable,
					)?;

				Self::deposit_event(Event::AIUSDDestroyed {
					beneficiary,
					aiusd_amount: aiusd_destroyed_amount,
					asset_id: source_asset_id,
					asset_amount: asset_actual_transfer_amount,
				});
				Ok(())
			} else {
				Err(Error::<T>::AssetNotEnabled.into())
			}
		}

		/// Enable a specific type of token available for switching
		#[pallet::call_index(2)]
		#[pallet::weight({195_000_000})]
		pub fn enable_token(
			origin: OriginFor<T>,
			source_asset_id: AssetIdOf<T>,
			decimal_ratio: AssetBalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			T::ManagerOrigin::ensure_origin(origin)?;
			EnabledTokens::<T>::insert(&source_asset_id, decimal_ratio);
			Self::deposit_event(Event::AssetEnabled { asset_id: source_asset_id, decimal_ratio });
			Ok(Pays::No.into())
		}

		/// disable a specific type of token available for switching
		#[pallet::call_index(3)]
		#[pallet::weight({195_000_000})]
		pub fn disable_token(
			origin: OriginFor<T>,
			source_asset_id: AssetIdOf<T>,
		) -> DispatchResultWithPostInfo {
			T::ManagerOrigin::ensure_origin(origin)?;
			EnabledTokens::<T>::remove(&source_asset_id);
			Self::deposit_event(Event::AssetDisabled { asset_id: source_asset_id });
			Ok(Pays::No.into())
		}
	}
}
