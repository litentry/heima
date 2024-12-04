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
use crate::*;
use frame_support::{assert_err, assert_ok};
use sp_runtime::{AccountId32, TokenError};

#[test]
fn test_mint_aiusd() {
	new_test_ext().execute_with(|| {
		let beneficiary: AccountId32 = AccountId32::from([2u8; 32]);
		let aiusd_asset_id: u32 = 1;
		let source_asset_id: u32 = 2;
		let target_decimal_ratio = 1_000_000;
		let target_asset_supply_amount: u128 = target_decimal_ratio * 1000;
		let mint_amount: u128 = 2_000_000_000_000_000_000; // 2 AIUSD (10^18 * 2)

		// Check balance
		let aiusd_balance = InspectFungibles::<Test>::balance(aiusd_asset_id, &beneficiary);
		assert_eq!(aiusd_balance, 0);
		let target_balance = InspectFungibles::<Test>::balance(source_asset_id, &beneficiary);
		assert_eq!(target_balance, target_asset_supply_amount);

		assert_ok!(AIUSD::enable_token(
			RuntimeOrigin::root(),
			source_asset_id,
			target_decimal_ratio
		));

		assert_err!(
			AIUSD::mint_aiusd(RuntimeOrigin::signed(beneficiary.clone()), 3, mint_amount),
			Error::<Test>::AssetNotEnabled
		);

		assert_ok!(AIUSD::mint_aiusd(
			RuntimeOrigin::signed(beneficiary.clone()),
			source_asset_id,
			mint_amount
		));
		// Check balance after mint
		let asset_amount = target_decimal_ratio * 2;
		let aiusd_balance = InspectFungibles::<Test>::balance(aiusd_asset_id, &beneficiary);
		assert_eq!(aiusd_balance, mint_amount);
		let target_balance = InspectFungibles::<Test>::balance(source_asset_id, &beneficiary);
		assert_eq!(target_balance, target_asset_supply_amount - asset_amount);

		System::assert_last_event(RuntimeEvent::AIUSD(Event::AIUSDCreated {
			beneficiary,
			aiusd_amount: mint_amount,
			asset_id: source_asset_id,
			asset_amount,
		}));
	});
}

#[test]
fn test_burn_aiusd() {
	new_test_ext().execute_with(|| {
		let beneficiary: AccountId32 = AccountId32::from([2u8; 32]);
		let aiusd_asset_id: u32 = 1;
		let source_asset_id: u32 = 2;
		let target_decimal_ratio = 1_000_000;
		let target_asset_supply_amount: u128 = target_decimal_ratio * 1000;
		let aiusd_amount: u128 = 2_000_000_000_000_000_000; // 2 AIUSD (10^18 * 2)

		// Check balance
		let aiusd_balance = InspectFungibles::<Test>::balance(aiusd_asset_id, &beneficiary);
		assert_eq!(aiusd_balance, 0);
		let target_balance = InspectFungibles::<Test>::balance(source_asset_id, &beneficiary);
		assert_eq!(target_balance, target_asset_supply_amount);

		assert_ok!(AIUSD::enable_token(
			RuntimeOrigin::root(),
			source_asset_id,
			target_decimal_ratio
		));

		// FundsUnavailable
		assert_err!(
			AIUSD::burn_aiusd(
				RuntimeOrigin::signed(beneficiary.clone()),
				source_asset_id,
				aiusd_amount
			),
			TokenError::FundsUnavailable
		);

		assert_ok!(AIUSD::mint_aiusd(
			RuntimeOrigin::signed(beneficiary.clone()),
			source_asset_id,
			aiusd_amount
		));
		// Check balance after mint
		let aiusd_balance = InspectFungibles::<Test>::balance(aiusd_asset_id, &beneficiary);
		assert_eq!(aiusd_balance, aiusd_amount);
		let target_balance = InspectFungibles::<Test>::balance(source_asset_id, &beneficiary);
		assert_eq!(target_balance, target_asset_supply_amount - target_decimal_ratio * 2);

		assert_ok!(AIUSD::burn_aiusd(
			RuntimeOrigin::signed(beneficiary.clone()),
			source_asset_id,
			aiusd_amount
		));
		// Check balance after burn
		let aiusd_balance = InspectFungibles::<Test>::balance(aiusd_asset_id, &beneficiary);
		assert_eq!(aiusd_balance, 0);
		let target_balance = InspectFungibles::<Test>::balance(source_asset_id, &beneficiary);
		assert_eq!(target_balance, target_asset_supply_amount);

		System::assert_last_event(RuntimeEvent::AIUSD(Event::AIUSDDestroyed {
			beneficiary,
			aiusd_amount,
			asset_id: source_asset_id,
			asset_amount: target_decimal_ratio * 2,
		}));
	});
}

#[test]
fn test_enable_disable_token() {
	new_test_ext().execute_with(|| {
		let source_asset_id: u32 = 2;
		let decimal_ratio: u128 = 1_000_000;

		// enable
		assert_ok!(AIUSD::enable_token(RuntimeOrigin::root(), source_asset_id, decimal_ratio));
		assert!(AIUSD::enabled_tokens(source_asset_id).is_some());
		System::assert_last_event(RuntimeEvent::AIUSD(Event::AssetEnabled {
			asset_id: source_asset_id,
			decimal_ratio,
		}));

		// disable
		assert_ok!(AIUSD::disable_token(RuntimeOrigin::root(), source_asset_id));
		assert!(AIUSD::enabled_tokens(source_asset_id).is_none());

		System::assert_last_event(RuntimeEvent::AIUSD(Event::AssetDisabled {
			asset_id: source_asset_id,
		}));
	});
}
