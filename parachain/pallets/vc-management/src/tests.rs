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

use crate::{mock::*, Error, Status};
use core_primitives::{Assertion, Identity, ShardIdentifier};
use frame_support::{assert_noop, assert_ok};
use sp_core::H256;
use sp_std::{vec, vec::Vec};

use pallet_teebag::test_util::{get_signer, TEST8_MRENCLAVE, TEST8_SIGNER_PUB};

type SystemAccountId = <Test as frame_system::Config>::AccountId;
const ALICE_PUBKEY: &[u8; 32] = &[1u8; 32];
const BOB_PUBKEY: &[u8; 32] = &[2u8; 32];
const EDDIE_PUBKEY: &[u8; 32] = &[5u8; 32];

#[test]
fn request_vc_without_delegatee_works() {
	new_test_ext().execute_with(|| {
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		assert_ok!(VCManagement::request_vc(
			RuntimeOrigin::signed(alice.clone()),
			shard,
			Assertion::A1
		));
		System::assert_last_event(RuntimeEvent::VCManagement(crate::Event::VCRequested {
			account: alice,
			shard,
			assertion: Assertion::A1,
		}));
	});
}

#[test]
fn request_vc_13_with_authorized_delegatee_works() {
	new_test_ext().execute_with(|| {
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		let eddie: SystemAccountId = get_signer(EDDIE_PUBKEY);
		assert_ok!(VCManagement::request_vc(
			RuntimeOrigin::signed(eddie.clone()),
			shard,
			Assertion::A13(alice.clone())
		));
		System::assert_last_event(RuntimeEvent::VCManagement(crate::Event::VCRequested {
			account: eddie,
			shard,
			assertion: Assertion::A13(alice),
		}));
	});
}

#[test]
fn request_vc_13_with_unauthorized_delegatee_fails() {
	new_test_ext().execute_with(|| {
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		// even alice can't apply A13 by herself
		assert_noop!(
			VCManagement::request_vc(
				RuntimeOrigin::signed(alice.clone()),
				shard,
				Assertion::A13(alice)
			),
			Error::<Test>::UnauthorizedUser
		);
	});
}

#[test]
fn vc_issued_works() {
	new_test_ext().execute_with(|| {
		let signer: SystemAccountId = get_signer(TEST8_SIGNER_PUB);
		let alice: Identity = get_signer(ALICE_PUBKEY);
		assert_ok!(VCManagement::vc_issued(
			RuntimeOrigin::signed(signer),
			alice,
			Assertion::A1,
			H256::default(),
			H256::default(),
		));
	});
}

#[test]
fn vc_issued_with_unpriviledged_origin_fails() {
	new_test_ext().execute_with(|| {
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		let bob: SystemAccountId = get_signer(BOB_PUBKEY);
		assert_noop!(
			VCManagement::vc_issued(
				RuntimeOrigin::signed(bob),
				alice.into(),
				Assertion::A1,
				H256::default(),
				H256::default(),
			),
			sp_runtime::DispatchError::BadOrigin
		);
	});
}

#[test]
fn set_admin_works() {
	new_test_ext().execute_with(|| {
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		let bob: SystemAccountId = get_signer(BOB_PUBKEY);
		assert_eq!(VCManagement::admin().unwrap(), alice);
		assert_ok!(VCManagement::set_admin(RuntimeOrigin::root(), bob.clone()));
		assert_eq!(VCManagement::admin().unwrap(), bob);
		System::assert_last_event(RuntimeEvent::VCManagement(crate::Event::AdminChanged {
			old_admin: Some(alice),
			new_admin: Some(bob),
		}));
	});
}

#[test]
fn set_admin_fails_with_unprivileged_origin() {
	new_test_ext().execute_with(|| {
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		let bob: SystemAccountId = get_signer(BOB_PUBKEY);
		assert_eq!(VCManagement::admin().unwrap(), alice);
		assert_noop!(
			VCManagement::set_admin(RuntimeOrigin::signed(bob.clone()), bob),
			sp_runtime::DispatchError::BadOrigin
		);
		assert_eq!(VCManagement::admin().unwrap(), alice);
	});
}

#[test]
fn add_schema_works() {
	new_test_ext().execute_with(|| {
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		assert_eq!(VCManagement::schema_index(), 0);
		let id: Vec<u8> = vec![1, 2, 3, 4];
		let content: Vec<u8> = vec![5, 6, 7, 8];
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		assert_ok!(VCManagement::add_schema(
			RuntimeOrigin::signed(alice.clone()),
			shard,
			id,
			content
		));
		System::assert_last_event(RuntimeEvent::VCManagement(crate::Event::SchemaIssued {
			account: alice,
			shard,
			index: 0,
		}));
		assert_eq!(VCManagement::schema_index(), 1);
	});
}

#[test]
fn add_schema_with_unpriviledged_origin_fails() {
	new_test_ext().execute_with(|| {
		let bob: SystemAccountId = get_signer(BOB_PUBKEY);
		let id: Vec<u8> = vec![1, 2, 3, 4];
		let content: Vec<u8> = vec![5, 6, 7, 8];
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		assert_noop!(
			VCManagement::add_schema(RuntimeOrigin::signed(bob), shard, id, content),
			Error::<Test>::RequireAdmin
		);
	});
}

#[test]
fn add_two_schemas_works() {
	new_test_ext().execute_with(|| {
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		assert_eq!(VCManagement::schema_index(), 0);
		let id: Vec<u8> = vec![1, 2, 3, 4];
		let content: Vec<u8> = vec![5, 6, 7, 8];
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		assert_ok!(VCManagement::add_schema(
			RuntimeOrigin::signed(alice.clone()),
			shard,
			id.clone(),
			content.clone()
		));
		assert_ok!(VCManagement::add_schema(
			RuntimeOrigin::signed(alice.clone()),
			shard,
			id,
			content
		));
		System::assert_last_event(RuntimeEvent::VCManagement(crate::Event::SchemaIssued {
			account: alice,
			shard,
			index: 1,
		}));
		assert_eq!(VCManagement::schema_index(), 2);
	});
}

#[test]
fn disable_schema_works() {
	new_test_ext().execute_with(|| {
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		let id: Vec<u8> = vec![1, 2, 3, 4];
		let content: Vec<u8> = vec![5, 6, 7, 8];
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		assert_ok!(VCManagement::add_schema(
			RuntimeOrigin::signed(alice.clone()),
			shard,
			id,
			content
		));
		assert!(VCManagement::schema_registry(0).is_some());
		assert_ok!(VCManagement::disable_schema(RuntimeOrigin::signed(alice), shard, 0));
		assert!(VCManagement::schema_registry(0).is_some());
		let context = VCManagement::schema_registry(0).unwrap();
		assert_eq!(context.status, Status::Disabled);
	});
}

#[test]
fn disable_schema_with_non_existent_fails() {
	new_test_ext().execute_with(|| {
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		assert_noop!(
			VCManagement::disable_schema(RuntimeOrigin::signed(alice), shard, 2),
			Error::<Test>::SchemaNotExists
		);
	});
}

#[test]
fn disable_schema_with_unpriviledged_origin_fails() {
	new_test_ext().execute_with(|| {
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		let bob: SystemAccountId = get_signer(BOB_PUBKEY);
		let id: Vec<u8> = vec![1, 2, 3, 4];
		let content: Vec<u8> = vec![5, 6, 7, 8];
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		assert_ok!(VCManagement::add_schema(RuntimeOrigin::signed(alice), shard, id, content));
		assert_noop!(
			VCManagement::disable_schema(RuntimeOrigin::signed(bob), shard, 0),
			Error::<Test>::RequireAdmin
		);
	});
}

#[test]
fn activate_schema_works() {
	new_test_ext().execute_with(|| {
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		let id: Vec<u8> = vec![1, 2, 3, 4];
		let content: Vec<u8> = vec![5, 6, 7, 8];
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		assert_ok!(VCManagement::add_schema(
			RuntimeOrigin::signed(alice.clone()),
			shard,
			id,
			content
		));
		assert!(VCManagement::schema_registry(0).is_some());
		assert_ok!(VCManagement::disable_schema(RuntimeOrigin::signed(alice.clone()), shard, 0));
		// schema is disabled
		assert_eq!(VCManagement::schema_registry(0).unwrap().status, Status::Disabled);
		// schema is activated
		assert_ok!(VCManagement::activate_schema(RuntimeOrigin::signed(alice), shard, 0));
		assert_eq!(VCManagement::schema_registry(0).unwrap().status, Status::Active);
	});
}

#[test]
fn activate_already_activated_schema_fails() {
	new_test_ext().execute_with(|| {
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		let id: Vec<u8> = vec![1, 2, 3, 4];
		let content: Vec<u8> = vec![5, 6, 7, 8];
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		assert_ok!(VCManagement::add_schema(
			RuntimeOrigin::signed(alice.clone()),
			shard,
			id,
			content
		));
		assert!(VCManagement::schema_registry(0).is_some());
		assert_noop!(
			VCManagement::activate_schema(RuntimeOrigin::signed(alice), shard, 0),
			Error::<Test>::SchemaAlreadyActivated
		);
	});
}

#[test]
fn revoke_schema_works() {
	new_test_ext().execute_with(|| {
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		let id: Vec<u8> = vec![1, 2, 3, 4];
		let content: Vec<u8> = vec![5, 6, 7, 8];
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		assert_ok!(VCManagement::add_schema(
			RuntimeOrigin::signed(alice.clone()),
			shard,
			id,
			content
		));
		assert!(VCManagement::schema_registry(0).is_some());
		assert_ok!(VCManagement::revoke_schema(RuntimeOrigin::signed(alice), shard, 0));
		// schema is deleted
		assert!(VCManagement::schema_registry(0).is_none());
	});
}

#[test]
fn revoke_schema_with_non_existent_fails() {
	new_test_ext().execute_with(|| {
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		assert_noop!(
			VCManagement::revoke_schema(RuntimeOrigin::signed(alice), shard, 2),
			Error::<Test>::SchemaNotExists
		);
	});
}

#[test]
fn revoke_schema_with_unprivileged_origin_fails() {
	new_test_ext().execute_with(|| {
		let alice: SystemAccountId = get_signer(ALICE_PUBKEY);
		let bob: SystemAccountId = get_signer(BOB_PUBKEY);
		let id: Vec<u8> = vec![1, 2, 3, 4];
		let content: Vec<u8> = vec![5, 6, 7, 8];
		let shard: ShardIdentifier = H256::from_slice(&TEST8_MRENCLAVE);
		assert_ok!(VCManagement::add_schema(RuntimeOrigin::signed(alice), shard, id, content));
		assert_noop!(
			VCManagement::revoke_schema(RuntimeOrigin::signed(bob), shard, 0),
			Error::<Test>::RequireAdmin
		);
	});
}

#[test]
fn on_vc_issued_works() {
	new_test_ext().execute_with(|| {
		let signer: SystemAccountId = get_signer(TEST8_SIGNER_PUB);
		let alice: Identity = get_signer(ALICE_PUBKEY);
		let omni_account = alice.to_omni_account();
		assert_ok!(VCManagement::on_vc_issued(
			RuntimeOrigin::signed(signer),
			alice.clone(),
			Assertion::A1,
			omni_account.clone(),
			H256::default(),
		));
		System::assert_last_event(RuntimeEvent::VCManagement(crate::Event::VCIssuedNew {
			identity: alice,
			assertion: Assertion::A1,
			omni_account,
			req_ext_hash: H256::default(),
		}));
	});
}
