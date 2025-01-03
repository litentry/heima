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
#[cfg(feature = "try-runtime")]
use frame_support::ensure;
use frame_support::traits::{
	Get, GetStorageVersion, OnRuntimeUpgrade, PalletInfoAccess, StorageVersion,
};
use frame_system::pallet_prelude::BlockNumberFor;
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
	// Clean correpted preimage storage
	RemoveCorreptedPreimageStorage<Runtime>,
	// Preimage V0 => V1
	// Migrate old StatusFor and PreimageFor Storage into new Storage
	pallet_preimage::migration::v1::Migration<Runtime>,
	// Balances V0 => V1
	// The official pallet migration is not need since we do not have any XCM deactive accounts
	// But our onchain inactiveIssuance storage of pallet_balance is non-negative
	// TODO: Where does this number come from?
	BalancesUpdateStorageVersionResetInactive<Runtime>,
	// Democracy V0 => V1
	// This migration only effects onging proposal/referedum, NextExternal
	// The referedum info's proposal hash is migrated if the hash is in old form (In our case, even for an onging one it will do nothing)
	pallet_democracy::migrations::v1::v1::Migration<Runtime>,
	// Bounties V0 => V4
	// The official migration does nothing but change pallet name and bump version
	// So we just bump version storage instead
	BountiesUpdateStorageVersion<Runtime>,
	// V3 to V4
	// XCMP QueueConfig has different default value
	// Migration targeting at changing storage value to new default value if old value matched
	// Our current Litentry setting has already hard-coded
	// This migration should have no effect except bumping storage version
	cumulus_pallet_xcmp_queue::migration::v4::MigrationToV4<Runtime>,
	// V4 to V5
	// Did nothing to storage
	// Just checking MaxActiveOutboundChannels is not exceeded
	// Our current Litentry Storage is 0
	// This migration should have no effect except bumping storage version
	cumulus_pallet_xcmp_queue::migration::v5::MigrateV4ToV5<Runtime>,
	// PolkadotXcm V0 => V1
	// Our storage is already correct
	// This migration is for can old weightInfo into new weightInfo form
	// Should do nothing but bump version storage for us
	pallet_xcm::migration::v1::MigrateToV1<Runtime>,
	// DeveloperCommittee V0 => V4
	// The official migration does nothing but change pallet name and bump version
	// So we just bump version storage instead
	DeveloperCommitteeUpdateStorageVersion<Runtime>,
	// DeveloperCommitteeMembership V0 => V4
	// The official migration does nothing but change pallet name and bump version
	// So we just bump version storage instead
	DeveloperCommitteeMembershipUpdateStorageVersion<Runtime>,
);

pub struct RemoveSchedulerOldStorage<T>(PhantomData<T>);
impl<T> OnRuntimeUpgrade for RemoveSchedulerOldStorage<T>
where
	T: frame_system::Config + pallet_scheduler::Config,
	BlockNumberFor<T>: From<u32>,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		log::info!("Pre check pallet scheduler storage only has two precise tasks leftover");
		let one: BlockNumberFor<T> = 3067200u32.into();
		let two: BlockNumberFor<T> = 2995200u32.into();
		for (when, _vec_schedule) in <Agenda<T>>::iter() {
			assert!(when == one || when == two, "Extra schedule exists");
		}
		Ok(Vec::<u8>::new())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		log::info!("Begin cleaning pallet scheduler storage two precise tasks leftover");
		// Remove Scheduler Storage precisely of according block agenda only
		// TODO: Very Weak safety
		let one: BlockNumberFor<T> = 3067200u32.into();
		let two: BlockNumberFor<T> = 2995200u32.into();
		Agenda::<T>::remove(one);
		Agenda::<T>::remove(two);

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
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		log::info!("Post check pallet scheduler storage two precise tasks cleaned");
		let one: BlockNumberFor<T> = 3067200u32.into();
		let two: BlockNumberFor<T> = 2995200u32.into();
		for (when, _vec_schedule) in <Agenda<T>>::iter() {
			assert!(when != one && when != two, "Old schedule still exists");
		}

		ensure!(StorageVersion::get::<pallet_scheduler::Pallet<T>>() == 4, "Must upgrade");

		Ok(())
	}
}

/// The original data layout of the preimage pallet without a specific version number.
mod preimage_helper {
	use alloc::collections::btree_map::BTreeMap;
	use frame_support::{pallet_prelude::*, storage_alias, traits::Currency};

	type BalanceOf<T> = <<T as pallet_preimage::Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	#[derive(Clone, Eq, PartialEq, Encode, Decode, TypeInfo, MaxEncodedLen, RuntimeDebug)]
	pub enum OldRequestStatus<AccountId, Balance> {
		Unrequested(Option<(AccountId, Balance)>),
		Requested(u32),
	}

	#[storage_alias]
	pub type PreimageFor<T: pallet_preimage::Config> = StorageMap<
		pallet_preimage::Pallet<T>,
		Identity,
		<T as frame_system::Config>::Hash,
		BoundedVec<u8, ConstU32<{ 4u32 * 1024u32 * 1024u32 }>>,
	>;

	#[storage_alias]
	pub type StatusFor<T: pallet_preimage::Config> = StorageMap<
		pallet_preimage::Pallet<T>,
		Identity,
		<T as frame_system::Config>::Hash,
		OldRequestStatus<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
	>;

	/// Returns the number of images or `None` if the storage is corrupted.
	#[cfg(feature = "try-runtime")]
	pub fn image_count<T: pallet_preimage::Config>() -> (u32, u32) {
		let images = PreimageFor::<T>::iter_values().count() as u32;
		let status = StatusFor::<T>::iter_values().count() as u32;

		(images, status)
	}
}

const PREIMAGE_LOG_TARGET: &str = "runtime::preimage";
pub struct RemoveCorreptedPreimageStorage<T>(PhantomData<T>);
impl<T> OnRuntimeUpgrade for RemoveCorreptedPreimageStorage<T>
where
	T: frame_system::Config + pallet_preimage::Config,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		let (image, status) = preimage_helper::image_count::<T>();
		assert!(image != status, "Preimage storage not correpted");
		Ok(Vec::<u8>::new())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		// Remove preimage correpted storage
		// TODO: Very Weak safety
		let mut weight = T::DbWeight::get().reads(1);
		if StorageVersion::get::<Pallet<T>>() != 0 {
			log::warn!(
				target: TARGET,
				"skipping MovePreimagesIntoBuckets: executed on wrong storage version.\
			Expected version 0"
			);
			return weight;
		}

		let status = preimage_helper::StatusFor::<T>::drain().collect::<Vec<_>>();
		weight.saturating_accrue(T::DbWeight::get().reads(status.len() as u64));

		let preimages = preimage_helper::PreimageFor::<T>::drain().collect::<BTreeMap<_, _>>();
		weight.saturating_accrue(T::DbWeight::get().reads(preimages.len() as u64));

		for (hash, status) in status.into_iter() {
			if preimages.get(&hash).is_none() {
				log::info!(target: PREIMAGE_LOG_TARGET, "Clean status for hash {:?}", &hash);
				preimage_helper::StatusFor::<T>::remove(hash);
			};
			weight
				.saturating_accrue(<T as frame_system::Config>::DbWeight::get().reads_writes(1, 1));
		}
		weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		let (image, status) = preimage_helper::image_count::<T>();
		assert!(image == status, "Preimage storage still correpted");

		Ok(())
	}
}

const BALANCES_LOG_TARGET: &str = "runtime::balances";
pub struct BalancesUpdateStorageVersionResetInactive<T>(PhantomData<T>);
impl<T> OnRuntimeUpgrade for BalancesUpdateStorageVersionResetInactive<T>
where
	T: frame_system::Config + pallet_balances::Config,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		ensure!(
			StorageVersion::get::<pallet_balances::Pallet<T>>() == 0,
			"Already upgrade to some non-zero version"
		);
		Ok(Vec::<u8>::new())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let on_chain_version = pallet_balances::Pallet::<T>::on_chain_storage_version();

		if on_chain_version == 0 {
			// Remove the old `StorageVersion` type.
			frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
				pallet_balances::Pallet::<T>::name().as_bytes(),
				"StorageVersion".as_bytes(),
			));

			// For security purpose, we may not want to do this now
			// InactiveIssuance::<T>::kill();

			// Set storage version to `1`.
			StorageVersion::new(1).put::<pallet_balances::Pallet<T>>();

			log::info!(target: BALANCES_LOG_TARGET, "Storage to version 1");
			T::DbWeight::get().reads_writes(1, 3)
		} else {
			log::info!(
				target: BALANCES_LOG_TARGET,
				"Migration did not execute. This probably should be removed"
			);
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		ensure!(StorageVersion::get::<pallet_balances::Pallet<T>>() == 1, "Must upgrade");
		Ok(())
	}
}

const BOUNTIES_LOG_TARGET: &str = "runtime::bounties";
pub struct BountiesUpdateStorageVersion<T>(PhantomData<T>);
impl<T> OnRuntimeUpgrade for BountiesUpdateStorageVersion<T>
where
	T: frame_system::Config + pallet_bounties::Config,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		ensure!(
			StorageVersion::get::<pallet_bounties::Pallet<T>>() == 0,
			"Already upgrade to some non-zero version"
		);
		Ok(Vec::<u8>::new())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let on_chain_version = pallet_bounties::Pallet::<T>::on_chain_storage_version();

		if on_chain_version == 0 {
			// Remove the old `StorageVersion` type.
			frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
				pallet_bounties::Pallet::<T>::name().as_bytes(),
				"StorageVersion".as_bytes(),
			));

			// Set storage version to `4`.
			StorageVersion::new(4).put::<pallet_bounties::Pallet<T>>();

			log::info!(target: BOUNTIES_LOG_TARGET, "Storage to version 4");
			T::DbWeight::get().reads_writes(1, 3)
		} else {
			log::info!(
				target: BOUNTIES_LOG_TARGET,
				"Migration did not execute. This probably should be removed"
			);
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		ensure!(StorageVersion::get::<pallet_bounties::Pallet<T>>() == 4, "Must upgrade");
		Ok(())
	}
}

const DEVELOPER_COMMITTEE_LOG_TARGET: &str = "runtime::collective3";
pub struct DeveloperCommitteeUpdateStorageVersion<T>(PhantomData<T>);
impl<T> OnRuntimeUpgrade for DeveloperCommitteeUpdateStorageVersion<T>
where
	T: frame_system::Config + pallet_collective::Config<pallet_collective::Instance3>,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		ensure!(
			StorageVersion::get::<pallet_collective::Pallet<T, pallet_collective::Instance3>>()
				== 0,
			"Already upgrade to some non-zero version"
		);
		Ok(Vec::<u8>::new())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let on_chain_version =
			pallet_collective::Pallet::<T, pallet_collective::Instance3>::on_chain_storage_version(
			);

		if on_chain_version == 0 {
			// Remove the old `StorageVersion` type.
			frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
				pallet_collective::Pallet::<T, pallet_collective::Instance3>::name().as_bytes(),
				"StorageVersion".as_bytes(),
			));

			// Set storage version to `4`.
			StorageVersion::new(4)
				.put::<pallet_collective::Pallet<T, pallet_collective::Instance3>>();

			log::info!(target: DEVELOPER_COMMITTEE_LOG_TARGET, "Storage to version 4");
			T::DbWeight::get().reads_writes(1, 3)
		} else {
			log::info!(
				target: DEVELOPER_COMMITTEE_LOG_TARGET,
				"Migration did not execute. This probably should be removed"
			);
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		ensure!(
			StorageVersion::get::<pallet_collective::Pallet<T, pallet_collective::Instance3>>()
				== 4,
			"Must upgrade"
		);
		Ok(())
	}
}

const DEVELOPER_COMMITTEE_MEMBERSHIP_LOG_TARGET: &str = "runtime::membership3";
pub struct DeveloperCommitteeMembershipUpdateStorageVersion<T>(PhantomData<T>);
impl<T> OnRuntimeUpgrade for DeveloperCommitteeMembershipUpdateStorageVersion<T>
where
	T: frame_system::Config + pallet_membership::Config<pallet_membership::Instance3>,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		ensure!(
			StorageVersion::get::<pallet_membership::Pallet<T, pallet_membership::Instance3>>()
				== 0,
			"Already upgrade to some non-zero version"
		);
		Ok(Vec::<u8>::new())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let on_chain_version =
			pallet_membership::Pallet::<T, pallet_membership::Instance3>::on_chain_storage_version(
			);

		if on_chain_version == 0 {
			// Remove the old `StorageVersion` type.
			frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
				pallet_membership::Pallet::<T, pallet_membership::Instance3>::name().as_bytes(),
				"StorageVersion".as_bytes(),
			));

			// Set storage version to `4`.
			StorageVersion::new(4)
				.put::<pallet_membership::Pallet<T, pallet_membership::Instance3>>();

			log::info!(target: DEVELOPER_COMMITTEE_MEMBERSHIP_LOG_TARGET, "Storage to version 4");
			T::DbWeight::get().reads_writes(1, 3)
		} else {
			log::info!(
				target: DEVELOPER_COMMITTEE_MEMBERSHIP_LOG_TARGET,
				"Migration did not execute. This probably should be removed"
			);
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		ensure!(
			StorageVersion::get::<pallet_membership::Pallet<T, pallet_membership::Instance3>>()
				== 4,
			"Must upgrade"
		);
		Ok(())
	}
}
