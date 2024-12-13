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

use super::mock::*;
use frame_support::assert_ok;
use hex_literal::hex;
use pallet_evm::EnsureAddressOrigin;
use parity_scale_codec::Encode;
use sp_core::{H160, U256};
#[test]
fn address_mapping() {
	new_test_ext().execute_with(|| {
		pub const ALICE: AccountId = sp_runtime::AccountId32::new(hex![
			"d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
		]);
		// Check address mapping logic state
		assert_eq!(
			H160::from_slice(&ALICE.encode()[0..20]),
			H160::from_slice(&hex!["d43593c715Fdd31c61141ABd04a99FD6822c8558"])
		);
		assert_ok!(EnsureAddressEqualAndStore::<Test>::try_address_origin(
			&H160::from_slice(&hex!["d43593c715Fdd31c61141ABd04a99FD6822c8558"]),
			RuntimeOrigin::signed(ALICE),
		));
	})
}

#[test]
fn evm_ethereum_pallet_call_test() {
	new_test_ext().execute_with(|| {
		// EVM test, account without actual evm private key
		pub const ALICE: AccountId = sp_runtime::AccountId32::new(hex![
			"d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
		]);
		let alice_evm: AccountId = sp_runtime::AccountId32::new(hex![
			"d43593c715fdd31c61141abd04a99fd6822c8558000000000000000000000000"
		]);
		pub const BOB: AccountId = sp_runtime::AccountId32::new(hex![
			"fe65717dad0447d715f660a0a58411de509b42e6000000000000000000000000"
		]);

		assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), BOB, 100_000_000_000_000));
		assert_eq!(Balances::free_balance(ALICE), 8_000_000_000_000_000_000);
		assert_ok!(EVM::call(
			RuntimeOrigin::signed(ALICE),
			H160::from_slice(&ALICE.encode()[0..20]),
			H160::from_slice(&BOB.encode()[0..20]),
			Vec::new(),
			U256::from(5_000_000_000_000u128),
			1000000,
			U256::from(1_000_000_000),
			None,
			None,
			Vec::new(),
		));

		assert_eq!(Balances::free_balance(BOB), 105_000_000_000_000);
		assert_eq!(Balances::free_balance(ALICE), 7_999_994_999_999_979_000);
		assert_eq!(Balances::free_balance(alice_evm), 10_000_000_000_000_000);
		System::assert_last_event(RuntimeEvent::EVM(pallet_evm::Event::<Test>::Executed {
			address: H160::from_slice(&BOB.encode()[0..20]),
		}));
	});
}

#[test]
fn evm_ethereum_pallet_create_test() {
	new_test_ext().execute_with(|| {
		// EVM test, account without actual evm private key
		pub const ALICE: AccountId = sp_runtime::AccountId32::new(hex![
			"d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
		]);
		pub const BOB: AccountId = sp_runtime::AccountId32::new(hex![
			"fe65717dad0447d715f660a0a58411de509b42e6000000000000000000000000"
		]);

		assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), BOB, 100_000_000_000_000));
		assert_eq!(Balances::free_balance(ALICE), 8_000_000_000_000_000_000);

		assert_ok!(EVM::create(
			RuntimeOrigin::signed(ALICE),
			H160::from_slice(&ALICE.encode()[0..20]),
			Vec::from(hex!("608060405234801561001057600080fd5b50610113806100206000396000f3fe6080604052348015600f57600080fd5b506004361060325760003560e01c8063c2985578146037578063febb0f7e146057575b600080fd5b603d605f565b604051808215151515815260200191505060405180910390f35b605d6068565b005b60006001905090565b600060db576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001828103825260098152602001807f6572726f725f6d7367000000000000000000000000000000000000000000000081525060200191505060405180910390fd5b56fea2646970667358221220fde68a3968e0e99b16fabf9b2997a78218b32214031f8e07e2c502daf603a69e64736f6c63430006060033")),
			U256::from(0),
			1000000000,
			U256::from(1),
			None,
			None,
			Vec::new(),));
		System::assert_last_event(RuntimeEvent::EVM(pallet_evm::Event::<Test>::Created {
			address: H160::from_slice(&hex!["8a50db1e0f9452cfd91be8dc004ceb11cb08832f"]),
		}));
	});
}
