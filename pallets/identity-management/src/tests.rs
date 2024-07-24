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
#[allow(unused)]
use crate::{mock::*, Error, ShardIdentifier};
use core_primitives::{ErrorDetail, IMPError};
use frame_support::{assert_noop, assert_ok};
use sp_core::H256;

use pallet_teebag::test_util::{get_signer, TEST8_MRENCLAVE};
type SystemAccountId = <Test as frame_system::Config>::AccountId;
const ALICE_PUBKEY: &[u8; 32] = &[1u8; 32];
const BOB_PUBKEY: &[u8; 32] = &[2u8; 32];
const EDDIE_PUBKEY: &[u8; 32] = &[5u8; 32];

#[test]
fn link_identity_without_delegatee_works() {
	new_test_ext().execute_with(|| {
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		assert_ok!(IdentityManagement::link_identity(
			RuntimeOrigin::signed(alice.clone()),
			shard,
			alice.clone(),
			vec![1u8; 2048],
			vec![1u8; 2048],
			vec![1u8; 2048],
		));
		System::assert_last_event(RuntimeEvent::IdentityManagement(
			crate::Event::LinkIdentityRequested {
				shard,
				account: alice,
				encrypted_identity: vec![1u8; 2048],
				encrypted_validation_data: vec![1u8; 2048],
				encrypted_web3networks: vec![1u8; 2048],
			},
		));
	});
}

#[test]
fn link_identity_with_authorized_delegatee_works() {
	new_test_ext().execute_with(|| {
		let eddie: SystemAccountId = get_signer(EDDIE_PUBKEY);
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		assert_ok!(IdentityManagement::link_identity(
			RuntimeOrigin::signed(eddie), // authorized delegatee set in initialisation
			shard,
			alice.clone(),
			vec![1u8; 2048],
			vec![1u8; 2048],
			vec![1u8; 2048],
		));
		System::assert_last_event(RuntimeEvent::IdentityManagement(
			crate::Event::LinkIdentityRequested {
				shard,
				account: alice,
				encrypted_identity: vec![1u8; 2048],
				encrypted_validation_data: vec![1u8; 2048],
				encrypted_web3networks: vec![1u8; 2048],
			},
		));
	});
}

#[test]
fn link_identity_with_unauthorized_delegatee_fails() {
	new_test_ext().execute_with(|| {
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		let bob: SystemAccountId = get_signer(BOB_PUBKEY);
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		assert_noop!(
			IdentityManagement::link_identity(
				RuntimeOrigin::signed(bob),
				shard,
				alice,
				vec![1u8; 2048],
				vec![1u8; 2048],
				vec![1u8; 2048],
			),
			Error::<Test>::UnauthorizedUser
		);
	});
}

#[test]
fn deactivate_identity_works() {
	new_test_ext().execute_with(|| {
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		assert_ok!(IdentityManagement::deactivate_identity(
			RuntimeOrigin::signed(alice.clone()),
			shard,
			vec![1u8; 2048]
		));
		System::assert_last_event(RuntimeEvent::IdentityManagement(
			crate::Event::DeactivateIdentityRequested {
				shard,
				account: alice,
				encrypted_identity: vec![1u8; 2048],
			},
		));
	});
}

#[test]
fn activate_identity_works() {
	new_test_ext().execute_with(|| {
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		assert_ok!(IdentityManagement::activate_identity(
			RuntimeOrigin::signed(alice.clone()),
			shard,
			vec![1u8; 2048]
		));
		System::assert_last_event(RuntimeEvent::IdentityManagement(
			crate::Event::ActivateIdentityRequested {
				shard,
				account: alice,
				encrypted_identity: vec![1u8; 2048],
			},
		));
	});
}

#[test]
fn tee_callback_with_unregistered_enclave_fails() {
	new_test_ext().execute_with(|| {
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		assert_noop!(
			IdentityManagement::some_error(
				RuntimeOrigin::signed(alice),
				None,
				IMPError::LinkIdentityFailed(ErrorDetail::WrongWeb2Handle),
				H256::default(),
			),
			sp_runtime::DispatchError::BadOrigin,
		);
	});
}

#[test]
fn extrinsic_whitelist_origin_works() {
	new_test_ext().execute_with(|| {
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		// activate the whitelist which is empty at the beginning
		assert_ok!(IMPExtrinsicWhitelist::switch_group_control_on(RuntimeOrigin::root()));
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		assert_noop!(
			IdentityManagement::link_identity(
				RuntimeOrigin::signed(alice.clone()),
				shard,
				alice.clone(),
				vec![1u8; 2048],
				vec![1u8; 2048],
				vec![1u8; 2048],
			),
			sp_runtime::DispatchError::BadOrigin
		);

		// add `alice` to whitelist group
		assert_ok!(IMPExtrinsicWhitelist::add_group_member(RuntimeOrigin::root(), alice.clone()));
		assert_ok!(IdentityManagement::link_identity(
			RuntimeOrigin::signed(alice.clone()),
			shard,
			alice.clone(),
			vec![1u8; 2048],
			vec![1u8; 2048],
			vec![1u8; 2048],
		));
		System::assert_last_event(RuntimeEvent::IdentityManagement(
			crate::Event::LinkIdentityRequested {
				shard,
				account: alice,
				encrypted_identity: vec![1u8; 2048],
				encrypted_validation_data: vec![1u8; 2048],
				encrypted_web3networks: vec![1u8; 2048],
			},
		));
	});
}
