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

use crate::{mock::*, Error, Event, Inspect, Instance1, State};
use frame_support::{assert_noop, assert_ok};

#[test]
fn set_mint_state_check_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(HalvingMint::mint_state(), State::Stopped);
		assert_noop!(
			HalvingMint::set_mint_state(RuntimeOrigin::signed(1), State::Running),
			sp_runtime::DispatchError::BadOrigin,
		);
		assert_noop!(
			HalvingMint::set_mint_state(RuntimeOrigin::root(), State::Running),
			Error::<Test, Instance1>::MintNotStarted,
		);
		assert_ok!(HalvingMint::start_mint_from_next_block(
			RuntimeOrigin::root(),
			1,
			"Test".as_bytes().to_vec(),
			"Test".as_bytes().to_vec(),
			18
		));
		assert_eq!(HalvingMint::mint_state(), State::Running);
		assert_noop!(
			HalvingMint::set_mint_state(RuntimeOrigin::root(), State::Running),
			Error::<Test, Instance1>::MintStateUnchanged,
		);
		assert_ok!(HalvingMint::set_mint_state(RuntimeOrigin::root(), State::Stopped));
		assert_eq!(HalvingMint::mint_state(), State::Stopped);
		System::assert_last_event(Event::MintStateChanged { new_state: State::Stopped }.into());
	});
}

#[test]
fn start_mint_too_early_fails() {
	new_test_ext().execute_with(|| {
		assert_eq!(System::block_number(), 1);
		assert_noop!(
			HalvingMint::start_mint_from_block(
				RuntimeOrigin::root(),
				0,
				1,
				"Test".as_bytes().to_vec(),
				"Test".as_bytes().to_vec(),
				18
			),
			Error::<Test, Instance1>::StartBlockTooEarly,
		);
		assert_noop!(
			HalvingMint::start_mint_from_block(
				RuntimeOrigin::root(),
				1,
				1,
				"Test".as_bytes().to_vec(),
				"Test".as_bytes().to_vec(),
				18
			),
			Error::<Test, Instance1>::StartBlockTooEarly,
		);
		assert_ok!(HalvingMint::start_mint_from_block(
			RuntimeOrigin::root(),
			2,
			1,
			"Test".as_bytes().to_vec(),
			"Test".as_bytes().to_vec(),
			18
		));
		System::assert_last_event(Event::MintStarted { asset_id: 1, start_block: 2 }.into());
	});
}

#[test]
fn halving_mint_works() {
	new_test_ext().execute_with(|| {
		let beneficiary = HalvingMint::beneficiary_account();

		assert_eq!(System::block_number(), 1);
		assert_eq!(Assets::total_issuance(1), 0);
		assert_eq!(Assets::balance(1, beneficiary), 0);
		assert_ok!(HalvingMint::start_mint_from_next_block(
			RuntimeOrigin::root(),
			1,
			"Test".as_bytes().to_vec(),
			"Test".as_bytes().to_vec(),
			18
		));
		System::assert_last_event(Event::MintStarted { asset_id: 1, start_block: 2 }.into());

		run_to_block(2);
		// 50 tokens are minted
		assert_eq!(Assets::total_issuance(1), 50);
		assert_eq!(Assets::balance(1, beneficiary), 0);
		assert_eq!(Assets::balance(1, 1), 50);

		run_to_block(11);
		assert_eq!(Assets::total_issuance(1), 500);
		assert_eq!(Assets::balance(1, 1), 500);

		run_to_block(12);
		// the first halving
		assert_eq!(Assets::total_issuance(1), 525);
		assert_eq!(Assets::balance(1, 1), 525);

		run_to_block(22);
		// the second halving
		assert_eq!(Assets::total_issuance(1), 762);
		assert_eq!(Assets::balance(1, 1), 762);

		run_to_block(52);
		// the fifth halving - only 1 token is minted
		assert_eq!(Assets::total_issuance(1), 961);
		assert_eq!(Assets::balance(1, 1), 961);

		run_to_block(62);
		// the sixth halving - but 0 tokens will be minted
		assert_eq!(Assets::total_issuance(1), 970);
		assert_eq!(Assets::balance(1, 1), 970);

		run_to_block(1_000);
		// no changes since the sixth halving, the total minted token will be fixated on 970,
		// the "missing" 30 comes from the integer division and the total_issuance is too small.
		//
		// we'll have much accurate result in reality where token unit is 18 decimal
		assert_eq!(Assets::total_issuance(1), 970);
		assert_eq!(Assets::balance(1, 1), 970);
	});
}

#[test]
fn set_on_token_minted_state_works() {
	new_test_ext().execute_with(|| {
		let beneficiary = HalvingMint::beneficiary_account();

		assert_ok!(HalvingMint::start_mint_from_next_block(
			RuntimeOrigin::root(),
			1,
			"Test".as_bytes().to_vec(),
			"Test".as_bytes().to_vec(),
			18
		));
		assert_ok!(HalvingMint::set_on_token_minted_state(RuntimeOrigin::root(), State::Stopped));
		System::assert_last_event(
			Event::OnTokenMintedStateChanged { new_state: State::Stopped }.into(),
		);

		run_to_block(2);
		// 50 tokens are minted, but none is transferred away
		assert_eq!(Assets::total_issuance(1), 50);
		assert_eq!(Assets::balance(1, beneficiary), 50);
		assert_eq!(Assets::balance(1, 1), 0);

		run_to_block(10);
		assert_ok!(HalvingMint::set_on_token_minted_state(RuntimeOrigin::root(), State::Running));
		System::assert_last_event(
			Event::OnTokenMintedStateChanged { new_state: State::Running }.into(),
		);

		run_to_block(11);
		// start to transfer token
		assert_eq!(Assets::total_issuance(1), 500);
		assert_eq!(Assets::balance(1, beneficiary), 450);
		assert_eq!(Assets::balance(1, 1), 50);
	});
}

#[test]
fn set_mint_state_works() {
	new_test_ext().execute_with(|| {
		let beneficiary = HalvingMint::beneficiary_account();

		assert_ok!(HalvingMint::start_mint_from_next_block(
			RuntimeOrigin::root(),
			1,
			"Test".as_bytes().to_vec(),
			"Test".as_bytes().to_vec(),
			18
		));

		run_to_block(2);
		assert_eq!(Assets::total_issuance(1), 50);
		assert_eq!(Assets::balance(1, beneficiary), 0);
		assert_eq!(Assets::balance(1, 1), 50);
		// stop the minting
		assert_ok!(HalvingMint::set_mint_state(RuntimeOrigin::root(), State::Stopped));

		run_to_block(3);
		// no new tokens should be minted
		assert_eq!(Assets::total_issuance(1), 50);
		assert_eq!(Assets::balance(1, beneficiary), 0);
		assert_eq!(Assets::balance(1, 1), 50);

		run_to_block(4);
		// resume the minting
		assert_ok!(HalvingMint::set_mint_state(RuntimeOrigin::root(), State::Running));

		run_to_block(5);
		assert_eq!(Assets::total_issuance(1), 100);
		assert_eq!(Assets::balance(1, beneficiary), 0);
		assert_eq!(Assets::balance(1, 1), 100);
		assert_eq!(HalvingMint::skipped_blocks(), 2);

		// the first halving should be delayed to block 14
		run_to_block(14);
		assert_eq!(Assets::total_issuance(1), 525);
		assert_eq!(Assets::balance(1, beneficiary), 0);
		assert_eq!(Assets::balance(1, 1), 525);
	});
}
