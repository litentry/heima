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

use crate::{mock::*, AccountStore, MemberAccountHash, *};
use core_primitives::{CallEthereum, Identity};
use frame_support::{assert_noop, assert_ok};
use sp_core::hashing::blake2_256;
use sp_core::H160;
use sp_runtime::{traits::BadOrigin, ModuleError};
use sp_std::vec;

fn add_account_call<T: pallet::Config<Permission = mock::Permission>>(
	account: MemberAccount,
	permissions: Option<Vec<<T as pallet::Config>::Permission>>,
) -> Box<RuntimeCall> {
	let call =
		RuntimeCall::OmniAccount(crate::Call::add_account { member_account: account, permissions });
	Box::new(call)
}

fn remove_accounts_call(hashes: Vec<H256>) -> Box<RuntimeCall> {
	let call =
		RuntimeCall::OmniAccount(crate::Call::remove_accounts { member_account_hashes: hashes });
	Box::new(call)
}

fn publicize_account_call(id: Identity) -> Box<RuntimeCall> {
	let call = RuntimeCall::OmniAccount(crate::Call::publicize_account { member_account: id });
	Box::new(call)
}

fn request_intent_call(intent: Intent) -> Box<RuntimeCall> {
	RuntimeCall::OmniAccount(crate::Call::request_intent { intent }).into()
}

fn make_balance_transfer_call(dest: AccountId, value: Balance) -> Box<RuntimeCall> {
	let call = RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive { dest, value });
	Box::new(call)
}

#[test]
fn create_account_store_works() {
	new_test_ext().execute_with(|| {
		let tee_signer = get_tee_signer();

		assert_ok!(OmniAccount::create_account_store(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity,
		));

		let member_accounts: MemberAccounts<Test> =
			vec![public_member_account(alice())].try_into().unwrap();

		System::assert_has_event(Event::AccountStoreCreated { who: alice().omni_account }.into());
		System::assert_last_event(
			Event::AccountStoreUpdated {
				who: alice().omni_account,
				account_store: member_accounts,
			}
			.into(),
		);

		// create it the second time will fail
		assert_noop!(
			OmniAccount::create_account_store(RuntimeOrigin::signed(tee_signer), alice().identity),
			Error::<Test>::AccountAlreadyAdded
		);
	});
}

#[test]
fn add_account_without_creating_store_fails() {
	new_test_ext().execute_with(|| {
		let tee_signer = get_tee_signer();
		let call = add_account_call::<Test>(private_member_account(bob()), None);

		assert_noop!(
			OmniAccount::dispatch_as_omni_account(
				RuntimeOrigin::signed(tee_signer.clone()),
				alice().identity.hash(),
				call,
			),
			Error::<Test>::AccountNotFound
		);
	});
}

#[test]
fn add_account_works() {
	new_test_ext().execute_with(|| {
		let tee_signer = get_tee_signer();

		let bob = private_member_account(bob());
		let charlie = public_member_account(charlie());

		let expected_member_accounts: MemberAccounts<Test> =
			vec![public_member_account(alice()), bob.clone()].try_into().unwrap();

		assert_ok!(OmniAccount::create_account_store(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity,
		));

		let call = add_account_call::<Test>(bob.clone(), None);
		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.hash(),
			call,
		));

		System::assert_has_event(
			Event::AccountAdded {
				who: alice().omni_account.clone(),
				member_account_hash: bob.hash(),
			}
			.into(),
		);
		System::assert_has_event(
			Event::AccountStoreUpdated {
				who: alice().omni_account.clone(),
				account_store: expected_member_accounts.clone(),
			}
			.into(),
		);

		assert_eq!(
			AccountStore::<Test>::get(alice().omni_account).unwrap(),
			expected_member_accounts
		);

		let call = add_account_call::<Test>(charlie.clone(), None);
		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.hash(),
			call,
		));
		let expected_member_accounts: MemberAccounts<Test> = BoundedVec::truncate_from(vec![
			public_member_account(alice()),
			bob.clone(),
			charlie.clone(),
		]);

		System::assert_has_event(
			Event::AccountAdded { who: alice().omni_account, member_account_hash: charlie.hash() }
				.into(),
		);
		System::assert_has_event(
			Event::AccountStoreUpdated {
				who: alice().omni_account,
				account_store: expected_member_accounts.clone(),
			}
			.into(),
		);

		assert!(MemberAccountHash::<Test>::contains_key(bob.hash()));
		assert!(MemberAccountHash::<Test>::contains_key(charlie.hash()));
		assert!(MemberAccountPermissions::<Test>::contains_key(bob.hash()));
		assert_eq!(
			MemberAccountPermissions::<Test>::get(bob.hash()).to_vec(),
			vec![Permission::All]
		);
		assert!(MemberAccountPermissions::<Test>::contains_key(charlie.hash()));
		assert_eq!(
			MemberAccountPermissions::<Test>::get(charlie.hash()).to_vec(),
			vec![Permission::All]
		);
	});
}

#[test]
fn add_account_origin_check_works() {
	new_test_ext().execute_with(|| {
		let tee_signer = get_tee_signer();
		let bob = private_member_account(bob());

		assert_noop!(
			OmniAccount::add_account(RuntimeOrigin::signed(tee_signer), bob.clone(), None),
			BadOrigin
		);

		assert_noop!(
			OmniAccount::add_account(RuntimeOrigin::signed(alice().omni_account), bob, None),
			BadOrigin
		);
	});
}

#[test]
fn add_account_with_already_linked_account_fails() {
	new_test_ext().execute_with(|| {
		let tee_signer = get_tee_signer();
		let bob = public_member_account(bob());

		assert_ok!(OmniAccount::create_account_store(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.clone(),
		));

		let call = add_account_call::<Test>(bob.clone(), None);
		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.hash(),
			call.clone(),
		));

		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.hash(),
			call,
		));

		System::assert_has_event(
			Event::DispatchedAsOmniAccount {
				who: alice().omni_account,
				result: Err(DispatchError::Module(ModuleError {
					index: 5,
					error: [0, 0, 0, 0],
					message: Some("AccountAlreadyAdded"),
				})),
			}
			.into(),
		);

		// intent to create a new AccountStore with an account that is already added
		assert_ok!(OmniAccount::create_account_store(
			RuntimeOrigin::signed(tee_signer.clone()),
			charlie().identity,
		));

		let call = add_account_call::<Test>(public_member_account(alice()), None);
		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			charlie().identity.hash(),
			call,
		));

		System::assert_has_event(
			Event::DispatchedAsOmniAccount {
				who: alice().omni_account,
				result: Err(DispatchError::Module(ModuleError {
					index: 5,
					error: [0, 0, 0, 0],
					message: Some("AccountAlreadyAdded"),
				})),
			}
			.into(),
		);
	});
}

#[test]
fn add_account_store_len_limit_reached_works() {
	new_test_ext().execute_with(|| {
		let tee_signer = get_tee_signer();

		assert_ok!(OmniAccount::create_account_store(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity,
		));

		let member_accounts: MemberAccounts<Test> = vec![
			public_member_account(alice()),
			private_member_account(bob()),
			private_member_account(charlie()),
		]
		.try_into()
		.unwrap();

		AccountStore::<Test>::insert(alice().omni_account, member_accounts);

		let call = add_account_call::<Test>(
			MemberAccount::Private(vec![7, 8, 9], H256::from(blake2_256(&[7, 8, 9]))),
			None,
		);

		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.hash(),
			call,
		));

		System::assert_has_event(
			Event::DispatchedAsOmniAccount {
				who: alice().omni_account,
				result: Err(DispatchError::Module(ModuleError {
					index: 5,
					error: [1, 0, 0, 0],
					message: Some("AccountStoreLenLimitReached"),
				})),
			}
			.into(),
		);
	});
}

#[test]
fn remove_account_works() {
	new_test_ext().execute_with(|| {
		let tee_signer = get_tee_signer();
		let bob = private_member_account(bob());

		assert_ok!(OmniAccount::create_account_store(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity,
		));

		let call = add_account_call::<Test>(bob.clone(), None);
		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.hash(),
			call,
		));

		// normal signed origin should give `BadOrigin`, no matter
		// it's from TEE-worker, or omni-account itself
		assert_noop!(
			OmniAccount::remove_accounts(
				RuntimeOrigin::signed(tee_signer.clone()),
				vec![bob.hash()]
			),
			sp_runtime::DispatchError::BadOrigin
		);

		assert_noop!(
			OmniAccount::remove_accounts(
				RuntimeOrigin::signed(alice().omni_account),
				vec![bob.hash()]
			),
			sp_runtime::DispatchError::BadOrigin
		);

		let call = remove_accounts_call(vec![bob.hash()]);
		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.hash(),
			call,
		));

		System::assert_has_event(
			Event::DispatchedAsOmniAccount {
				who: alice().omni_account,
				result: DispatchResult::Ok(()),
			}
			.into(),
		);

		let expected_member_accounts: MemberAccounts<Test> =
			BoundedVec::truncate_from(vec![public_member_account(alice())]);

		System::assert_has_event(
			Event::AccountRemoved {
				who: alice().omni_account,
				member_account_hashes: vec![bob.hash()],
			}
			.into(),
		);
		System::assert_has_event(
			Event::AccountStoreUpdated {
				who: alice().omni_account,
				account_store: expected_member_accounts.clone(),
			}
			.into(),
		);

		assert_eq!(
			AccountStore::<Test>::get(alice().omni_account).unwrap(),
			expected_member_accounts
		);
		assert!(!MemberAccountHash::<Test>::contains_key(bob.hash()));

		let call = remove_accounts_call(vec![alice().identity.hash()]);
		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.hash(),
			call,
		));

		assert!(!AccountStore::<Test>::contains_key(alice().omni_account));
	});
}

#[test]
fn remove_account_empty_account_check_works() {
	new_test_ext().execute_with(|| {
		let tee_signer = get_tee_signer();

		assert_ok!(OmniAccount::create_account_store(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity,
		));

		let bob = private_member_account(bob());
		let call = add_account_call::<Test>(bob, None);
		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.hash(),
			call,
		));

		let call = remove_accounts_call(vec![]);
		// execution itself is ok, but error is shown in the dispatch result
		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.hash(),
			call,
		));
		System::assert_has_event(
			Event::DispatchedAsOmniAccount {
				who: alice().omni_account,
				result: Err(DispatchError::Module(ModuleError {
					index: 5,
					error: [5, 0, 0, 0],
					message: Some("EmptyAccount"),
				})),
			}
			.into(),
		);
	});
}

#[test]
fn publicize_account_works() {
	new_test_ext().execute_with(|| {
		let tee_signer = get_tee_signer();
		let private_bob = private_member_account(bob());
		let public_bob = public_member_account(bob());

		assert_ok!(OmniAccount::create_account_store(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.clone(),
		));

		let call = add_account_call::<Test>(private_bob.clone(), None);
		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.hash(),
			call,
		));

		let expected_member_accounts: MemberAccounts<Test> =
			vec![public_member_account(alice()), private_bob.clone()].try_into().unwrap();
		assert_eq!(
			AccountStore::<Test>::get(alice().omni_account).unwrap(),
			expected_member_accounts
		);

		let call = publicize_account_call(bob().identity);
		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.hash(),
			call,
		));

		System::assert_has_event(
			Event::DispatchedAsOmniAccount {
				who: alice().omni_account,
				result: DispatchResult::Ok(()),
			}
			.into(),
		);

		let expected_member_accounts: MemberAccounts<Test> =
			BoundedVec::truncate_from(vec![public_member_account(alice()), public_bob]);

		System::assert_has_event(
			Event::AccountMadePublic {
				who: alice().omni_account,
				member_account_hash: bob().identity.hash(),
			}
			.into(),
		);
		System::assert_has_event(
			Event::AccountStoreUpdated {
				who: alice().omni_account,
				account_store: expected_member_accounts.clone(),
			}
			.into(),
		);

		assert_eq!(
			AccountStore::<Test>::get(alice().omni_account).unwrap(),
			expected_member_accounts
		);
	});
}

#[test]
fn publicize_account_identity_not_found_works() {
	new_test_ext().execute_with(|| {
		let tee_signer = get_tee_signer();
		let bob = private_member_account(bob());

		let call = publicize_account_call(charlie().identity);
		assert_noop!(
			OmniAccount::dispatch_as_omni_account(
				RuntimeOrigin::signed(tee_signer.clone()),
				alice().identity.hash(),
				call,
			),
			Error::<Test>::AccountNotFound
		);

		assert_ok!(OmniAccount::create_account_store(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity,
		));

		let call = add_account_call::<Test>(bob.clone(), None);
		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.hash(),
			call,
		));

		let call = publicize_account_call(charlie().identity);
		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.hash(),
			call,
		));
		System::assert_has_event(
			Event::DispatchedAsOmniAccount {
				who: alice().omni_account,
				result: Err(DispatchError::Module(ModuleError {
					index: 5,
					error: [2, 0, 0, 0],
					message: Some("AccountNotFound"),
				})),
			}
			.into(),
		);
	});
}

#[test]
fn request_intent_works() {
	new_test_ext().execute_with(|| {
		let tee_signer = get_tee_signer();
		let bob = private_member_account(bob());

		assert_ok!(OmniAccount::create_account_store(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity
		));

		let call = add_account_call::<Test>(bob, None);
		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.hash(),
			call,
		));

		let intent =
			Intent::CallEthereum(CallEthereum { address: H160::zero(), input: BoundedVec::new() });

		let call = request_intent_call(intent.clone());
		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.hash(),
			call,
		));

		System::assert_has_event(
			Event::DispatchedAsOmniAccount {
				who: alice().omni_account,
				result: DispatchResult::Ok(()),
			}
			.into(),
		);

		System::assert_has_event(
			Event::IntentRequested { who: alice().omni_account, intent }.into(),
		);
	});
}

#[test]
fn dispatch_as_signed_works() {
	new_test_ext().execute_with(|| {
		let tee_signer = get_tee_signer();

		assert_ok!(Balances::transfer_keep_alive(
			RuntimeOrigin::signed(alice().native_account),
			alice().omni_account,
			6
		));

		assert_ok!(OmniAccount::create_account_store(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity,
		));

		let call = add_account_call::<Test>(private_member_account(bob()), None);
		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.hash(),
			call,
		));

		let call = make_balance_transfer_call(bob().native_account, 5);
		assert_ok!(OmniAccount::dispatch_as_signed(
			RuntimeOrigin::signed(tee_signer),
			alice().identity.hash(),
			call
		));
		System::assert_has_event(
			Event::DispatchedAsSigned { who: alice().omni_account, result: DispatchResult::Ok(()) }
				.into(),
		);

		assert_eq!(Balances::free_balance(bob().native_account), 5);
	});
}

#[test]
fn dispatch_as_omni_account_increments_omni_account_nonce() {
	new_test_ext().execute_with(|| {
		let tee_signer = get_tee_signer();

		let bob = private_member_account(bob());

		assert_ok!(OmniAccount::create_account_store(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity,
		));

		assert_eq!(System::account_nonce(alice().omni_account), 0);

		let call = add_account_call::<Test>(bob.clone(), None);
		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity.hash(),
			call,
		));
		assert_eq!(System::account_nonce(alice().omni_account), 1);
	});
}

#[test]
fn dispatch_as_signed_account_increments_omni_account_nonce() {
	new_test_ext().execute_with(|| {
		let tee_signer = get_tee_signer();

		assert_ok!(Balances::transfer_keep_alive(
			RuntimeOrigin::signed(alice().native_account),
			alice().omni_account,
			6
		));

		assert_ok!(OmniAccount::create_account_store(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().identity,
		));

		assert_eq!(System::account_nonce(alice().omni_account), 0);

		let call = make_balance_transfer_call(bob().native_account, 5);

		assert_ok!(OmniAccount::dispatch_as_signed(
			RuntimeOrigin::signed(tee_signer),
			alice().identity.hash(),
			call
		));
		assert_eq!(System::account_nonce(alice().omni_account), 1);
	});
}
