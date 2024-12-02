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

//! Inspired by:
//! - Moonbeam `pallet_asset_manager`
//! implementations.

// (1) The local asset support is removed. This pallet is for managing Xcm foreign asset
// specifically. (2) The mapping of AssetId and AssetType is now capable of manual choosen
// (3) Compatibility to orml_token instead of pallet_assets
// (4) Code for destroy asset is removed

//! TODO Doc comments for the pallet
//! # Asset Manager Pallet
//!
//! This pallet allows to register new assets if certain conditions are met
//! The main goal of this pallet is to allow Litentry to register XCM assets
//! and control the creation of foreign assets
//! The assumption is we work with AssetTypes, which can then be comverted to AssetIds
//!
//! This pallet has five storage items:
//! - AssetIdType: A mapping from AssetId->AssetType.
//! - AssetIdUnitsPerSecond: An AssetId->u128 mapping that holds how much each AssetId should be
//!   charged per unit of second, in the case such an Asset is received as a XCM asset.
//! - AssetTypeId: A mapping from AssetType -> AssetId.
//! - ForeignAssetTracker: The counter of foreign assets that have been created so far.
//! - AssetIdMetadata: An AssetId->AssetMetadata mapping that holds the metadata token info.
//!
//! This pallet has five extrinsics:
//! - register_foreign_asset: Register a foreign asset in this pallet.
//! - set_asset_units_per_second: Set the unit per second that should be charged for a particular
//!   asset.
//! - update_foreign_asset_metadata: Update the metadata of asset.
//! - add_asset_type: Add the correspondence between AssetId and AssetType.
//! - remove_asset_type: Remove the correspondence between AssetId and AssetType. At least one
//!   relation must exist.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::needless_borrows_for_generic_args)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
pub mod tests;
pub mod weights;

pub use crate::weights::WeightInfo;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency},
	transactional, PalletId,
};
use frame_system::pallet_prelude::*;
// use orml_traits::GetByKey;
pub use pallet::*;
use parity_scale_codec::HasCompact;
use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, One};
use sp_std::{convert::*, vec::Vec};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// The AssetManagers's pallet id
	pub const PALLET_ID: PalletId = PalletId(*b"asstmngr");

	pub(crate) type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Clone, Default, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
	pub struct AssetMetadata<Balance> {
		pub name: Vec<u8>,
		pub symbol: Vec<u8>,
		pub decimals: u8,
		pub minimal_balance: Balance,
		pub is_frozen: bool,
	}

	// Defines the trait to obtain a generic AssetType from a generic AssetId and viceversa
	pub trait AssetTypeGetter<AssetId, AssetType> {
		// Get asset type from assetId
		fn get_asset_type(asset_id: AssetId) -> Option<AssetType>;

		// Get assetId from assetType
		fn get_asset_id(asset_type: AssetType) -> Option<AssetId>;
	}

	// We implement this trait to be able to get the AssetType and units per second registered
	impl<T: Config> AssetTypeGetter<T::AssetId, T::ForeignAssetType> for Pallet<T> {
		fn get_asset_type(asset_id: T::AssetId) -> Option<T::ForeignAssetType> {
			AssetIdType::<T>::get(asset_id)
		}

		fn get_asset_id(asset_type: T::ForeignAssetType) -> Option<T::AssetId> {
			AssetTypeId::<T>::get(asset_type)
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The Asset Id. This will be used to register the asset in Assets
		type AssetId: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Default
			+ Ord
			+ PartialOrd
			+ Copy
			+ HasCompact
			+ MaxEncodedLen;

		/// The Foreign Asset Kind.
		type ForeignAssetType: Parameter + Member + Ord + PartialOrd;

		/// The units in which we record balances.
		type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;

		/// Origin that is allowed to create and modify asset information for foreign assets
		type ForeignAssetModifierOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The currency mechanism in which we reserve deposits for local assets.
		type Currency: ReservableCurrency<Self::AccountId>;

		type WeightInfo: WeightInfo;
	}

	/// An error that can occur while executing the mapping pallet's logic.
	#[pallet::error]
	pub enum Error<T> {
		AssetAlreadyExists,
		AssetTypeDoesNotExist,
		AssetIdDoesNotExist,
		DefaultAssetTypeRemoved,
		AssetIdLimitReached,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The foreign asset updated.
		ForeignAssetMetadataUpdated { asset_id: T::AssetId, metadata: AssetMetadata<BalanceOf<T>> },
		/// AssetTracker manipulated
		ForeignAssetTrackerUpdated { old_asset_tracker: T::AssetId, new_asset_tracker: T::AssetId },
		/// New asset with the asset manager is registered
		ForeignAssetTypeRegistered { asset_id: T::AssetId, asset_type: T::ForeignAssetType },
		/// New Event gives the info about involved asset_id, removed asset_type, and the new
		/// default asset_id and asset_type mapping after removal
		ForeignAssetTypeRemoved {
			asset_id: T::AssetId,
			removed_asset_type: T::ForeignAssetType,
			default_asset_type: T::ForeignAssetType,
		},
		/// Changed the amount of units we
		/// are charging per execution second for a given asset
		UnitsPerSecondChanged { asset_id: T::AssetId, units_per_second: u128 },
	}

	/// Mapping from an asset id to asset type.
	/// This is mostly used when receiving transaction specifying an asset directly,
	/// like transferring an asset from this chain to another.
	#[pallet::storage]
	#[pallet::getter(fn asset_id_type)]
	pub type AssetIdType<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetId, T::ForeignAssetType, OptionQuery>;

	/// Reverse mapping of AssetIdType. Mapping from an asset type to an asset id.
	/// This is mostly used when receiving a multilocation XCM message to retrieve
	/// the corresponding asset in which tokens should me minted.
	#[pallet::storage]
	#[pallet::getter(fn asset_type_id)]
	pub type AssetTypeId<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ForeignAssetType, T::AssetId, OptionQuery>;

	/// Stores the units per second for local execution for a AssetType.
	/// This is used to know how to charge for XCM execution in a particular asset
	/// Not all assets might contain units per second, hence the different storage
	#[pallet::storage]
	#[pallet::getter(fn asset_id_units_per_second)]
	pub type AssetIdUnitsPerSecond<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetId, u128, ValueQuery>;

	/// Stores the tracker of foreign assets id that have been
	/// created so far
	#[pallet::storage]
	#[pallet::getter(fn foreign_asset_tracker)]
	pub type ForeignAssetTracker<T: Config> = StorageValue<_, T::AssetId, ValueQuery>;

	/// The storages for AssetIdMetadata.
	/// AssetIdMetadata: map AssetId => Option<AssetMetadata>
	#[pallet::storage]
	#[pallet::getter(fn asset_metadatas)]
	pub type AssetIdMetadata<T: Config> =
		StorageMap<_, Twox64Concat, T::AssetId, AssetMetadata<BalanceOf<T>>, OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register new asset with the asset manager
		/// TODO::Reserve native token multilocation through GenesisBuild/RuntimeUpgrade
		/// TODO::Add Multilocation filter for register
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::register_foreign_asset_type())]
		#[transactional]
		pub fn register_foreign_asset_type(
			origin: OriginFor<T>,
			asset_type: T::ForeignAssetType,
			metadata: AssetMetadata<BalanceOf<T>>,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;
			ensure!(!AssetTypeId::<T>::contains_key(&asset_type), Error::<T>::AssetAlreadyExists);

			let asset_id = ForeignAssetTracker::<T>::get();

			AssetIdMetadata::<T>::insert(&asset_id, &metadata);
			AssetIdType::<T>::insert(&asset_id, &asset_type);
			AssetTypeId::<T>::insert(&asset_type, &asset_id);

			ForeignAssetTracker::<T>::put(
				asset_id.checked_add(&One::one()).ok_or(Error::<T>::AssetIdLimitReached)?,
			);

			Self::deposit_event(Event::<T>::ForeignAssetTypeRegistered { asset_id, asset_type });
			Self::deposit_event(Event::<T>::ForeignAssetMetadataUpdated { asset_id, metadata });

			Ok(())
		}

		// Update asset metadata
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::update_foreign_asset_metadata())]
		pub fn update_foreign_asset_metadata(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			metadata: AssetMetadata<BalanceOf<T>>,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;

			ensure!(AssetIdType::<T>::contains_key(&asset_id), Error::<T>::AssetIdDoesNotExist);
			AssetIdMetadata::<T>::insert(&asset_id, &metadata);

			Self::deposit_event(Event::<T>::ForeignAssetMetadataUpdated { asset_id, metadata });
			Ok(())
		}

		/// Change the amount of units we are charging per execution second
		/// for a given ForeignAssetType
		/// 0 means not support
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::set_asset_units_per_second())]
		pub fn set_asset_units_per_second(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			units_per_second: u128,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;

			ensure!(AssetIdType::<T>::contains_key(&asset_id), Error::<T>::AssetIdDoesNotExist);

			AssetIdUnitsPerSecond::<T>::insert(&asset_id, &units_per_second);

			Self::deposit_event(Event::UnitsPerSecondChanged { asset_id, units_per_second });
			Ok(())
		}

		/// Add the xcm type mapping for a existing assetId, other assetType still exists if any.
		/// TODO: Change add_asset_type with internal function wrapper
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::add_asset_type())]
		pub fn add_asset_type(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			new_asset_type: T::ForeignAssetType,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;

			// TODO: TBD, mutate replace insert + ensure! as asset_id is always an update
			ensure!(AssetIdType::<T>::contains_key(&asset_id), Error::<T>::AssetIdDoesNotExist);
			ensure!(
				!AssetTypeId::<T>::contains_key(&new_asset_type),
				Error::<T>::AssetAlreadyExists
			);

			// Insert new asset type info
			AssetIdType::<T>::insert(&asset_id, &new_asset_type);
			AssetTypeId::<T>::insert(&new_asset_type, &asset_id);

			Self::deposit_event(Event::ForeignAssetTypeRegistered {
				asset_id,
				asset_type: new_asset_type,
			});
			Ok(())
		}

		/// We do not allow the destroy of asset id so far; So at least one AssetTpye should be
		/// assigned to existing AssetId Both asset_type and potential new_default_asset_type must
		/// be an existing relation with asset_id
		/// TODO: Change remove_asset_type with internal function wrapper
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::remove_asset_type())]
		#[transactional]
		pub fn remove_asset_type(
			origin: OriginFor<T>,
			asset_type: T::ForeignAssetType,
			new_default_asset_type: Option<T::ForeignAssetType>,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;

			let asset_id =
				AssetTypeId::<T>::get(&asset_type).ok_or(Error::<T>::AssetTypeDoesNotExist)?;

			if let Some(i) = new_default_asset_type {
				// new default asset type must register already, and not belong to other asset_id
				ensure!(
					AssetTypeId::<T>::get(&i).ok_or(Error::<T>::AssetTypeDoesNotExist)?
						== asset_id.clone(),
					Error::<T>::AssetAlreadyExists
				);
				AssetIdType::<T>::insert(&asset_id, &i);
			}

			let default_asset_type: T::ForeignAssetType =
			    // This should be impossible to not get
				AssetIdType::<T>::get(&asset_id).ok_or(Error::<T>::AssetIdDoesNotExist)?;
			if default_asset_type == asset_type {
				return Err(Error::<T>::DefaultAssetTypeRemoved.into());
			}

			// Remove from AssetTypeId
			AssetTypeId::<T>::remove(&asset_type);

			Self::deposit_event(Event::ForeignAssetTypeRemoved {
				asset_id,
				removed_asset_type: asset_type,
				default_asset_type,
			});
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// The account ID of AssetManager
		pub fn account_id() -> T::AccountId {
			PALLET_ID.into_account_truncating()
		}
	}

	// impl<T: Config> GetByKey<T::AssetId, BalanceOf<T>> for Pallet<T> {
	// 	fn get(asset_id: &T::AssetId) -> BalanceOf<T> {
	// 		let metadata: AssetMetadata<BalanceOf<T>> =
	// 			AssetIdMetadata::<T>::get(asset_id).unwrap_or_default();
	// 		metadata.minimal_balance
	// 	}
	// }

	// AssetManager or other FeeToWeight source should implement this trait
	// Defines the trait to obtain the units per second of a give asset_type for local execution
	// This parameter will be used to charge for fees upon asset_type deposit
	pub trait UnitsToWeightRatio<AssetType> {
		// Whether payment in a particular asset_type is suppotrted
		fn payment_is_supported(asset_type: AssetType) -> bool;
		// Get units per second from asset type
		fn get_units_per_second(asset_type: AssetType) -> Option<u128>;
	}

	impl<T: Config> UnitsToWeightRatio<T::ForeignAssetType> for Pallet<T> {
		fn payment_is_supported(asset_type: T::ForeignAssetType) -> bool {
			if let Some(asset_id) = AssetTypeId::<T>::get(&asset_type) {
				if AssetIdUnitsPerSecond::<T>::get(asset_id) > 0 {
					return true;
				}
			}
			false
		}
		fn get_units_per_second(asset_type: T::ForeignAssetType) -> Option<u128> {
			if let Some(asset_id) = AssetTypeId::<T>::get(&asset_type) {
				let ups = AssetIdUnitsPerSecond::<T>::get(asset_id);
				if ups > 0 {
					return Some(ups);
				}
			}
			None
		}
	}
}
