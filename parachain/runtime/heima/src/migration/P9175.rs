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
use frame_support::{
	traits::{Get, OnRuntimeUpgrade},
	StorageHasher, Twox128,
};
use sp_std::marker::PhantomData;
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

pub struct RemoveSudoAndStorage<T>(PhantomData<T>);
impl<T> OnRuntimeUpgrade for RemoveSudoAndStorage<T>
where
	T: frame_system::Config,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		log::info!("Pre check pallet Sudo exists");
		assert!(
			frame_support::storage::migration::have_storage_value(b"Sudo", b"Key", b"",),
			"Storage query fails: Sudo Key"
		);
		Ok(Vec::<u8>::new())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		use sp_io::KillStorageResult;
		// Remove Sudo Storage
		// TODO: Very Weak safety
		let entries: u64 = 4 + 100;
		let _res: KillStorageResult = frame_support::storage::unhashed::clear_prefix(
			&Twox128::hash(b"Sudo"),
			Some(entries.try_into().unwrap()),
			None,
		)
		.into();
		<T as frame_system::Config>::DbWeight::get().writes(entries)
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		use sp_io::KillStorageResult;

		log::info!("Post check Sudo");
		let res: KillStorageResult =
			frame_support::storage::unhashed::clear_prefix(&Twox128::hash(b"Sudo"), Some(0), None)
				.into();

		match res {
			KillStorageResult::AllRemoved(0) | KillStorageResult::SomeRemaining(0) => {},
			KillStorageResult::AllRemoved(n) | KillStorageResult::SomeRemaining(n) => {
				log::error!("Remaining entries: {:?}", n);
				return Err("Sudo not removed")
			},
		};

		Ok(())
	}
}
