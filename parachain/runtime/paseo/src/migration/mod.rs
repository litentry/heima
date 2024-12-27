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

// By the time of this migration on Paseo 9221 (polkadot stable2407)
// The current storage version:      pallet version:
// xcmpQueue:                -                     3
// transactionPayment:       V2                    0
// vesting:                  V1                    0

// Our target  storage version:      pallet version: (stable2407)
// xcmpQueue:                -                     3 => 5
// transactionPayment:       V2                    0
// vesting:                  V1                    0

// In try-runtime, current implementation, the storage version is not checked,
// Pallet version is used instead.

use frame_support::traits::OnRuntimeUpgrade;
use sp_std::marker::PhantomData;

pub struct MigrateStorageVersionPatch<T>(PhantomData<T>);
impl<T> OnRuntimeUpgrade for MigrateStorageVersionPatch<T>
where
	T: frame_system::Config
		+ cumulus_pallet_xcmp_queue::Config
		+ cumulus_pallet_xcmp_queue::migration::v5::V5Config,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<alloc::vec::Vec<u8>, sp_runtime::TryRuntimeError> {
		// Only checking the lowest storage version
		let xcmp_queue_storage_version_check =
			cumulus_pallet_xcmp_queue::migration::v4::MigrationToV4::<T>::pre_upgrade()?;

		Ok(VersionedPostUpgradeData::MigrationExecuted(Inner::pre_upgrade()?).encode())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let mut weight = frame_support::weights::Weight::from_parts(0, 0);
		// V3 to V4
		// XCMP QueueConfig has different default value
		// Migration targeting at changing storage value to new default value if old value matched
		// Our current Paseo setting has already hard-coded
		// This migration should have no effect except bumping storage version
		weight +=
			cumulus_pallet_xcmp_queue::migration::v4::MigrationToV4::<T>::on_runtime_upgrade();
		// V4 to V5
		// Did nothing to storage
		// Just checking MaxActiveOutboundChannels is not exceeded
		// Our current Paseo Storage is 0
		// This migration should have no effect except bumping storage version
		weight +=
			cumulus_pallet_xcmp_queue::migration::v5::MigrateV4ToV5::<T>::on_runtime_upgrade();

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(
		versioned_post_upgrade_data_bytes: alloc::vec::Vec<u8>,
	) -> Result<(), sp_runtime::TryRuntimeError> {
		// Only checking the lowest storage version
		cumulus_pallet_xcmp_queue::migration::v4::MigrationToV4::<T>::post_upgrade(
			versioned_post_upgrade_data_bytes,
		)
	}
}
