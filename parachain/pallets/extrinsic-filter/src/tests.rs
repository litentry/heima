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

use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::Dispatchable;

#[test]
fn set_mode_works() {
	new_test_ext().execute_with(|| {
		// default mode should be `Normal`
		assert_eq!(ExtrinsicFilter::mode(), crate::OperationalMode::Normal);
		assert_ok!(ExtrinsicFilter::set_mode(RuntimeOrigin::root(), crate::OperationalMode::Test));
		assert_eq!(ExtrinsicFilter::mode(), crate::OperationalMode::Test);
		System::assert_last_event(RuntimeEvent::ExtrinsicFilter(crate::Event::ModeSet {
			new_mode: crate::OperationalMode::Test,
		}));
	});
}

#[test]
fn set_mode_fails_with_unauthorized_origin() {
	new_test_ext().execute_with(|| {
		assert_eq!(ExtrinsicFilter::mode(), crate::OperationalMode::Normal);
		assert_noop!(
			ExtrinsicFilter::set_mode(RuntimeOrigin::signed(1), crate::OperationalMode::Test),
			sp_runtime::DispatchError::BadOrigin
		);
	});
}

#[test]
fn set_mode_should_not_clear_blocked_extrinsics() {
	new_test_ext().execute_with(|| {
		assert_eq!(ExtrinsicFilter::mode(), crate::OperationalMode::Normal);

		// block Balances.transfer
		assert_ok!(ExtrinsicFilter::block_extrinsics(
			RuntimeOrigin::root(),
			b"Balances".to_vec(),
			Some(b"transfer_keep_alive".to_vec())
		));
		assert_eq!(
			ExtrinsicFilter::blocked_extrinsics((
				b"Balances".to_vec(),
				b"transfer_keep_alive".to_vec()
			)),
			Some(())
		);

		assert_ok!(ExtrinsicFilter::set_mode(RuntimeOrigin::root(), crate::OperationalMode::Test));
		assert_eq!(ExtrinsicFilter::mode(), crate::OperationalMode::Test);
		// previously blocked extrinsics are still there
		assert_eq!(
			ExtrinsicFilter::blocked_extrinsics((
				b"Balances".to_vec(),
				b"transfer_keep_alive".to_vec()
			)),
			Some(())
		);
	});
}

#[test]
fn safe_mode_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(ExtrinsicFilter::set_mode(RuntimeOrigin::root(), crate::OperationalMode::Safe));
		// SafeModeFilter allows frame_system calls
		let call: RuntimeCall = frame_system::Call::remark { remark: vec![] }.into();
		assert_ok!(call.dispatch(RuntimeOrigin::signed(1)));

		// SafeModeFilter disallows pallet_timestamp calls
		let call: RuntimeCall = pallet_timestamp::Call::set { now: 100 }.into();
		assert_noop!(
			call.dispatch(RuntimeOrigin::none()),
			frame_system::Error::<Test>::CallFiltered
		);

		// SafeModeFilter disallows balance calls
		let call: RuntimeCall =
			pallet_balances::Call::transfer_keep_alive { dest: 2, value: 10 }.into();
		assert_noop!(
			call.dispatch(RuntimeOrigin::signed(1)),
			frame_system::Error::<Test>::CallFiltered
		);
	});
}

#[test]
fn normal_mode_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(ExtrinsicFilter::set_mode(
			RuntimeOrigin::root(),
			crate::OperationalMode::Normal
		));
		// NormalModeFilter allows frame_system calls
		let call: RuntimeCall = frame_system::Call::remark { remark: vec![] }.into();
		assert_ok!(call.dispatch(RuntimeOrigin::signed(1)));

		// NormalModeFilter allows pallet_timestamp calls
		let call: RuntimeCall = pallet_timestamp::Call::set { now: 100 }.into();
		assert_ok!(call.dispatch(RuntimeOrigin::none()));

		// NormalModeFilter disallows balance calls
		let call: RuntimeCall =
			pallet_balances::Call::transfer_keep_alive { dest: 2, value: 10 }.into();
		assert_noop!(
			call.dispatch(RuntimeOrigin::signed(1)),
			frame_system::Error::<Test>::CallFiltered
		);
	});
}

#[test]
fn test_mode_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(ExtrinsicFilter::set_mode(RuntimeOrigin::root(), crate::OperationalMode::Test));
		// TestModeFilter allows frame_system calls
		let call: RuntimeCall = frame_system::Call::remark { remark: vec![] }.into();
		assert_ok!(call.dispatch(RuntimeOrigin::signed(1)));

		// TestModeFilter allows pallet_timestamp calls
		let call: RuntimeCall = pallet_timestamp::Call::set { now: 100 }.into();
		assert_ok!(call.dispatch(RuntimeOrigin::none()));

		// TestModeFilter allows balance calls
		let call: RuntimeCall =
			pallet_balances::Call::transfer_keep_alive { dest: 2, value: 10 }.into();
		assert_ok!(call.dispatch(RuntimeOrigin::signed(1)));
		assert_eq!(Balances::free_balance(2), 10);
	});
}

#[test]
fn block_single_extrinsic_works() {
	new_test_ext().execute_with(|| {
		// TestModeFilter allows everything
		assert_ok!(ExtrinsicFilter::set_mode(RuntimeOrigin::root(), crate::OperationalMode::Test));

		// block Balances.transfer
		assert_ok!(ExtrinsicFilter::block_extrinsics(
			RuntimeOrigin::root(),
			b"Balances".to_vec(),
			Some(b"transfer_keep_alive".to_vec())
		));
		System::assert_last_event(RuntimeEvent::ExtrinsicFilter(crate::Event::ExtrinsicsBlocked {
			pallet_name_bytes: b"Balances".to_vec(),
			function_name_bytes: Some(b"transfer_keep_alive".to_vec()),
		}));
		assert_eq!(
			ExtrinsicFilter::blocked_extrinsics((
				b"Balances".to_vec(),
				b"transfer_keep_alive".to_vec()
			)),
			Some(())
		);
		// try to dispatch Balances.transfer should fail
		let call: RuntimeCall =
			pallet_balances::Call::transfer_keep_alive { dest: 2, value: 10 }.into();
		assert_noop!(
			call.dispatch(RuntimeOrigin::signed(1)),
			frame_system::Error::<Test>::CallFiltered
		);
	});
}

#[test]
fn block_whole_pallet_works() {
	new_test_ext().execute_with(|| {
		// TestModeFilter allows everything
		assert_ok!(ExtrinsicFilter::set_mode(RuntimeOrigin::root(), crate::OperationalMode::Test));

		// block the whole Balances pallet
		assert_ok!(ExtrinsicFilter::block_extrinsics(
			RuntimeOrigin::root(),
			b"Balances".to_vec(),
			None
		));
		System::assert_last_event(RuntimeEvent::ExtrinsicFilter(crate::Event::ExtrinsicsBlocked {
			pallet_name_bytes: b"Balances".to_vec(),
			function_name_bytes: None,
		}));
		assert_eq!(
			ExtrinsicFilter::blocked_extrinsics((b"Balances".to_vec(), Vec::<u8>::default())),
			Some(())
		);
		// try to dispatch Balances.transfer should fail
		let call: RuntimeCall =
			pallet_balances::Call::transfer_keep_alive { dest: 2, value: 10 }.into();
		assert_noop!(
			call.dispatch(RuntimeOrigin::signed(1)),
			frame_system::Error::<Test>::CallFiltered
		);

		// Balances.transfer_keep_alive should fail too
		let call: RuntimeCall =
			pallet_balances::Call::transfer_keep_alive { dest: 2, value: 10 }.into();
		assert_noop!(
			call.dispatch(RuntimeOrigin::signed(1)),
			frame_system::Error::<Test>::CallFiltered
		);
		assert_eq!(Balances::free_balance(2), 0);
	});
}

#[test]
fn unblock_single_extrinsic_works() {
	new_test_ext().execute_with(|| {
		// TestModeFilter allows everything
		assert_ok!(ExtrinsicFilter::set_mode(RuntimeOrigin::root(), crate::OperationalMode::Test));

		// block Balances.transfer
		assert_ok!(ExtrinsicFilter::block_extrinsics(
			RuntimeOrigin::root(),
			b"Balances".to_vec(),
			Some(b"transfer_keep_alive".to_vec())
		));
		System::assert_last_event(RuntimeEvent::ExtrinsicFilter(crate::Event::ExtrinsicsBlocked {
			pallet_name_bytes: b"Balances".to_vec(),
			function_name_bytes: Some(b"transfer_keep_alive".to_vec()),
		}));
		assert_eq!(
			ExtrinsicFilter::blocked_extrinsics((
				b"Balances".to_vec(),
				b"transfer_keep_alive".to_vec()
			)),
			Some(())
		);
		// try to dispatch Balances.transfer should fail
		let call: RuntimeCall =
			pallet_balances::Call::transfer_keep_alive { dest: 2, value: 10 }.into();
		assert_noop!(
			call.dispatch(RuntimeOrigin::signed(1)),
			frame_system::Error::<Test>::CallFiltered
		);

		// unblock Balances.transfer
		assert_ok!(ExtrinsicFilter::unblock_extrinsics(
			RuntimeOrigin::root(),
			b"Balances".to_vec(),
			Some(b"transfer_keep_alive".to_vec())
		));

		System::assert_last_event(RuntimeEvent::ExtrinsicFilter(
			crate::Event::ExtrinsicsUnblocked {
				pallet_name_bytes: b"Balances".to_vec(),
				function_name_bytes: Some(b"transfer_keep_alive".to_vec()),
			},
		));
		assert_eq!(
			ExtrinsicFilter::blocked_extrinsics((
				b"Balances".to_vec(),
				b"transfer_keep_alive".to_vec()
			)),
			None
		);
		let call: RuntimeCall =
			pallet_balances::Call::transfer_keep_alive { dest: 2, value: 10 }.into();
		assert_ok!(call.dispatch(RuntimeOrigin::signed(1)));
		assert_eq!(Balances::free_balance(2), 10);
	});
}

#[test]
fn unblock_whole_pallet_works() {
	new_test_ext().execute_with(|| {
		// TestModeFilter allows everything
		assert_ok!(ExtrinsicFilter::set_mode(RuntimeOrigin::root(), crate::OperationalMode::Test));

		// block the whole Balances pallet
		assert_ok!(ExtrinsicFilter::block_extrinsics(
			RuntimeOrigin::root(),
			b"Balances".to_vec(),
			None
		));
		System::assert_last_event(RuntimeEvent::ExtrinsicFilter(crate::Event::ExtrinsicsBlocked {
			pallet_name_bytes: b"Balances".to_vec(),
			function_name_bytes: None,
		}));
		assert_eq!(
			ExtrinsicFilter::blocked_extrinsics((b"Balances".to_vec(), Vec::<u8>::default())),
			Some(())
		);
		// try to dispatch Balances.transfer should fail
		let call: RuntimeCall =
			pallet_balances::Call::transfer_keep_alive { dest: 2, value: 10 }.into();
		assert_noop!(
			call.dispatch(RuntimeOrigin::signed(1)),
			frame_system::Error::<Test>::CallFiltered
		);

		// Balances.transfer_keep_alive should fail too
		let call: RuntimeCall =
			pallet_balances::Call::transfer_keep_alive { dest: 2, value: 10 }.into();
		assert_noop!(
			call.dispatch(RuntimeOrigin::signed(1)),
			frame_system::Error::<Test>::CallFiltered
		);
		assert_eq!(Balances::free_balance(2), 0);

		// unblock the whole Balances pallet
		assert_ok!(ExtrinsicFilter::unblock_extrinsics(
			RuntimeOrigin::root(),
			b"Balances".to_vec(),
			None
		));
		System::assert_last_event(RuntimeEvent::ExtrinsicFilter(
			crate::Event::ExtrinsicsUnblocked {
				pallet_name_bytes: b"Balances".to_vec(),
				function_name_bytes: None,
			},
		));
		assert_eq!(
			ExtrinsicFilter::blocked_extrinsics((b"Balances".to_vec(), Vec::<u8>::default())),
			None
		);
		// try to dispatch Balances.transfer should work
		let call: RuntimeCall =
			pallet_balances::Call::transfer_keep_alive { dest: 2, value: 10 }.into();
		assert_ok!(call.dispatch(RuntimeOrigin::signed(1)));
		assert_eq!(Balances::free_balance(2), 10);

		// Balances.transfer_keep_alive should work too
		let call: RuntimeCall =
			pallet_balances::Call::transfer_keep_alive { dest: 2, value: 10 }.into();
		assert_ok!(call.dispatch(RuntimeOrigin::signed(1)));
		assert_eq!(Balances::free_balance(2), 20);
	});
}

#[test]
fn whitelisting_fails() {
	new_test_ext().execute_with(|| {
		// we disallow whitelisting, so set it to NormalMode and then try to unblock some
		// extrinsics will not work
		assert_ok!(ExtrinsicFilter::set_mode(
			RuntimeOrigin::root(),
			crate::OperationalMode::Normal
		));
		assert_noop!(
			ExtrinsicFilter::unblock_extrinsics(
				RuntimeOrigin::root(),
				b"Balances".to_vec(),
				Some(b"transfer_keep_alive".to_vec())
			),
			Error::<Test>::ExtrinsicNotBlocked
		);
		let call: RuntimeCall =
			pallet_balances::Call::transfer_keep_alive { dest: 2, value: 10 }.into();
		assert_noop!(
			call.dispatch(RuntimeOrigin::signed(1)),
			frame_system::Error::<Test>::CallFiltered
		);
	});
}

#[test]
fn block_this_pallet_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(ExtrinsicFilter::set_mode(RuntimeOrigin::root(), crate::OperationalMode::Test));

		assert_noop!(
			ExtrinsicFilter::block_extrinsics(
				RuntimeOrigin::root(),
				b"ExtrinsicFilter".to_vec(),
				Some(b"block_extrinsics".to_vec()),
			),
			Error::<Test>::CannotBlock
		);

		assert_eq!(
			ExtrinsicFilter::blocked_extrinsics((
				b"ExtrinsicFilter".to_vec(),
				b"block_extrinsics".to_vec()
			)),
			None
		);
	});
}

#[test]
fn block_more_than_once_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(ExtrinsicFilter::set_mode(RuntimeOrigin::root(), crate::OperationalMode::Test));

		assert_ok!(ExtrinsicFilter::block_extrinsics(
			RuntimeOrigin::root(),
			b"Balances".to_vec(),
			Some(b"transfer_keep_alive".to_vec()),
		));

		assert_noop!(
			ExtrinsicFilter::block_extrinsics(
				RuntimeOrigin::root(),
				b"Balances".to_vec(),
				Some(b"transfer_keep_alive".to_vec()),
			),
			Error::<Test>::ExtrinsicAlreadyBlocked
		);

		// Balances.transfer should be blocked though
		let call: RuntimeCall =
			pallet_balances::Call::transfer_keep_alive { dest: 2, value: 10 }.into();
		assert_noop!(
			call.dispatch(RuntimeOrigin::signed(1)),
			frame_system::Error::<Test>::CallFiltered
		);
	});
}

#[test]
fn unpaired_block_unblock_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(ExtrinsicFilter::set_mode(RuntimeOrigin::root(), crate::OperationalMode::Test));

		// block a single extrinsic and unblock the whole pallet should fail
		assert_ok!(ExtrinsicFilter::block_extrinsics(
			RuntimeOrigin::root(),
			b"Balances".to_vec(),
			Some(b"transfer_keep_alive".to_vec()),
		));
		assert_noop!(
			ExtrinsicFilter::unblock_extrinsics(RuntimeOrigin::root(), b"Balances".to_vec(), None),
			Error::<Test>::ExtrinsicNotBlocked
		);

		// Balances.transfer should still be blocked
		let call: RuntimeCall =
			pallet_balances::Call::transfer_keep_alive { dest: 2, value: 10 }.into();
		assert_noop!(
			call.dispatch(RuntimeOrigin::signed(1)),
			frame_system::Error::<Test>::CallFiltered
		);

		// clear the storage
		let _ = crate::BlockedExtrinsics::<Test>::clear(u32::MAX, None);

		// block the whole pallet and unblock a single extrinsic should fail
		assert_ok!(ExtrinsicFilter::block_extrinsics(
			RuntimeOrigin::root(),
			b"Balances".to_vec(),
			None,
		));
		assert_noop!(
			ExtrinsicFilter::unblock_extrinsics(
				RuntimeOrigin::root(),
				b"Balances".to_vec(),
				Some(b"Balances".to_vec()),
			),
			Error::<Test>::ExtrinsicNotBlocked
		);

		// Balances.transfer should still be blocked
		let call: RuntimeCall =
			pallet_balances::Call::transfer_keep_alive { dest: 2, value: 10 }.into();
		assert_noop!(
			call.dispatch(RuntimeOrigin::signed(1)),
			frame_system::Error::<Test>::CallFiltered
		);
		// Balances.transfer_keep_alive should fail too
		let call: RuntimeCall =
			pallet_balances::Call::transfer_keep_alive { dest: 2, value: 10 }.into();
		assert_noop!(
			call.dispatch(RuntimeOrigin::signed(1)),
			frame_system::Error::<Test>::CallFiltered
		);
	});
}

#[test]
fn blocked_extrinsic_is_retained_upon_mode_switch() {
	new_test_ext().execute_with(|| {
		assert_ok!(ExtrinsicFilter::set_mode(RuntimeOrigin::root(), crate::OperationalMode::Safe));

		// SafeModeFilter disallows pallet_timestamp calls
		let call: RuntimeCall = pallet_timestamp::Call::set { now: 100 }.into();
		assert_noop!(
			call.clone().dispatch(RuntimeOrigin::none()),
			frame_system::Error::<Test>::CallFiltered
		);

		// block the whole pallet_timestamp
		assert_ok!(ExtrinsicFilter::block_extrinsics(
			RuntimeOrigin::root(),
			b"Timestamp".to_vec(),
			None,
		));

		// switch to NormalMode, calls to pallet_timestamp should still be filtered out
		assert_ok!(ExtrinsicFilter::set_mode(
			RuntimeOrigin::root(),
			crate::OperationalMode::Normal
		));
		assert_noop!(
			call.clone().dispatch(RuntimeOrigin::none()),
			frame_system::Error::<Test>::CallFiltered
		);

		// unblock from NormalMode works
		assert_ok!(ExtrinsicFilter::unblock_extrinsics(
			RuntimeOrigin::root(),
			b"Timestamp".to_vec(),
			None,
		));

		// ... and takes effect
		assert_ok!(call.dispatch(RuntimeOrigin::none()));
	});
}
