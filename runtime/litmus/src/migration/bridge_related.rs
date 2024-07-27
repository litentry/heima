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
#![allow(clippy::type_complexity)]

use frame_support::{
	migration::{clear_storage_prefix, storage_key_iter},
	pallet_prelude::*,
	traits::{Get, OnRuntimeUpgrade},
	Blake2_256, Twox64Concat,
};
use pallet_assets_handler::{AssetInfo, ExternalBalances, MaximumIssuance, ResourceToAssetInfo};
use pallet_balances::AccountData;
use pallet_bridge::{BridgeChainId, ResourceId};
use sp_std::{convert::TryInto, marker::PhantomData, vec::Vec};

pub const DECIMAL_CONVERTOR: u128 = 1_000_000u128;

use hex_literal::hex;
#[cfg(feature = "try-runtime")]
use parity_scale_codec::Encode;
use storage::migration::get_storage_value;

mod old {
	use super::*;
	#[frame_support::storage_alias]
	pub type BridgeBalances<T: pallet_bridge_transfer::Config> = StorageDoubleMap<
		pallet_bridge_transfer::Pallet<T>,
		Twox64Concat,
		ResourceId,
		Twox64Concat,
		<T as frame_system::Config>::AccountId,
		u128,
	>;

	#[frame_support::storage_alias]
	pub type Resources<T: pallet_bridge::Config> =
		StorageMap<pallet_bridge::Pallet<T>, Blake2_256, ResourceId, Vec<u8>>;

	#[frame_support::storage_alias]
	pub type BridgeFee<T: pallet_bridge::Config> =
		StorageMap<pallet_bridge::Pallet<T>, Twox64Concat, BridgeChainId, u128>;
}

type AssetId<T> = <T as pallet_assets::Config>::AssetId;
// bridge::derive_resource_id(1, &bridge::hashing::blake2_128(b"LIT"));
pub const native_token_resource_id: [u8; 32] =
	hex!("0000000000000000000000000000000a21dfe87028f214dd976be8479f5af001");

// Replace Frame System Storage for Decimal Change from 12 to 18
// Replace Balances Storage for Decimal Change from 12 to 18
pub struct ReplaceBridgeRelatedStorage<T>(PhantomData<T>);
impl<T> ReplaceBridgeRelatedStorage<T>
where
	T: frame_system::Config<AccountData = AccountData<u128>>
		+ pallet_balances::Config<Balance = u128>,
{
	pub fn relocate_resource_fee_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceBridgeRelatedStorage",
			"running migration to Bridge Resources/Bridge Fee"
		);

		// We get only one resource registed
		// Which is native tokrn
		let pallet_prefix: &[u8] = b"Bridge";
		let storage_item_prefix_resources: &[u8] = b"Resources";
		let stored_data_resources: Vec<_> = storage_key_iter::<ResourceId, Vec<u8>, Blake2_256>(
			pallet_prefix,
			storage_item_prefix_resources,
		)
		.collect();
		let migrated_count_resources = frame_support::weights::Weight::from_parts(
			0,
			stored_data_resources
				.len()
				.try_into()
				.expect("There are between 0 and 2**64 mappings stored."),
		);
		assert_eq!(migrated_count_resources, 1);
		// Now remove the old storage
		// https://crates.parity.io/frame_support/storage/migration/fn.clear_storage_prefix.html
		let _ = clear_storage_prefix(pallet_prefix, storage_item_prefix_resources, &[], None, None);

		// This is fee for native token
		// There should be only 1 item
		let storage_item_prefix_fee: &[u8] = b"BridgeFee";
		let stored_data_fee: Vec<_> = storage_key_iter::<BridgeChainId, u128, Twox64Concat>(
			pallet_prefix,
			storage_item_prefix_fee,
		)
		.collect();
		let migrated_count_fee = frame_support::weights::Weight::from_parts(
			0,
			stored_data_fee
				.len()
				.try_into()
				.expect("There are between 0 and 2**64 mappings stored."),
		);
		assert_eq!(migrated_count_fee, 1);
		let _ = clear_storage_prefix(pallet_prefix, storage_item_prefix_fee, &[], None, None);

		// Replace into new storage of AssetsHandler
		let resource_id: ResourceId = stored_data_resources.0 .0;
		let fee: u128 = stored_data_fee.0 .1.saturating_mul(DECIMAL_CONVERTOR);
		let asset_info: AssetInfo<AssetId<T>, u128> = AssetInfo {
			fee,
			asset: None, // None for native token Asset Id
		};
		<ResourceToAssetInfo<T>>::insert(&resource_id, asset_info);

		let weight = T::DbWeight::get();
		migrated_count_resources
			.saturating_add(migrated_count_fee)
			.saturating_mul(weight.write + weight.read)
	}
	pub fn delete_bridge_balances_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceBridgeRelatedStorage",
			"running migration to Bridge Transfer Bridge Balances"
		);

		let result = <old::BridgeBalances<T>>::clear(u32::Max, None);

		let weight = T::DbWeight::get();
		frame_support::weights::Weight::from_parts(
			0,
			result.unique.into().saturating_mul(weight.write + weight.read),
		)
	}
	pub fn relocate_external_balance_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceBridgeRelatedStorage",
			"running migration to ExternalBalances"
		);
		let pallet_prefix: &[u8] = b"BridgeTransfer";
		let storage_item_prefix: &[u8] = b"ExternalBalances";
		let stored_data = get_storage_value::<u128>(pallet_prefix, storage_item_prefix, b"")
			.expect("Storage query fails: BridgeTransfer ExternalBalances");
		let _ = clear_storage_prefix(pallet_prefix, storage_item_prefix, &[], None, None);

		<ExternalBalances<T>>::put(stored_data.saturating_mul(DECIMAL_CONVERTOR.into()));
		let weight = T::DbWeight::get();
		frame_support::weights::Weight::from_parts(0, weight.write + weight.read)
	}
	pub fn relocate_maximum_issuance_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceBridgeRelatedStorage",
			"running migration to MaximumIssuance"
		);
		let pallet_prefix: &[u8] = b"BridgeTransfer";
		let storage_item_prefix: &[u8] = b"MaximumIssuance";
		let stored_data = get_storage_value::<u128>(pallet_prefix, storage_item_prefix, b"")
			.expect("Storage query fails: BridgeTransfer MaximumIssuance");
		let _ = clear_storage_prefix(pallet_prefix, storage_item_prefix, &[], None, None);

		<MaximumIssuance<T>>::put(stored_data.saturating_mul(DECIMAL_CONVERTOR.into()));
		let weight = T::DbWeight::get();
		frame_support::weights::Weight::from_parts(0, weight.write + weight.read)
	}
}

#[cfg(feature = "try-runtime")]
impl<T> ReplaceBridgeRelatedStorage<T>
where
	T: frame_system::Config<AccountData = AccountData<u128>>
		+ pallet_balances::Config<Balance = u128>,
{
	pub fn pre_upgrade_resource_fee_storage() -> Result<Vec<u8>, &'static str> {
		let resources_iter = <old::Resources<T>>::iter();
		assert_eq!(
			resources_iter.next(),
			Some((native_token_resource_id, b"BridgeTransfer.transfer".to_vec()))
		);
		assert!(resources_iter.next().is_none());

		let fee_iter = <old::BridgeFee<T>>::iter();
		// Just For Reference
		// Ethereum: chain_id=0
		// substrate_Litmus: chain_id=1
		// substrate_Litentry:chain_id=2
		// Rinkeby: chain_id=4
		// substrate_Rococo:chain_id=3
		// substrate_Stage: chain_id=5
		// Goerli: chain_id=6
		assert_eq!(fee_iter.next(), Some(0u8, 16_000_000_000_000u128));
		assert!(fee_iter.next().is_none());

		Ok(Vec::new())
	}
	pub fn post_upgrade_resource_fee_storage(_state: Vec<u8>) -> Result<(), &'static str> {
		let resources_iter = <old::Resources<T>>::iter();
		assert_eq!(resources_iter.next(), None);

		let fee_iter = <old::BridgeFee<T>>::iter();
		assert_eq!(fee_iter.next(), None);

		// Check AssetsHandler Storage
		let new_resource_fee_iter = <ResourceToAssetInfo<T>>::iter();
		let expected_asset_info = AssetInfo {
			fee: 16_000_000_000_000u128.saturating_mul(DECIMAL_CONVERTOR),
			asset: None,
		};
		assert_eq!(
			new_resource_fee_iter.next(),
			Some((native_token_resource_id, expected_asset_info))
		);
		Ok(())
	}
	pub fn pre_upgrade_bridge_balances_storage() -> Result<Vec<u8>, &'static str> {
		Ok(Vec::new())
	}
	pub fn post_upgrade_bridge_balances_storage(_state: Vec<u8>) -> Result<(), &'static str> {
		assert!(<old::BridgeBalances<T>>::iter().next().is_none());
		Ok(())
	}
	pub fn pre_upgrade_external_balance_storage() -> Result<Vec<u8>, &'static str> {
		let pallet_prefix: &[u8] = b"BridgeTransfer";
		let storage_item_prefix: &[u8] = b"ExternalBalances";
		let stored_data = get_storage_value::<u128>(pallet_prefix, storage_item_prefix, b"")
			.expect("Storage query fails: BridgeTransfer ExternalBalances");
		Ok(stored_data.saturating_mul(DECIMAL_CONVERTOR).encode())
	}
	pub fn post_upgrade_external_balance_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_state = u128::decode(&mut &state[..])
			.map_err(|_| "Failed to decode BridgeTransfer ExternalBalances")?;

		let pallet_prefix: &[u8] = b"AssetsHandler";
		let storage_item_prefix: &[u8] = b"ExternalBalances";
		let actual_state = get_storage_value::<u128>(pallet_prefix, storage_item_prefix, b"")
			.expect("Storage query fails: BridgeTransfer ExternalBalances");
		assert_eq!(expected_state, actual_state);
		Ok(())
	}
	pub fn pre_upgrade_maximum_issuance_storage() -> Result<Vec<u8>, &'static str> {
		let pallet_prefix: &[u8] = b"BridgeTransfer";
		let storage_item_prefix: &[u8] = b"MaximumIssuance";
		let stored_data = get_storage_value::<u128>(pallet_prefix, storage_item_prefix, b"")
			.expect("Storage query fails: BridgeTransfer MaximumIssuance");
		Ok(stored_data.saturating_mul(DECIMAL_CONVERTOR).encode())
	}
	pub fn post_upgrade_maximum_issuance_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_state = u128::decode(&mut &state[..])
			.map_err(|_| "Failed to decode BridgeTransfer MaximumIssuance")?;

		let pallet_prefix: &[u8] = b"AssetsHandler";
		let storage_item_prefix: &[u8] = b"MaximumIssuance";
		let actual_state = get_storage_value::<u128>(pallet_prefix, storage_item_prefix, b"")
			.expect("Storage query fails: BridgeTransfer MaximumIssuance");
		assert_eq!(expected_state, actual_state);
		Ok(())
	}
}

impl<T> OnRuntimeUpgrade for ReplaceBridgeRelatedStorage<T>
where
	T: frame_system::Config<AccountData = AccountData<u128>>
		+ pallet_balances::Config<Balance = u128>,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		let _ = Self::pre_upgrade_resource_fee_storage()?;
		let _ = Self::pre_upgrade_bridge_balances_storage()?;

		let external_balances_vec = Self::pre_upgrade_external_balance_storage()?;
		let maximum_issuance_vec = Self::pre_upgrade_maximum_issuance_storage()?;

		Ok((external_balances_vec, maximum_issuance_vec).encode())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let mut weight = frame_support::weights::Weight::from_parts(0, 0);

		// Replace Old Bridge's Resources, BridgeFee with AssetsHandler's ResourceToAssetInfo
		weight += Self::relocate_resource_fee_storage();
		// Delete BridgeTransfer's Bridge Balances Storage
		weight += Self::delete_bridge_balances_storage();

		weight += Self::relocate_external_balance_storage();
		weight += Self::relocate_maximum_issuance_storage();
		weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		let pre_vec: (Vec<u8>, Vec<u8>) =
			Decode::decode(&mut &state[..]).map_err(|_| "Failed to decode Tuple")?;

		Self::post_upgrade_resource_fee_storage(Vec::<u8>::new())?;
		Self::post_upgrade_bridge_balances_storage(Vec::<u8>::new())?;

		Self::post_upgrade_external_balance_storage(pre_vec.0)?;
		Self::post_upgrade_maximum_issuance_storage(pre_vec.1)?;

		Ok(())
	}
}
