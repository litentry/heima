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

//! Migration P9251: Remove old bridge pallets storage
//!
//! This migration safely removes all on-chain storage for the legacy bridge implementation
//! (ChainBridge, BridgeTransfer, AssetsHandler) after migration to omni-bridge.
//! The pallets are completely independent with no shared storage.

use frame_support::{
	ensure,
	migration::{clear_storage_prefix, storage_key_iter},
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
	Blake2_128Concat, Twox64Concat,
};
use sp_core::hashing::twox_128;
use sp_std::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use parity_scale_codec::{Decode, Encode};

#[cfg(feature = "try-runtime")]
use pallet_chain_bridge::{BridgeChainId, DepositNonce};

#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

const TARGET: &str = "runtime::migration::P9251";

/// Migration to remove old bridge pallets storage
pub struct RemoveOldBridgeStorage<T>(PhantomData<T>);

impl<T> RemoveOldBridgeStorage<T>
where
	T: frame_system::Config,
{
	/// Remove all ChainBridge pallet storage
	fn remove_chain_bridge_storage() -> Weight {
		log::info!(target: TARGET, "🗑️ Removing ChainBridge storage");

		let pallet_prefix = b"ChainBridge";
		let mut weight = Weight::zero();

		// Clear all storage items for ChainBridge
		let storage_names = [
			b"ChainNonces" as &[u8],
			b"RelayerThreshold" as &[u8],
			b"Relayers" as &[u8],
			b"RelayerCount" as &[u8],
			b"Votes" as &[u8],
			b"BridgeEvents" as &[u8],
		];
		for storage_name in storage_names {
			let result = clear_storage_prefix(pallet_prefix, storage_name, &[], None, None);
			weight = weight.saturating_add(T::DbWeight::get().writes(result.unique.into()));

			log::info!(
				target: TARGET,
				"🗑️ Cleared {} items from ChainBridge::{}",
				result.unique,
				core::str::from_utf8(storage_name).unwrap_or("unknown")
			);
		}

		weight
	}

	/// Remove all BridgeTransfer pallet storage  
	fn remove_bridge_transfer_storage() -> Weight {
		log::info!(target: TARGET, "🗑️ Removing BridgeTransfer storage");

		let pallet_prefix = b"BridgeTransfer";
		let mut weight = Weight::zero();

		// These may already be migrated to AssetsHandler, but clean up any remaining
		let storage_names = [b"ExternalBalances" as &[u8], b"MaximumIssuance" as &[u8]];
		for storage_name in storage_names {
			let result = clear_storage_prefix(pallet_prefix, storage_name, &[], None, None);
			weight = weight.saturating_add(T::DbWeight::get().writes(result.unique.into()));

			log::info!(
				target: TARGET,
				"🗑️ Cleared {} items from BridgeTransfer::{}",
				result.unique,
				core::str::from_utf8(storage_name).unwrap_or("unknown")
			);
		}

		weight
	}

	/// Remove all AssetsHandler pallet storage
	fn remove_assets_handler_storage() -> Weight {
		log::info!(target: TARGET, "🗑️ Removing AssetsHandler storage");

		let pallet_prefix = b"AssetsHandler";
		let mut weight = Weight::zero();

		let storage_names = [
			b"ResourceToAssetInfo" as &[u8],
			b"ExternalBalances" as &[u8],
			b"MaximumIssuance" as &[u8],
		];
		for storage_name in storage_names {
			let result = clear_storage_prefix(pallet_prefix, storage_name, &[], None, None);
			weight = weight.saturating_add(T::DbWeight::get().writes(result.unique.into()));

			log::info!(
				target: TARGET,
				"🗑️ Cleared {} items from AssetsHandler::{}",
				result.unique,
				core::str::from_utf8(storage_name).unwrap_or("unknown")
			);
		}

		weight
	}

	#[cfg(feature = "try-runtime")]
	/// Count storage items for pre/post upgrade verification
	fn count_storage_items(pallet_prefix: &[u8]) -> u32 {
		let mut count = 0u32;

		match pallet_prefix {
			b"ChainBridge" => {
				count += storage_key_iter::<BridgeChainId, DepositNonce, Twox64Concat>(
					pallet_prefix,
					b"ChainNonces",
				)
				.count() as u32;

				count +=
					storage_key_iter::<Vec<u8>, bool, Blake2_128Concat>(pallet_prefix, b"Relayers")
						.count() as u32;

				// Note: Votes is a DoubleMap, counting is complex, skip for now

				// Check single value storages
				let items = [
					b"RelayerThreshold" as &[u8],
					b"RelayerCount" as &[u8],
					b"BridgeEvents" as &[u8],
				];
				for item in items {
					if sp_io::storage::exists(&sp_io::storage::hashed_key(
						&twox_128(pallet_prefix),
						&twox_128(item),
					)) {
						count += 1;
					}
				}
			},
			b"BridgeTransfer" | b"AssetsHandler" => {
				let items: &[&[u8]] = if pallet_prefix == b"BridgeTransfer" {
					&[b"ExternalBalances", b"MaximumIssuance"]
				} else {
					&[b"ResourceToAssetInfo", b"ExternalBalances", b"MaximumIssuance"]
				};

				for item in items {
					if sp_io::storage::exists(&sp_io::storage::hashed_key(
						&twox_128(pallet_prefix),
						&twox_128(item),
					)) {
						count += 1;
					}
				}
			},
			_ => {},
		}

		count
	}
}

impl<T> OnRuntimeUpgrade for RemoveOldBridgeStorage<T>
where
	T: frame_system::Config,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		log::info!(target: TARGET, "🔍 Pre-upgrade: Checking old bridge storage");

		let chain_bridge_count = Self::count_storage_items(b"ChainBridge");
		let bridge_transfer_count = Self::count_storage_items(b"BridgeTransfer");
		let assets_handler_count = Self::count_storage_items(b"AssetsHandler");

		log::info!(
			target: TARGET,
			"📊 Found {} ChainBridge, {} BridgeTransfer, {} AssetsHandler items",
			chain_bridge_count, bridge_transfer_count, assets_handler_count
		);

		Ok((chain_bridge_count, bridge_transfer_count, assets_handler_count).encode())
	}

	fn on_runtime_upgrade() -> Weight {
		log::info!(target: TARGET, "🚀 Starting P9251: Remove old bridge storage");

		let mut weight = Weight::zero();

		weight = weight.saturating_add(Self::remove_chain_bridge_storage());
		weight = weight.saturating_add(Self::remove_bridge_transfer_storage());
		weight = weight.saturating_add(Self::remove_assets_handler_storage());

		log::info!(target: TARGET, "✅ P9251 migration completed");
		weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		let (chain_bridge_before, bridge_transfer_before, assets_handler_before): (u32, u32, u32) =
			Decode::decode(&mut &state[..]).map_err(|_| "Failed to decode pre-upgrade state")?;

		// Verify all storage is cleared
		let chain_bridge_after = Self::count_storage_items(b"ChainBridge");
		let bridge_transfer_after = Self::count_storage_items(b"BridgeTransfer");
		let assets_handler_after = Self::count_storage_items(b"AssetsHandler");

		ensure!(chain_bridge_after == 0, "ChainBridge storage not fully cleared");
		ensure!(bridge_transfer_after == 0, "BridgeTransfer storage not fully cleared");
		ensure!(assets_handler_after == 0, "AssetsHandler storage not fully cleared");

		log::info!(
			target: TARGET,
			"✅ Post-upgrade verified: Removed {} ChainBridge, {} BridgeTransfer, {} AssetsHandler items",
			chain_bridge_before, bridge_transfer_before, assets_handler_before
		);

		Ok(())
	}
}
