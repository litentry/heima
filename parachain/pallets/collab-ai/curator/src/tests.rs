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
use crate::{CandidateStatus, CuratorIndexToInfo, Error, PublicCuratorCount, PublicCuratorToIndex};
use frame_support::{assert_noop, assert_ok};
use pallet_balances::Error as BalanceError;
use sp_core::crypto::AccountId32;

#[test]
fn test_register_curator_ok() {
	new_test_ext().execute_with(|| {
		let curator: AccountId32 = AccountId32::from([1u8; 32]);
		let info_hash: [u8; 32] = [1; 32];
		let curator_index = PublicCuratorCount::<Test>::get();

		// Register curator
		assert_ok!(Curator::regist_curator(
			RuntimeOrigin::signed(curator.clone()),
			sp_core::H256(info_hash)
		));

		// Check if curator is stored correctly
		assert_eq!(PublicCuratorToIndex::<Test>::get(&curator), Some(curator_index));
		assert_eq!(PublicCuratorCount::<Test>::get(), curator_index + 1);
		assert_eq!(
			CuratorIndexToInfo::<Test>::get(curator_index),
			Some((sp_core::H256(info_hash), 1, curator.clone(), CandidateStatus::Unverified))
		);

		System::assert_last_event(RuntimeEvent::Curator(crate::Event::CuratorRegisted {
			curator,
			curator_index,
			info_hash: sp_core::H256(info_hash),
		}));
	})
}

#[test]
fn test_register_curator_curator_already_registered() {
	new_test_ext().execute_with(|| {
		let curator: AccountId32 = AccountId32::from([1u8; 32]);
		let info_hash: [u8; 32] = [1; 32];

		// Register curator
		assert_ok!(Curator::regist_curator(
			RuntimeOrigin::signed(curator.clone()),
			sp_core::H256(info_hash)
		));

		// Register curator twice
		assert_noop!(
			Curator::regist_curator(
				RuntimeOrigin::signed(curator.clone()),
				sp_core::H256(info_hash)
			),
			Error::<Test>::CuratorAlreadyRegistered
		);
	});
}

#[test]
fn test_register_curator_without_minimum_curator_deposit() {
	new_test_ext().execute_with(|| {
		let curator: AccountId32 = AccountId32::from([5u8; 32]);
		let info_hash: [u8; 32] = [1; 32];

		// Register curator with insufficient balance
		assert_noop!(
			Curator::regist_curator(
				RuntimeOrigin::signed(curator.clone()),
				sp_core::H256(info_hash)
			),
			BalanceError::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn test_update_curator_ok() {
	new_test_ext().execute_with(|| {
		let curator: AccountId32 = AccountId32::from([1u8; 32]);
		let curator_index = PublicCuratorCount::<Test>::get();
		let info_hash: [u8; 32] = [1; 32];
		let updated_info_hash: [u8; 32] = [2; 32];

		// Register curator with info_hash
		assert_ok!(Curator::regist_curator(
			RuntimeOrigin::signed(curator.clone()),
			sp_core::H256(info_hash)
		));

		// Check the storage
		assert_eq!(
			CuratorIndexToInfo::<Test>::get(curator_index),
			Some((sp_core::H256(info_hash), 1, curator.clone(), CandidateStatus::Unverified))
		);

		// Update the info hash
		assert_ok!(Curator::update_curator(
			RuntimeOrigin::signed(curator.clone()),
			sp_core::H256(updated_info_hash)
		));

		// Check the storage after update
		assert_eq!(
			CuratorIndexToInfo::<Test>::get(curator_index),
			Some((
				sp_core::H256(updated_info_hash),
				1,
				curator.clone(),
				CandidateStatus::Unverified
			))
		);

		System::assert_last_event(RuntimeEvent::Curator(crate::Event::CuratorUpdated {
			curator,
			curator_index,
			info_hash: sp_core::H256(updated_info_hash),
		}));
	});
}

#[test]
fn test_update_curator_curator_not_registered() {
	new_test_ext().execute_with(|| {
		let curator: AccountId32 = AccountId32::from([1u8; 32]);
		let info_hash: [u8; 32] = [1; 32];

		// Update the info hash
		assert_noop!(
			Curator::update_curator(
				RuntimeOrigin::signed(curator.clone()),
				sp_core::H256(info_hash)
			),
			Error::<Test>::CuratorNotRegistered
		);
	});
}

#[test]
fn test_update_curator_curator_banned_insufficient_balance() {
	new_test_ext().execute_with(|| {
		let curator: AccountId32 = AccountId32::from([2u8; 32]);
		let info_hash: [u8; 32] = [1; 32];
		let updated_info_hash: [u8; 32] = [2; 32];

		// Register curator with info_hash
		assert_ok!(Curator::regist_curator(
			RuntimeOrigin::signed(curator.clone()),
			sp_core::H256(info_hash)
		));

		assert_ok!(Curator::judge_curator_status(
			RuntimeOrigin::root(),
			curator.clone(),
			CandidateStatus::Banned
		));

		// Update the info hash
		assert_noop!(
			Curator::update_curator(
				RuntimeOrigin::signed(curator.clone()),
				sp_core::H256(updated_info_hash)
			),
			BalanceError::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn test_update_curator_curator_banned_sufficient_balance() {
	new_test_ext().execute_with(|| {
		let curator: AccountId32 = AccountId32::from([1u8; 32]);
		let curator_index = PublicCuratorCount::<Test>::get();
		let info_hash: [u8; 32] = [1; 32];
		let updated_info_hash: [u8; 32] = [2; 32];

		// Register curator with info_hash
		assert_ok!(Curator::regist_curator(
			RuntimeOrigin::signed(curator.clone()),
			sp_core::H256(info_hash)
		));

		// Check the storage
		assert_eq!(
			CuratorIndexToInfo::<Test>::get(curator_index),
			Some((sp_core::H256(info_hash), 1, curator.clone(), CandidateStatus::Unverified))
		);

		assert_ok!(Curator::judge_curator_status(
			RuntimeOrigin::root(),
			curator.clone(),
			CandidateStatus::Banned
		));

		// Update the info hash
		assert_ok!(Curator::update_curator(
			RuntimeOrigin::signed(curator.clone()),
			sp_core::H256(updated_info_hash)
		));

		// Check the storage after update
		assert_eq!(
			CuratorIndexToInfo::<Test>::get(curator_index),
			Some((sp_core::H256(updated_info_hash), 1, curator.clone(), CandidateStatus::Banned))
		);

		System::assert_last_event(RuntimeEvent::Curator(crate::Event::CuratorUpdated {
			curator,
			curator_index,
			info_hash: sp_core::H256(updated_info_hash),
		}));
	});
}

#[test]
fn test_judge_curator_status_ok() {
	new_test_ext().execute_with(|| {
		let curator: AccountId32 = AccountId32::from([1u8; 32]);
		let curator_index = PublicCuratorCount::<Test>::get();
		let info_hash: [u8; 32] = [1; 32];

		// Register curator with info_hash
		assert_ok!(Curator::regist_curator(
			RuntimeOrigin::signed(curator.clone()),
			sp_core::H256(info_hash)
		));

		// Check the storage
		assert_eq!(
			CuratorIndexToInfo::<Test>::get(curator_index),
			Some((sp_core::H256(info_hash), 1, curator.clone(), CandidateStatus::Unverified))
		);

		assert_ok!(Curator::judge_curator_status(
			RuntimeOrigin::root(),
			curator.clone(),
			CandidateStatus::Verified
		));

		// Check the storage after status update
		assert_eq!(
			CuratorIndexToInfo::<Test>::get(curator_index),
			Some((sp_core::H256(info_hash), 1, curator.clone(), CandidateStatus::Verified))
		);

		System::assert_last_event(RuntimeEvent::Curator(crate::Event::CuratorStatusUpdated {
			curator,
			curator_index,
			status: CandidateStatus::Verified,
		}));
	});
}

#[test]
fn test_judge_curator_curator_not_registered() {
	new_test_ext().execute_with(|| {
		let curator: AccountId32 = AccountId32::from([1u8; 32]);

		// Try judging the curator status
		assert_noop!(
			Curator::judge_curator_status(
				RuntimeOrigin::root(),
				curator.clone(),
				CandidateStatus::Verified
			),
			Error::<Test>::CuratorNotRegistered
		);
	})
}

#[test]
fn test_clean_curator_ok() {
	new_test_ext().execute_with(|| {
		let curator: AccountId32 = AccountId32::from([1u8; 32]);
		let curator_index = PublicCuratorCount::<Test>::get();
		let info_hash: [u8; 32] = [1; 32];

		// Register curator with info_hash
		assert_ok!(Curator::regist_curator(
			RuntimeOrigin::signed(curator.clone()),
			sp_core::H256(info_hash)
		));

		// Check the storage
		assert_eq!(
			CuratorIndexToInfo::<Test>::get(curator_index),
			Some((sp_core::H256(info_hash), 1, curator.clone(), CandidateStatus::Unverified))
		);

		assert_ok!(Curator::clean_curator(RuntimeOrigin::signed(curator.clone()),));

		// Check the storage after status update
		assert_eq!(CuratorIndexToInfo::<Test>::get(curator_index), None);

		System::assert_last_event(RuntimeEvent::Curator(crate::Event::CuratorCleaned {
			curator,
			curator_index,
		}));
	});
}

#[test]
fn test_clean_curator_curator_not_registered() {
	new_test_ext().execute_with(|| {
		let curator: AccountId32 = AccountId32::from([1u8; 32]);

		// Try cleaning the curator info
		assert_noop!(
			Curator::clean_curator(RuntimeOrigin::signed(curator),),
			Error::<Test>::CuratorNotRegistered
		);
	})
}
