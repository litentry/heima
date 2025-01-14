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
// parachainIdentity         -                     0
// xcmpQueue:                -                     3
// transactionPayment:       V2                    0
// vesting:                  V1                    0

// Our target  storage version:      pallet version: (stable2407)
// parachainIdentity         -                     1
// xcmpQueue:                -                     3 => 5
// transactionPayment:       V2                    0
// vesting:                  V1                    0

// In try-runtime, current implementation, the storage version is not checked,
// Pallet version is used instead.
#[cfg(feature = "try-runtime")]
use frame_support::ensure;
use frame_support::traits::{
	Get, GetStorageVersion, OnRuntimeUpgrade, PalletInfoAccess, StorageVersion,
};
use sp_std::marker::PhantomData;
use sp_std::vec::Vec;

pub type Migrations<Runtime> = (
	// Identity V0 => V1
	// Our storage is empty
	// The official migration is for can old IdentityFor into new IdentityFor
	// Should do nothing but bump version storage for us
	IdentityUpdateStorageVersion<Runtime>,
	// V3 to V4
	// XCMP QueueConfig has different default value
	// Migration targeting at changing storage value to new default value if old value matched
	// Our current Paseo setting has already hard-coded
	// This migration should have no effect except bumping storage version
	cumulus_pallet_xcmp_queue::migration::v4::MigrationToV4<Runtime>,
	// V4 to V5
	// Did nothing to storage
	// Just checking MaxActiveOutboundChannels is not exceeded
	// Our current Paseo Storage is 0
	// This migration should have no effect except bumping storage version
	cumulus_pallet_xcmp_queue::migration::v5::MigrateV4ToV5<Runtime>,
);

const IDENTITY_LOG_TARGET: &str = "runtime::identity";
pub struct IdentityUpdateStorageVersion<T>(PhantomData<T>);
impl<T> OnRuntimeUpgrade for IdentityUpdateStorageVersion<T>
where
	T: frame_system::Config + pallet_identity::Config,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		ensure!(
			StorageVersion::get::<pallet_identity::Pallet<T>>() == 0,
			"Already upgrade to some non-zero version"
		);
		Ok(Vec::<u8>::new())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let on_chain_version = pallet_identity::Pallet::<T>::on_chain_storage_version();

		if on_chain_version == 0 {
			// Remove the old `StorageVersion` type.
			frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
				pallet_identity::Pallet::<T>::name().as_bytes(),
				"StorageVersion".as_bytes(),
			));

			// Set storage version to `1`.
			StorageVersion::new(1).put::<pallet_identity::Pallet<T>>();

			log::info!(target: IDENTITY_LOG_TARGET, "Storage to version 1");
			T::DbWeight::get().reads_writes(1, 3)
		} else {
			log::info!(
				target: IDENTITY_LOG_TARGET,
				"Migration did not execute. This probably should be removed"
			);
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		ensure!(StorageVersion::get::<pallet_identity::Pallet<T>>() == 1, "Must upgrade");
		Ok(())
	}
}
