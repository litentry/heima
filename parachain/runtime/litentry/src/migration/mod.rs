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

// By the time of this migration on Litentry 9220 (polkadot stable2407)
// The current        storage version:      pallet version:
// scheduler:                      -                     0
// multisig:                       -                     0
// preimage:                       -                     0
// balances:                       -                     0
// democracy:                      -                     0
// bounties:                       -                     0
// xcmpQueue:                      -                     3
// polkadotXcm:                    -                     0
// developerCommittee              -                     0
// developerCommitteeMembership    -                     0
// transactionPayment:             V1Ancient             0
// vesting:                        V1                    0

// Our target         storage version:      pallet version: (stable2407)
// scheduler:                      -                     0 => 4
// multisig:                       -                     0 => 1
// preimage:                       -                     0 => 1
// balances:                       -                     0 => 1
// democracy:                      -                     0 => 1
// bounties:                       -                     0 => 4
// xcmpQueue:                      -                     3 => 5
// polkadotXcm:                    -                     0 => 1
// developerCommittee              -                     0 => 4
// developerCommitteeMembership    -                     0 => 4
// transactionPayment:             V1Ancient => V2       0   (it is built by genesis, so maybe no fix is fine)
// vesting:                        V1                    0

// In try-runtime, current implementation, the storage version is not checked,
// Pallet version is used instead.
use frame_support::traits::{Get, OnRuntimeUpgrade};
use frame_system::pallet_prelude::{BlockNumberFor, StorageVersion};
use pallet_scheduler::Agenda;
use sp_std::marker::PhantomData;
#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

pub type Migrations<Runtime> = (
	// Scheduler V0 => V4
	// The official pallet does not provide any available migration
	// We, Litentry Storage have two old unexecuted expired root tasks.
	// This storage should be clean up and update storage version to V4 directly.
	// PS: Looks like two old tasks fits V2/V3 structure
	RemoveSchedulerOldStorage<Runtime>,
	// Multsig V0 => V1
	// Remove old unsettled call storage and do the refund
	// Does not matter if we do have old storage or not
	// Should simply works
	pallet_multisig::migrations::v1::MigrateToV1<Runtime>,
	// Preimage V0 => V1
	//
	pallet_preimage::migration::v1::Migration<Runtime>,
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

pub struct RemoveSchedulerOldStorage<T>(PhantomData<T>);
impl<T> OnRuntimeUpgrade for RemoveSchedulerOldStorage<T>
where
	T: frame_system::Config + pallet_scheduler::Config,
	BlockNumberFor<T>: From<u128>,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		log::info!("Pre check pallet scheduler storage only has two precise tasks leftover");
		for (when, vec_schedule) in <Agenda<T>>::iter() {
			assert!(
				when == 3067200u128.into() || when == 2995200u128.into(),
				"Extra schedule exists"
			);
		}
		Ok(Vec::<u8>::new())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		// Remove Scheduler Storage precisely of according block agenda only
		// TODO: Very Weak safety
		Agenda::<T>::remove(2995200u128.into());
		Agenda::<T>::remove(3067200u128.into());

		#[allow(deprecated)]
		frame_support::storage::migration::remove_storage_prefix(
			b"Scheduler",
			b"StorageVersion",
			&[],
		);
		StorageVersion::new(4).put::<pallet_scheduler::Pallet<T>>();
		<T as frame_system::Config>::DbWeight::get().reads_writes(2, 4)
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		use sp_io::KillStorageResult;
		for (when, vec_schedule) in <Agenda<T>>::iter() {
			assert!(
				when != 3067200u128.into() && when != 2995200u128.into(),
				"Old schedule still exists"
			);
		}

		ensure!(StorageVersion::get::<pallet_scheduler::Pallet<T>>() == 4, "Must upgrade");

		Ok(())
	}
}
