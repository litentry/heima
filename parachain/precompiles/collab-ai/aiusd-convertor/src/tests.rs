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
use pallet_aiusd_convertor::{Event, InspectFungibles};
use pallet_evm::AddressMapping;
use precompile_utils::testing::PrecompileTesterExt;
use sp_core::H160;
use sp_runtime::AccountId32;

#[test]
fn test_mint_and_burn_aiusd() {
	new_test_ext().execute_with(|| {
		let aiusd_asset_id = 1;
		let source_asset_id = 2;
		let target_decimal_ratio = 1_000_000;
		let target_asset_supply_amount: u128 = target_decimal_ratio * 1000;
		let h160_address: H160 = H160::from_low_u64_be(1001);
		let beneficiary =
			<TruncatedAddressMapping as AddressMapping<AccountId32>>::into_account_id(h160_address);

		// Check balance
		let aiusd_balance = InspectFungibles::<Test>::balance(aiusd_asset_id, &beneficiary);
		assert_eq!(aiusd_balance, 0);
		let target_balance = InspectFungibles::<Test>::balance(source_asset_id, &beneficiary);
		assert_eq!(target_balance, target_asset_supply_amount);

		// mint
		let mint_amount: u128 = 3_000_000_000_000_000_000; // 3 AIUSD (10^18 * 3)
		PrecompilesValue::get()
			.prepare_test(
				h160_address,
				precompile_address(),
				PCall::<Test>::mint_aiusd {
					asset_id: source_asset_id.into(),
					amount: mint_amount.into(),
				},
			)
			.expect_no_logs()
			.execute_returns(());

		let mint_asset_amount = target_decimal_ratio * 3;
		// Check balance after mint
		let aiusd_balance = InspectFungibles::<Test>::balance(aiusd_asset_id, &beneficiary);
		assert_eq!(aiusd_balance, mint_amount);
		let target_balance = InspectFungibles::<Test>::balance(source_asset_id, &beneficiary);
		assert_eq!(target_balance, target_asset_supply_amount - mint_asset_amount);
		System::assert_last_event(RuntimeEvent::AIUSD(Event::AIUSDCreated {
			beneficiary: beneficiary.clone(),
			aiusd_amount: mint_amount,
			asset_id: source_asset_id,
			asset_amount: mint_asset_amount,
		}));

		// burn
		let burn_amount: u128 = 2_000_000_000_000_000_000; // 2 AIUSD (10^18 * 2)
		PrecompilesValue::get()
			.prepare_test(
				h160_address,
				precompile_address(),
				PCall::<Test>::burn_aiusd {
					asset_id: source_asset_id.into(),
					amount: burn_amount.into(),
				},
			)
			.expect_no_logs()
			.execute_returns(());

		let burn_asset_amount = target_decimal_ratio * 2;
		// Check balance after burn
		let aiusd_balance = InspectFungibles::<Test>::balance(aiusd_asset_id, &beneficiary);
		assert_eq!(aiusd_balance, mint_amount - burn_amount);
		let target_balance = InspectFungibles::<Test>::balance(source_asset_id, &beneficiary);
		assert_eq!(
			target_balance,
			target_asset_supply_amount - mint_asset_amount + burn_asset_amount
		);
		System::assert_last_event(RuntimeEvent::AIUSD(Event::AIUSDDestroyed {
			beneficiary,
			aiusd_amount: burn_amount,
			asset_id: source_asset_id,
			asset_amount: burn_asset_amount,
		}));
	});
}
