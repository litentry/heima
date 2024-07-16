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

extern crate alloc;
use crate::{mock::*, *};
use frame_support::assert_ok;
use precompile_utils::testing::*;

fn precompiles() -> BridgeTransferMockPrecompile<Test> {
	PrecompilesValue::get()
}

#[test]
fn transfer_native_is_ok() {
	new_test_ext().execute_with(|| {
		let dest_bridge_id: pallet_bridge::BridgeChainId = 0;
		let resource_id = NativeTokenResourceId::get();
		let dest_account: [u8; 64] = vec![1].into();
		assert_ok!(pallet_bridge::Pallet::<Test>::update_fee(
			RuntimeOrigin::root(),
			dest_bridge_id,
			10
		));
		assert_ok!(pallet_bridge::Pallet::<Test>::whitelist_chain(
			RuntimeOrigin::root(),
			dest_bridge_id
		));

		precompiles()
			.prepare_test(
				U8Wrapper(1u8),
				precompile_address(),
				PCall::transfer_native {
					amount: 100u128.into(),
					receipt: dest_account.into(),
					dest_id: dest_bridge_id.into(),
				},
			)
			.expect_no_logs()
			.execute_returns(());

		assert_eq!(
			pallet_balances::Pallet::<Test>::free_balance(TreasuryAccount::get()),
			ENDOWED_BALANCE + 10
		);
		assert_eq!(
			pallet_balances::Pallet::<Test>::free_balance(Into::<AccountId>::into(U8Wrapper(1u8))),
			ENDOWED_BALANCE - 100
		);
		assert_events(vec![
			mock::RuntimeEvent::Balances(pallet_balances::Event::Deposit {
				who: TreasuryAccount::get(),
				amount: 10,
			}),
			RuntimeEvent::Bridge(pallet_bridge::Event::FungibleTransfer(
				dest_bridge_id,
				1,
				resource_id,
				100 - 10,
				dest_account,
			)),
		]);
	})
}
