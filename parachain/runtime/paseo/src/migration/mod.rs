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

use frame_support::traits::{
	Get, GetStorageVersion, OnRuntimeUpgrade, PalletInfoAccess, StorageVersion,
};
use log;
use sp_std::marker::PhantomData;

pub type Migrations<Runtime> = (IdentityUpdateStorageVersion<Runtime>,);

const IDENTITY_LOG_TARGET: &str = "runtime::identity";
pub struct IdentityUpdateStorageVersion<T>(PhantomData<T>);
impl<T> OnRuntimeUpgrade for IdentityUpdateStorageVersion<T>
where
	T: frame_system::Config + pallet_identity::Config,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		ensure!(
			StorageVersion::get::<pallet_identity::Pallet<T>>() == 1,
			"Current pallet_identity StorageVersion is not 1."
		);
		Ok(Vec::<u8>::new())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let on_chain_version = pallet_identity::Pallet::<T>::on_chain_storage_version();

		if on_chain_version == 1 {
			frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
				pallet_identity::Pallet::<T>::name().as_bytes(),
				"StorageVersion".as_bytes(),
			));

			StorageVersion::new(2).put::<pallet_identity::Pallet<T>>();

			log::info!(target: IDENTITY_LOG_TARGET, "Upgrade pallet_identity StorageVersion to version 2 successfully");
			T::DbWeight::get().reads_writes(1, 3)
		} else {
			log::info!(
				target: IDENTITY_LOG_TARGET,
				"Migration did not execute. This probably should be removed."
			);
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		ensure!(
			StorageVersion::get::<pallet_identity::Pallet<T>>() == 2,
			"Please upgrade pallet_identity StorageVersion to 2"
		);
		Ok(())
	}
}
