// Copyright 2020-2021 Trust Computing GmbH.
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
	inherent::Vec,
	pallet_prelude::*,
	storage::storage_prefix,
	traits::{Currency, Get, OnRuntimeUpgrade},
	StorageHasher, Twox128,
};
use pallet_parachain_staking::BalanceOf;
use sp_runtime::{traits::Zero, Perbill};
use sp_std::marker::PhantomData;

pub struct MigrateCollatorSelectionIntoParachainStaking<T>(PhantomData<T>);
impl<T> OnRuntimeUpgrade for MigrateCollatorSelectionIntoParachainStaking<T>
where
	T: pallet_parachain_staking::Config + pallet_bridge_transfer::Config,
	<T as frame_system::Config>::Event: From<pallet_parachain_staking::Event<T>>,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;
		use core_primitives::AccountId;

		log::info!("Pre check pallet CollatorSelection exists");
		// Get Invulnerables address from CollatorSelection
		// WARN: We do not care about any Candidates storage, as we forbid any general transaction
		// by sudo and so no info there in practice
		let invulnerables = frame_support::storage::migration::get_storage_value::<Vec<AccountId>>(
			b"CollatorSelection",
			b"Invulnerables",
			b"",
		)
		.expect("Storage query fails: CollatorSelection Invulnerables");

		if invulnerables.is_empty() {
			return Err("CollatorSelection empty")
		};
		let invulnerables_len: u32 = invulnerables.len().try_into().unwrap_or(0);
		assert!(
			invulnerables_len >=
				<T as pallet_parachain_staking::Config>::MinSelectedCandidates::get(),
			"Need More Initial Candidates"
		);

		// Set the temporary storage for post upgrade check
		Self::set_temp_storage(invulnerables, "invulnerables");

		// Ensure ParachainStaking is Empty
		assert!(
			!frame_support::storage::migration::have_storage_value(
				b"ParachainStaking",
				b"SelectedCandidates",
				b"",
			),
			"ParachainStaking SelectedCandidates Storage Already Exist"
		);

		assert!(
			!frame_support::storage::migration::have_storage_value(
				b"ParachainStaking",
				b"CandidatePool",
				b"",
			),
			"ParachainStaking CandidatePool Storage Already Exist"
		);

		assert!(
			!frame_support::storage::migration::have_storage_value(
				b"ParachainStaking",
				b"Total",
				b"",
			),
			"ParachainStaking Total Storage Already Exist"
		);
		Ok(())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		use sp_io::KillStorageResult;
		let mut invulnerables = frame_support::storage::migration::get_storage_value::<
			Vec<<T as frame_system::Config>::AccountId>,
		>(b"CollatorSelection", b"Invulnerables", b"")
		.expect("Storage query fails: CollatorSelection Invulnerables");
		invulnerables.sort();

		let invulnerables_len: u32 = invulnerables.len().try_into().unwrap_or(0);
		assert!(
			invulnerables_len >=
				<T as pallet_parachain_staking::Config>::MinSelectedCandidates::get(),
			"Need More Initial Candidates"
		);

		// Add whitelist Storage
		frame_support::storage::migration::put_storage_value::<
			Vec<<T as frame_system::Config>::AccountId>,
		>(b"ParachainStaking", b"Candidates", b"", invulnerables.clone());

		let mut candidate_count = 0u32;
		// Get the minimum collator stake amount
		let min_collator_stk = <T as pallet_parachain_staking::Config>::MinCollatorStk::get();
		// Initialize the candidates
		for candidate in invulnerables {
			assert!(
				<T as pallet_parachain_staking::Config>::Currency::free_balance(&candidate) >=
					min_collator_stk,
				"Account does not have enough balance to bond as a candidate."
			);

			if let Err(error) = <pallet_parachain_staking::Pallet<T>>::join_candidates(
				<T as frame_system::Config>::Origin::from(Some(candidate.clone()).into()),
				min_collator_stk,
			) {
				log::warn!("Join candidates failed in genesis with error {:?}", error);
			} else {
				candidate_count = candidate_count.saturating_add(1u32);
			}
		}

		assert!(candidate_count > 0, "No valid candidates");

		// Reproduce the genesis build
		// Initialize the rest setup of parachain-staking
		// After runtimeUpgrade, we should:
		// (1) Set the inflation by                              set_inflation
		// (2) Set collator commission by                        set_collator_commission
		// (3) Set parachain bond config by                      parachset_parachain_bond_account
		// (4) Set total selected candidates to minimum config   set_total_selected
		// (5) Choose top TotalSelected collator candidates
		// (6) Refresh round if it is during runtimeUpgrade
		// (7) Snapshot total staked (through select_top_candidates)
		// (8) NewRound Event Deposit

		// Inflation config as Default, so do nothing
		// ...
		// Set collator commission to default config
		frame_support::storage::migration::put_storage_value::<Perbill>(
			b"ParachainStaking",
			b"CollatorCommission",
			b"",
			<T as pallet_parachain_staking::Config>::DefaultCollatorCommission::get(),
		);
		// Set parachain bond config to default config
		frame_support::storage::migration::put_storage_value::<
			pallet_parachain_staking::ParachainBondConfig<<T as frame_system::Config>::AccountId>,
		>(
			b"ParachainStaking",
			b"ParachainBondInfo",
			b"",
			pallet_parachain_staking::ParachainBondConfig {
				// must be set soon; if not => due inflation will be sent to some weird place
				account: <T as frame_system::Config>::AccountId::decode(
					&mut sp_runtime::traits::TrailingZeroInput::zeroes(),
				)
				.expect("infinite length input; no invalid inputs for type; qed"),
				percent:
					<T as pallet_parachain_staking::Config>::DefaultParachainBondReservePercent::get(
					),
			},
		);

		// // Set total selected candidates to minimum config
		frame_support::storage::migration::put_storage_value::<u32>(
			b"ParachainStaking",
			b"TotalSelected",
			b"",
			candidate_count,
		);
		// Choose top TotalSelected collator candidates
		// WARNING/TODO: We change the private into public of select_top_candidates function in
		// pallet. We should change it back in next runtime upgrade for safety.
		let (v_count, _, total_staked) =
			<pallet_parachain_staking::Pallet<T>>::select_top_candidates(1u32);

		// Start Round 1 at Block 0
		let round: pallet_parachain_staking::RoundInfo<<T as frame_system::Config>::BlockNumber> =
			pallet_parachain_staking::RoundInfo::new(
				1u32,
				0u32.into(),
				<T as pallet_parachain_staking::Config>::DefaultBlocksPerRound::get(),
			);
		frame_support::storage::migration::put_storage_value::<
			pallet_parachain_staking::RoundInfo<<T as frame_system::Config>::BlockNumber>,
		>(b"ParachainStaking", b"Round", b"", round);

		// // Snapshot total stake
		// The code below is supposed to behave same as:
		// <Staked<T>>::insert(1u32, <Total<T>>::get())
		let val = frame_support::storage::migration::get_storage_value::<BalanceOf<T>>(
			b"ParachainStaking",
			b"Total",
			b"",
		);
		let storage_prefix = storage_prefix(b"ParachainStaking", b"Staked");
		let key_hashed = 1u32.using_encoded(Twox64Concat::hash);
		let mut final_key = Vec::with_capacity(storage_prefix.len() + key_hashed.len());
		final_key.extend_from_slice(&storage_prefix);
		final_key.extend_from_slice(key_hashed.as_ref());
		frame_support::storage::unhashed::put(final_key.as_ref(), &val);

		// Deposit NewRound event at RuntimeUpgrade
		<frame_system::Pallet<T>>::deposit_event(pallet_parachain_staking::Event::NewRound {
			starting_block: <T as frame_system::Config>::BlockNumber::zero(),
			round: 1u32,
			selected_collators_number: v_count,
			total_balance: total_staked,
		});

		// Remove CollatorSelection Storage
		// TODO: Very Weak safety
		let entries: u64 = 4 + 6142;
		let _res: KillStorageResult = frame_support::storage::unhashed::clear_prefix(
			&Twox128::hash(b"CollatorSelection"),
			Some(entries.try_into().unwrap()),
			None,
		)
		.into();
		<T as frame_system::Config>::DbWeight::get().writes(entries)
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;
		use core_primitives::AccountId;
		use sp_io::KillStorageResult;

		log::info!("Post check CollatorSelection");
		let res: KillStorageResult = frame_support::storage::unhashed::clear_prefix(
			&Twox128::hash(b"CollatorSelection"),
			Some(0),
			None,
		)
		.into();

		match res {
			KillStorageResult::AllRemoved(0) | KillStorageResult::SomeRemaining(0) => {},
			KillStorageResult::AllRemoved(n) | KillStorageResult::SomeRemaining(n) => {
				log::error!("Remaining entries: {:?}", n);
				return Err("CollatorSelection not removed")
			},
		};

		assert!(
			frame_support::storage::migration::have_storage_value(
				b"ParachainStaking",
				b"CandidatePool",
				b"",
			),
			"ParachainStaking CandidatePool Storage not migrate properly"
		);

		assert!(
			frame_support::storage::migration::have_storage_value(
				b"ParachainStaking",
				b"Total",
				b"",
			),
			"ParachainStaking Total Storage not migrate properly"
		);

		// Check the Selected Candidates info
		let mut selected_candidates = frame_support::storage::migration::get_storage_value::<
			Vec<AccountId>,
		>(b"ParachainStaking", b"SelectedCandidates", b"")
		.expect("Storage query fails: ParachainStaking SelectedCandidates");
		selected_candidates.sort();
		let mut invulnerables: Vec<AccountId> =
			Self::get_temp_storage("invulnerables").expect("qed");
		invulnerables.sort();

		assert!(selected_candidates == invulnerables, "candidates not migrate properly");

		// Check the Round info
		let round_info = frame_support::storage::migration::get_storage_value::<
			pallet_parachain_staking::RoundInfo<T::BlockNumber>,
		>(b"ParachainStaking", b"Round", b"")
		.expect("Storage query fails: ParachainStaking Round");

		let expected_round_info = pallet_parachain_staking::RoundInfo::<T::BlockNumber>::new(
			1u32,
			0u32.into(),
			<T as pallet_parachain_staking::Config>::DefaultBlocksPerRound::get(),
		);
		assert!(round_info == expected_round_info, "round info not migrate properly");

		Ok(())
	}
}
