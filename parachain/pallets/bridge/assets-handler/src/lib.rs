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

//! A pallet for handling bridge transfer lgoic.
#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
use frame_support::{
	pallet_prelude::*,
	traits::{
		tokens::{
			fungible::{Inspect as FInspect, Mutate as FMutate},
			fungibles::Mutate as FsMutate,
			Fortitude, Precision,
		},
		StorageVersion,
	},
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use parity_scale_codec::FullCodec;
type BalanceOf<T> =
	<pallet_balances::Pallet<T> as FInspect<<T as frame_system::Config>::AccountId>>::Balance;
use pallet_bridge_common::BridgeHandler;
use pallet_parachain_staking::IssuanceAdapter;
use sp_runtime::{
	traits::{CheckedAdd, CheckedSub},
	ArithmeticError, DispatchError, FixedPointOperand,
};
use sp_std::{fmt::Debug, prelude::*};
type ResourceId = pallet_chain_bridge::ResourceId;
use pallet_bridge_common::AssetInfo;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);
	pub type AssetId<T> = <T as pallet_assets::Config>::AssetId;

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_balances::Config
		+ pallet_assets::Config
		+ pallet_chain_bridge::Config
	{
		/// Overarching event type
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Treasury account to receive assets fee
		type TreasuryAccount: Get<Self::AccountId>;

		/// The privileged origin to call update_maximum_issuance
		type SetMaximumIssuanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		#[pallet::constant]
		type DefaultMaximumIssuance: Get<BalanceOf<Self>>;

		#[pallet::constant]
		// In parachain local decimal format
		type ExternalTotalIssuance: Get<BalanceOf<Self>>;
	}

	// Resource Id of pallet assets token
	#[pallet::storage]
	#[pallet::getter(fn resource_to_asset_info)]
	pub type ResourceToAssetInfo<T: Config> =
		StorageMap<_, Twox64Concat, ResourceId, AssetInfo<AssetId<T>, BalanceOf<T>>, OptionQuery>;

	#[pallet::type_value]
	pub fn DefaultExternalBalances<T: Config>() -> BalanceOf<T> {
		T::ExternalTotalIssuance::get()
			.checked_sub(&pallet_balances::Pallet::<T>::total_issuance())
			.map_or_else(|| 0u32.into(), |v| v)
	}

	// Native Token External Unlocked Total Balances of outside
	#[pallet::storage]
	#[pallet::getter(fn external_balances)]
	pub type ExternalBalances<T: Config> =
		StorageValue<_, BalanceOf<T>, ValueQuery, DefaultExternalBalances<T>>;

	// Native Token Maximum Issuance Limit from outside remote chain
	#[pallet::storage]
	#[pallet::getter(fn maximum_issuance)]
	pub type MaximumIssuance<T: Config> =
		StorageValue<_, BalanceOf<T>, ValueQuery, T::DefaultMaximumIssuance>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// asset id = None means native token
		ResourceUpdated {
			resource_id: ResourceId,
			asset: AssetInfo<AssetId<T>, BalanceOf<T>>,
		},
		ResourceRemoved {
			resource_id: ResourceId,
		},
		/// A certain amount of asset tokens was minted
		TokenBridgeIn {
			asset_id: Option<AssetId<T>>,
			to: T::AccountId,
			amount: BalanceOf<T>,
		},
		TokenBridgeOut {
			asset_id: Option<AssetId<T>>,
			from: T::AccountId,
			// Before Fee
			amount: BalanceOf<T>,
			fee: BalanceOf<T>,
		},
		/// MaximumIssuance was changed
		MaximumIssuanceChanged {
			old_value: BalanceOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidResourceId,
		CannotPayAsFee,
		ReachMaximumSupply,
		Overflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Stores an asset id on chain under an associated resource ID.
		#[pallet::call_index(0)]
		#[pallet::weight({1000})]
		pub fn set_resource(
			origin: OriginFor<T>,
			resource_id: ResourceId,
			asset: AssetInfo<AssetId<T>, BalanceOf<T>>,
		) -> DispatchResult {
			<T as pallet_chain_bridge::Config>::BridgeCommitteeOrigin::ensure_origin(origin)?;
			ResourceToAssetInfo::<T>::insert(resource_id, asset.clone());
			Self::deposit_event(Event::ResourceUpdated { resource_id, asset });
			Ok(())
		}

		/// Removes a resource ID from the resource mapping.
		///
		/// After this call, bridge transfers with the associated resource ID will
		/// be rejected.
		#[pallet::call_index(1)]
		#[pallet::weight({1000})]
		pub fn remove_resource(origin: OriginFor<T>, resource_id: ResourceId) -> DispatchResult {
			<T as pallet_chain_bridge::Config>::BridgeCommitteeOrigin::ensure_origin(origin)?;
			ResourceToAssetInfo::<T>::remove(resource_id);
			Self::deposit_event(Event::ResourceRemoved { resource_id });
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight({1000})]
		pub fn set_maximum_issuance(
			origin: OriginFor<T>,
			maximum_issuance: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			T::SetMaximumIssuanceOrigin::ensure_origin(origin)?;
			Self::deposit_event(Event::MaximumIssuanceChanged {
				old_value: MaximumIssuance::<T>::get(),
			});
			MaximumIssuance::<T>::set(maximum_issuance);
			Ok(Pays::No.into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight({1000})]
		pub fn set_external_balances(
			origin: OriginFor<T>,
			external_balances: BalanceOf<T>,
		) -> DispatchResult {
			frame_system::ensure_root(origin)?;
			<ExternalBalances<T>>::put(external_balances);
			Ok(())
		}
	}
}

impl<T, B, A> BridgeHandler<B, A, ResourceId> for Pallet<T>
where
	T: Config
		+ frame_system::Config<AccountId = A>
		+ pallet_chain_bridge::Config<Balance = B>
		+ pallet_assets::Config<Balance = B>
		+ pallet_balances::Config<Balance = B>,
	B: Copy
		+ FixedPointOperand
		+ CheckedSub
		+ CheckedAdd
		+ MaybeSerializeDeserialize
		+ Debug
		+ FullCodec
		+ 'static,
	A: Clone + std::cmp::Eq,
{
	fn prepare_token_bridge_in(
		resource_id: ResourceId,
		who: A,
		amount: B,
	) -> Result<B, DispatchError> {
		let asset_info = Self::resource_to_asset_info(resource_id);
		match asset_info {
			None => Err(Error::<T>::InvalidResourceId.into()),
			// Native token
			Some(AssetInfo { fee: _, asset: None }) => {
				// Native token require maximum issuance check
				let total_issuance: BalanceOf<T> = pallet_balances::Pallet::<T>::total_issuance();
				let new_issuance =
					total_issuance.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
				if new_issuance > MaximumIssuance::<T>::get() {
					return Err(Error::<T>::ReachMaximumSupply.into());
				}
				// Native token require external balance modification
				let external_balances = <ExternalBalances<T>>::get()
					.checked_sub(&amount)
					.ok_or(Error::<T>::Overflow)?;
				<ExternalBalances<T>>::put(external_balances);

				Self::deposit_event(Event::TokenBridgeIn {
					asset_id: None,
					to: who.clone(),
					amount,
				});
				pallet_balances::Pallet::<T>::mint_into(&who, amount)
			},
			// pallet assets
			Some(AssetInfo { fee: _, asset: Some(asset) }) => {
				Self::deposit_event(Event::TokenBridgeIn {
					asset_id: Some(asset.clone()),
					to: who.clone(),
					amount,
				});
				pallet_assets::Pallet::<T>::mint_into(asset, &who, amount)
			},
		}
	}
	// Return actual amount to target chain after deduction e.g fee
	fn prepare_token_bridge_out(
		resource_id: ResourceId,
		who: A,
		amount: B,
	) -> Result<B, DispatchError> {
		let asset_info = Self::resource_to_asset_info(resource_id);
		match asset_info {
			None => Err(Error::<T>::InvalidResourceId.into()),
			// Native token
			Some(AssetInfo { fee, asset: None }) => {
				let burn_amount = pallet_balances::Pallet::<T>::burn_from(
					&who,
					amount,
					Precision::Exact,
					Fortitude::Polite,
				)?;
				ensure!(burn_amount > fee, Error::<T>::CannotPayAsFee);

				// Native token require external balance modification
				let external_balances = <ExternalBalances<T>>::get()
					.checked_add(&burn_amount)
					.ok_or(Error::<T>::Overflow)?;
				<ExternalBalances<T>>::put(external_balances);

				pallet_balances::Pallet::<T>::mint_into(&T::TreasuryAccount::get(), fee)?;
				Self::deposit_event(Event::TokenBridgeOut {
					asset_id: None,
					from: who,
					amount: burn_amount,
					fee,
				});
				Ok(burn_amount.checked_sub(&fee).ok_or(ArithmeticError::Overflow)?)
			},
			// pallet assets
			Some(AssetInfo { fee, asset: Some(asset) }) => {
				// Since we use Exact approach
				// Burn amount will always be amount exactly
				// Otherwise
				let burn_amount = pallet_assets::Pallet::<T>::burn_from(
					asset.clone(),
					&who,
					amount,
					Precision::Exact,
					Fortitude::Polite,
				)?;
				ensure!(burn_amount > fee, Error::<T>::CannotPayAsFee);
				pallet_assets::Pallet::<T>::mint_into(
					asset.clone(),
					&T::TreasuryAccount::get(),
					fee,
				)?;
				Self::deposit_event(Event::TokenBridgeOut {
					asset_id: Some(asset),
					from: who,
					amount: burn_amount,
					fee,
				});
				Ok(burn_amount.checked_sub(&fee).ok_or(ArithmeticError::Overflow)?)
			},
		}
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn setup_native_asset_info(resource_id: ResourceId, fee: B) -> DispatchResult {
		let native_token_asset_info: AssetInfo<AssetId<T>, B> = AssetInfo { fee, asset: None };

		ResourceToAssetInfo::<T>::insert(resource_id, native_token_asset_info.clone());
		Self::deposit_event(Event::ResourceUpdated { resource_id, asset: native_token_asset_info });
		Ok(())
	}
}

impl<T: Config> IssuanceAdapter<BalanceOf<T>> for Pallet<T> {
	fn adapted_total_issuance() -> BalanceOf<T> {
		<ExternalBalances<T>>::get()
	}
}
