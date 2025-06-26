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

use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};
use sp_std::vec;

fn request_intent_call(intent: Intent) -> Box<RuntimeCall> {
	RuntimeCall::OmniAccount(crate::Call::request_intent { intent }).into()
}

fn make_balance_transfer_call(dest: AccountId, value: Balance) -> Box<RuntimeCall> {
	let call = RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive { dest, value });
	Box::new(call)
}

#[test]
fn request_intent_works() {
	new_test_ext().execute_with(|| {
		let intent = Intent::TransferNative(TransferNative {
			to: AccountId::from([0u8; 32]),
			value: Default::default(),
		});

		// Request through OmniAccount origin
		assert_ok!(OmniAccount::request_intent(
			RuntimeOrigin::from(RawOrigin::OmniAccount(alice().omni_account)),
			intent.clone()
		));

		System::assert_last_event(
			Event::IntentRequested { who: alice().omni_account, intent }.into(),
		);
	})
}

#[test]
fn dispatch_as_signed_works() {
	new_test_ext().execute_with(|| {
		let tee_signer = get_tee_signer();
		let dest = bob().native_account;
		let value = 5;
		let call = make_balance_transfer_call(dest, value);
		
		// Fund the omni account so it can make the transfer
		assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), alice().omni_account, 10));

		let res = OmniAccount::dispatch_as_signed(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().omni_account,
			call,
			Some(OmniAccountAuthType::Web3),
		);
		assert_ok!(res);

		// Ensure the transfer succeeded - bob should have received 5
		assert_eq!(Balances::free_balance(bob().native_account), 5);
		// And alice's omni account should have 5 left
		assert_eq!(Balances::free_balance(alice().omni_account), 5);
	});
}

#[test]
fn dispatch_as_omni_account_increments_omni_account_nonce() {
	new_test_ext().execute_with(|| {
		let alice_omni_account_nonce = System::account_nonce(&alice().omni_account);
		let tee_signer = get_tee_signer();
		let dest = bob().native_account;
		let value = 5;
		let call = make_balance_transfer_call(dest, value);

		assert_ok!(OmniAccount::dispatch_as_omni_account(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().omni_account,
			call,
			None,
		));

		assert_eq!(
			System::account_nonce(&alice().omni_account),
			alice_omni_account_nonce + 1
		);
	});
}

#[test]
fn dispatch_as_signed_account_increments_omni_account_nonce() {
	new_test_ext().execute_with(|| {
		let alice_omni_account_nonce = System::account_nonce(&alice().omni_account);
		let tee_signer = get_tee_signer();
		let dest = bob().native_account;
		let value = 5;
		let call = make_balance_transfer_call(dest, value);

		assert_ok!(OmniAccount::dispatch_as_signed(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().omni_account,
			call,
			None,
		));

		assert_eq!(
			System::account_nonce(&alice().omni_account),
			alice_omni_account_nonce + 1
		);
	});
}

#[test]
fn auth_token_requested_works() {
	new_test_ext().execute_with(|| {
		let tee_signer = get_tee_signer();
		let expires_at = 1234567890;

		assert_ok!(OmniAccount::auth_token_requested(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().omni_account,
			expires_at,
		));

		System::assert_has_event(
			Event::AuthTokenRequested { who: alice().omni_account, expires_at }.into(),
		);
	});
}


#[test]
fn omni_account_always_uses_converter() {
	new_test_ext().execute_with(|| {
		// Test that omni_account function always uses the converter
		let identity = alice().identity;
		let omni_account = OmniAccount::omni_account(TEST_CLIENT_ID.to_string(), identity.clone());
		
		// Should be the same as directly using the converter
		let expected = <Test as crate::Config>::OmniAccountConverter::convert(&identity, TEST_CLIENT_ID);
		assert_eq!(omni_account, expected);
	});
}

#[test]
fn intent_accepted_works() {
	new_test_ext().execute_with(|| {
		let tee_signer = get_tee_signer();
		let intent_id = 1u32;
		let intent = Intent::TransferNative(TransferNative {
			to: AccountId::from([0u8; 32]),
			value: Default::default(),
		});

		assert_ok!(OmniAccount::intent_accepted(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().omni_account,
			intent_id,
			intent.clone(),
		));

		// Check storage
		assert_eq!(OmniAccount::intents(alice().omni_account, intent_id), Some(intent.clone()));
		assert_eq!(OmniAccount::accepted_intent_ids(alice().omni_account), intent_id);

		// Check event
		System::assert_has_event(
			Event::IntentAccepted { who: alice().omni_account, intent_id, intent }.into(),
		);

		// Trying to accept the same intent again should fail
		let intent2 = Intent::SystemRemark(vec![1, 2, 3].try_into().unwrap());
		assert_noop!(
			OmniAccount::intent_accepted(
				RuntimeOrigin::signed(tee_signer),
				alice().omni_account,
				intent_id,
				intent2,
			),
			Error::<Test>::IntentAlreadyExists
		);
	});
}

#[test]
fn intent_completed_works() {
	new_test_ext().execute_with(|| {
		let tee_signer = get_tee_signer();
		let intent_id = 1u32;
		let detail = IntentCompletedDetail::Success;

		assert_ok!(OmniAccount::intent_completed(
			RuntimeOrigin::signed(tee_signer.clone()),
			alice().omni_account,
			intent_id,
			detail.clone(),
		));

		// Check storage
		assert_eq!(OmniAccount::completed_intent_ids(alice().omni_account), intent_id);

		// Check event
		System::assert_has_event(
			Event::IntentCompleted { who: alice().omni_account, intent_id, detail }.into(),
		);
	});
}