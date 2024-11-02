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

use crate::mock::*;
use frame_support::assert_ok;
use pallet_collab_ai_common::GuardianQuery; // Import GuardianQuery trait
use pallet_collab_ai_common::GuardianVote;
use sp_runtime::AccountId32;

#[test]
fn test_regist_guardian() {
	new_test_ext().execute_with(|| {
		let guardian: AccountId32 = AccountId32::from([1u8; 32]);
		let info_hash: [u8; 32] = [1; 32];

		// Register guardian
		assert_ok!(Guardian::regist_guardian(
			RuntimeOrigin::signed(guardian.clone()),
			sp_core::H256(info_hash)
		));

		// Check if guardian is stored correctly
		assert_eq!(Guardian::public_guardian_to_index(&guardian), Some(1));
		System::assert_last_event(RuntimeEvent::Guardian(crate::Event::GuardianRegisted {
			guardian,
			guardian_index: 1,
			info_hash: sp_core::H256(info_hash),
		}));
	});
}

#[test]
fn test_update_guardian() {
	new_test_ext().execute_with(|| {
		let guardian: AccountId32 = AccountId32::from([1u8; 32]);
		let info_hash: [u8; 32] = [1; 32];
		let updated_hash: [u8; 32] = [2; 32];

		// Register guardian
		assert_ok!(Guardian::regist_guardian(
			RuntimeOrigin::signed(guardian.clone()),
			sp_core::H256(info_hash)
		));

		// Update guardian
		assert_ok!(Guardian::update_guardian(
			RuntimeOrigin::signed(guardian.clone()),
			sp_core::H256(updated_hash)
		));

		// Check if guardian info is updated correctly
		let guardian_info = Guardian::guardian_index_to_info(1).unwrap();
		assert_eq!(guardian_info.0, sp_core::H256(updated_hash));
		System::assert_last_event(RuntimeEvent::Guardian(crate::Event::GuardianUpdated {
			guardian,
			guardian_index: 1,
			info_hash: sp_core::H256(updated_hash),
		}));
	});
}

#[test]
fn test_clean_guardian() {
	new_test_ext().execute_with(|| {
		let guardian: AccountId32 = AccountId32::from([1u8; 32]);
		let info_hash: [u8; 32] = [1; 32];

		// Register and clean guardian
		assert_ok!(Guardian::regist_guardian(
			RuntimeOrigin::signed(guardian.clone()),
			sp_core::H256(info_hash)
		));
		assert_ok!(Guardian::clean_guardian(RuntimeOrigin::signed(guardian.clone())));

		// Check if guardian is removed
		assert_eq!(Guardian::public_guardian_to_index(&guardian), None);
		System::assert_last_event(RuntimeEvent::Guardian(crate::Event::GuardianCleaned {
			guardian,
			guardian_index: 1,
		}));
	});
}

#[test]
fn test_vote_for_guardian() {
	new_test_ext().execute_with(|| {
		let guardian: AccountId32 = AccountId32::from([1u8; 32]);
		let voter: AccountId32 = AccountId32::from([2u8; 32]);
		let info_hash: [u8; 32] = [1; 32];

		// Register guardian
		assert_ok!(Guardian::regist_guardian(
			RuntimeOrigin::signed(guardian.clone()),
			sp_core::H256(info_hash)
		));

		// Cast a vote
		assert_ok!(Guardian::vote(
			RuntimeOrigin::signed(voter.clone()),
			guardian.clone(),
			Some(GuardianVote::Specific(1))
		));

		// Check if vote is recorded
		assert_eq!(
			Guardian::get_vote(voter.clone(), guardian.clone()),
			Some(GuardianVote::Specific(1))
		);
		System::assert_last_event(RuntimeEvent::Guardian(crate::Event::VoteGuardian {
			voter,
			guardian_index: 1,
			guardian,
			status: Some(GuardianVote::Specific(1)),
		}));
	});
}

#[test]
fn test_remove_all_votes() {
	new_test_ext().execute_with(|| {
		let guardian: AccountId32 = AccountId32::from([1u8; 32]);
		let voter: AccountId32 = AccountId32::from([2u8; 32]);
		let info_hash: [u8; 32] = [1; 32];

		// Register guardian and vote
		assert_ok!(Guardian::regist_guardian(
			RuntimeOrigin::signed(guardian.clone()),
			sp_core::H256(info_hash)
		));
		assert_ok!(Guardian::vote(
			RuntimeOrigin::signed(voter.clone()),
			guardian.clone(),
			Some(GuardianVote::Specific(1))
		));

		// Remove all votes
		assert_ok!(Guardian::remove_all_votes(RuntimeOrigin::signed(voter.clone())));

		// Check if votes are removed
		assert_eq!(Guardian::get_vote(voter.clone(), guardian.clone()), None);
		System::assert_last_event(RuntimeEvent::Guardian(crate::Event::RemoveAllVote { voter }));
	});
}
