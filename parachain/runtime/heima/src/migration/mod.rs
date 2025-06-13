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

#[cfg(feature = "try-runtime")]
use frame_support::ensure;
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;
#[cfg(feature = "try-runtime")]
use frame_support::traits::StorageVersion;
#[cfg(feature = "try-runtime")]
use sp_runtime::DispatchError;

use frame_support::{
	pallet_prelude::*,
	storage::migration::{clear_storage_prefix, put_storage_value},
	traits::{Get, GetStorageVersion, OnRuntimeUpgrade, PalletInfoAccess},
	weights::Weight,
};
use frame_system::pallet_prelude::BlockNumberFor;
use log;
use pallet_xcm::QueryStatus;
use sp_io::hashing::blake2_128;
use sp_std::marker::PhantomData;
use xcm::v5::{Junctions, Location};
use xcm::VersionedLocation;

pub type Migrations<Runtime> = (IdentityUpdateStorageVersion<Runtime>, RewriteXcmQueries<Runtime>);

const IDENTITY_LOG_TARGET: &str = "runtime::identity";
const XCM_LOG_TARGET: &str = "runtime::xcm";

pub struct IdentityUpdateStorageVersion<T>(PhantomData<T>);
impl<T> OnRuntimeUpgrade for IdentityUpdateStorageVersion<T>
where
	T: frame_system::Config + pallet_identity::Config,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, DispatchError> {
		ensure!(
			StorageVersion::get::<pallet_identity::Pallet<T>>() == 1,
			"Current pallet_identity StorageVersion is not 1."
		);
		Ok(Vec::<u8>::new())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let on_chain_version = pallet_identity::Pallet::<T>::on_chain_storage_version();
		let in_code_version = pallet_identity::Pallet::<T>::in_code_storage_version();

		if on_chain_version < in_code_version {
			frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
				pallet_identity::Pallet::<T>::name().as_bytes(),
				"StorageVersion".as_bytes(),
			));

			in_code_version.put::<pallet_identity::Pallet<T>>();

			log::info!(target: IDENTITY_LOG_TARGET, "Upgrade pallet_identity StorageVersion to version 2 successfully");
			T::DbWeight::get().reads_writes(1, 3)
		} else {
			log::info!(
				target: IDENTITY_LOG_TARGET,
				"No migration executed. This probably should be removed."
			);
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), DispatchError> {
		ensure!(
			StorageVersion::get::<pallet_identity::Pallet<T>>() == 2,
			"Please upgrade pallet_identity StorageVersion to 2"
		);
		Ok(())
	}
}

pub struct RewriteXcmQueries<T>(PhantomData<T>);
impl<T> OnRuntimeUpgrade for RewriteXcmQueries<T>
where
	T: frame_system::Config + pallet_xcm::Config,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, DispatchError> {
		log::info!(
			target: "runtime::xcm",
			"Starting pre-upgrade check for PolkadotXcm::Queries migration"
		);

		Ok(Vec::new()) // No state needed for this simple check
	}

	fn on_runtime_upgrade() -> Weight {
		let mut cursor = None;
		loop {
			let result =
				clear_storage_prefix(b"PolkadotXcm", b"Queries", &[], Some(100), cursor.as_deref());
			if let Some(next_cursor) = result.maybe_cursor {
				cursor = Some(next_cursor);
			} else {
				break;
			}
		}

		// There is only one known entry. But it's v2, which is already removed from stable2412.
		// So here manually insert that entry.
		let known_query_id: u64 = 0;
		let known_new_value: QueryStatus<BlockNumberFor<T>> = QueryStatus::VersionNotifier {
			origin: VersionedLocation::V5(Location { parents: 0, interior: Junctions::Here }),
			is_active: true,
		};

		// Build Blake2_128Concat key
		let encoded_key = known_query_id.encode();
		let mut final_key = blake2_128(&encoded_key).to_vec();
		final_key.extend(encoded_key);

		put_storage_value(b"PolkadotXcm", b"Queries", &final_key, known_new_value);

		log::info!(
			target: XCM_LOG_TARGET,
			"Removed PolkadotXcm::Queries entries: {:?}, and inserted new known QueryId={:?}",
			1, known_query_id
		);

		T::DbWeight::get().reads_writes(1, 1)
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), DispatchError> {
		log::info!(
			target: "runtime::xcm",
			"Post-upgrade: Checking PolkadotXcm::Queries migration"
		);

		let expected_query_id: u64 = 0;
		if let Some(status) = pallet_xcm::Pallet::<T>::query(expected_query_id) {
			log::info!(
				target: "runtime::xcm",
				"Post-upgrade: Query ID: {}, Status: {:?}",
				expected_query_id,
				status
			);
		} else {
			return Err(DispatchError::Other("Expected QueryId 0 not found in post-upgrade"));
		}

		Ok(())
	}
}
