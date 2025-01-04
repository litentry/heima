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

#[test]
fn default_threshold_works() {
	new_test_ext(false).execute_with(|| {
		assert_eq!(OmniBridge::relayer_threshold(), 1);
	});
}

#[test]
fn pay_in_with_no_symbol_fails() {
	new_test_ext(false).execute_with(|| {
		assert_noop!(
			OmniBridge::pay_in(RuntimeOrigin::signed(alice()), native_pay_in_request()),
			Error::<Test>::AssetSymbolNotExist
		);
	});
}

#[test]
fn pay_in_with_pay_in_pair_not_listed_fails() {
	new_test_ext(false).execute_with(|| {
		assert_ok!(OmniBridge::set_asset_symbol(
			RuntimeOrigin::signed(alice()),
			NativeOrWithId::Native,
			native_symbol()
		));
		assert_noop!(
			OmniBridge::pay_in(RuntimeOrigin::signed(alice()), native_pay_in_request()),
			Error::<Test>::PayInPairNotAllowed
		);
	});
}

#[test]
fn pay_in_with_pay_in_fee_not_set_fails() {
	new_test_ext(false).execute_with(|| {
		assert_ok!(OmniBridge::set_asset_symbol(
			RuntimeOrigin::signed(alice()),
			NativeOrWithId::Native,
			native_symbol()
		));
		assert_ok!(OmniBridge::add_pay_in_pair(
			RuntimeOrigin::signed(alice()),
			NativeOrWithId::Native,
			(foreign_chain(), native_symbol())
		));
		assert_noop!(
			OmniBridge::pay_in(RuntimeOrigin::signed(alice()), native_pay_in_request()),
			Error::<Test>::PayInFeeNotSet
		);
	});
}

#[test]
fn pay_in_with_too_low_pay_in_amount_fails() {
	new_test_ext(false).execute_with(|| {
		assert_ok!(OmniBridge::set_asset_symbol(
			RuntimeOrigin::signed(alice()),
			NativeOrWithId::Native,
			native_symbol()
		));
		assert_ok!(OmniBridge::add_pay_in_pair(
			RuntimeOrigin::signed(alice()),
			NativeOrWithId::Native,
			(foreign_chain(), native_symbol())
		));
		assert_ok!(OmniBridge::set_pay_in_fee(
			RuntimeOrigin::signed(alice()),
			NativeOrWithId::Native,
			foreign_chain(),
			11 // the requested pay-in amount can't cover the fee
		));
		assert_noop!(
			OmniBridge::pay_in(RuntimeOrigin::signed(alice()), native_pay_in_request()),
			Error::<Test>::PayInAmountTooLow
		);
	});
}

#[test]
fn pay_in_with_insufficient_funds_fails() {
	new_test_ext(true).execute_with(|| {
		let pay_in_request = new_pay_in_request(NativeOrWithId::Native, 51);
		assert_noop!(
			OmniBridge::pay_in(RuntimeOrigin::signed(alice()), pay_in_request),
			DispatchError::Token(sp_runtime::TokenError::FundsUnavailable)
		);
	});
}

#[test]
fn pay_in_works() {
	new_test_ext(true).execute_with(|| {
		assert_ok!(OmniBridge::pay_in(RuntimeOrigin::signed(alice()), native_pay_in_request()));
		System::assert_last_event(
			Event::PaidIn {
				from: alice(),
				nonce: 1,
				asset: NativeOrWithId::Native,
				dest_asset: (foreign_chain(), native_symbol()),
				dest_address: [1u8; 20].to_vec(),
				amount: 8, // 10(amount) - 2(fee)
			}
			.into(),
		);

		assert_ok!(OmniBridge::pay_in(RuntimeOrigin::signed(alice()), asset_pay_in_request()));
		System::assert_last_event(
			Event::PaidIn {
				from: alice(),
				nonce: 2, // increased
				asset: NativeOrWithId::WithId(TEST_ASSET),
				dest_asset: (foreign_chain(), asset_symbol()),
				dest_address: [1u8; 20].to_vec(),
				amount: 7, // 10(amount) - 3(fee)
			}
			.into(),
		);

		assert_eq!(Balances::free_balance(dave()), 2); // native pay-in fee
		assert_eq!(Assets::balance(TEST_ASSET, dave()), 3); // asset pay-in fee

		assert_eq!(Balances::free_balance(alice()), 40);
		assert_eq!(Assets::balance(TEST_ASSET, alice()), 90);
	});
}
