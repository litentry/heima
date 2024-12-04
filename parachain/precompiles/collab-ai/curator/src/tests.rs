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

use crate::mock::RuntimeEvent;
use crate::mock::*;
use pallet_evm::AddressMapping;
// use pallet_evm::AddressMapping::GuardianPrecompileCall;
use crate::CuratorPrecompileCall;
use pallet_curator::Event;
use precompile_utils::testing::PrecompileTesterExt;
use sp_core::{H160, H256, U256};

pub type PCall<Runtime> = CuratorPrecompileCall<Runtime>;

#[test]
fn test_regist_curator() {
	new_test_ext().execute_with(|| {
		let curator: H160 = H160::from_low_u64_be(1001);
		let info_hash: H256 = H256::from([1u8; 32]);

		PrecompilesValue::get()
			.prepare_test(
				curator,
				H160::from_low_u64_be(1000),
				PCall::<Test>::regist_curator { info_hash },
			)
			.expect_no_logs()
			.execute_returns(());

		System::assert_last_event(RuntimeEvent::Curator(Event::CuratorRegisted {
			curator: TruncatedAddressMapping::into_account_id(curator),
			curator_index: 1,
			info_hash,
		}));
	});
}

#[test]
fn test_update_curator() {
	new_test_ext().execute_with(|| {
		let curator: H160 = H160::from_low_u64_be(1001);
		let info_hash: H256 = H256::from([1u8; 32]);
		let updated_hash: H256 = H256::from([2u8; 32]);

		PrecompilesValue::get()
			.prepare_test(
				curator,
				H160::from_low_u64_be(1000),
				PCall::<Test>::regist_curator { info_hash },
			)
			.execute_returns(());

		PrecompilesValue::get()
			.prepare_test(
				curator,
				H160::from_low_u64_be(1000),
				PCall::<Test>::update_curator { info_hash: updated_hash },
			)
			.execute_returns(());

		System::assert_last_event(RuntimeEvent::Curator(Event::CuratorUpdated {
			curator: TruncatedAddressMapping::into_account_id(curator),
			curator_index: 1,
			info_hash: updated_hash,
		}));
	});
}

#[test]
fn test_clean_curator() {
	new_test_ext().execute_with(|| {
		let curator: H160 = H160::from_low_u64_be(1001);
		let info_hash: H256 = H256::from([1u8; 32]);

		PrecompilesValue::get()
			.prepare_test(
				curator,
				H160::from_low_u64_be(1000),
				PCall::<Test>::regist_curator { info_hash },
			)
			.execute_returns(());

		PrecompilesValue::get()
			.prepare_test(curator, H160::from_low_u64_be(1000), PCall::<Test>::clean_curator {})
			.execute_returns(());

		System::assert_last_event(RuntimeEvent::Curator(Event::CuratorCleaned {
			curator: TruncatedAddressMapping::into_account_id(curator),
			curator_index: 1,
		}));
	});
}

#[test]
fn test_public_curator_count() {
	new_test_ext().execute_with(|| {
		// Initially, there should be no curators
		PrecompilesValue::get()
			.prepare_test(
				H160::from_low_u64_be(1001),
				H160::from_low_u64_be(1000),
				PCall::<Test>::public_curator_count {},
			)
			.execute_returns(U256::from(1)); // Provide expected result

		// Register a curator to increase the count
		let info_hash: H256 = H256::from([1u8; 32]);
		PrecompilesValue::get()
			.prepare_test(
				H160::from_low_u64_be(1001),
				H160::from_low_u64_be(1000),
				PCall::<Test>::regist_curator { info_hash },
			)
			.execute_returns(());

		// Check the curator count again, should be 1
		PrecompilesValue::get()
			.prepare_test(
				H160::from_low_u64_be(1001),
				H160::from_low_u64_be(1000),
				PCall::<Test>::public_curator_count {},
			)
			.execute_returns(U256::from(2));
	});
}

#[test]
fn test_public_curator_to_index() {
	new_test_ext().execute_with(|| {
		let curator: H160 = H160::from_low_u64_be(1001);
		let info_hash: H256 = H256::from([1u8; 32]);

		// Register the curator
		PrecompilesValue::get()
			.prepare_test(
				curator,
				H160::from_low_u64_be(1000),
				PCall::<Test>::regist_curator { info_hash },
			)
			.execute_returns(());

		// Query the curator's index
		let curator_account = TruncatedAddressMapping::into_account_id(curator);
		let curator_account: [u8; 32] = curator_account.into();
		let curator_account: H256 = curator_account.into();
		PrecompilesValue::get()
			.prepare_test(
				curator,
				H160::from_low_u64_be(1000),
				PCall::<Test>::public_curator_to_index { curator: curator_account },
			)
			.execute_returns((true, U256::from(1)));
	});
}

#[test]
fn test_curator_index_to_info() {
	new_test_ext().execute_with(|| {
		let curator: H160 = H160::from_low_u64_be(1001);
		let info_hash: H256 = H256::from([1u8; 32]);

		// Register the guardian
		PrecompilesValue::get()
			.prepare_test(
				curator,
				H160::from_low_u64_be(1000),
				PCall::<Test>::regist_curator { info_hash },
			)
			.execute_returns(());

		// Query the curator info by index
		let curator_account = TruncatedAddressMapping::into_account_id(curator);
		let curator_account: [u8; 32] = curator_account.into();
		let curator_account: H256 = curator_account.into();
		PrecompilesValue::get()
			.prepare_test(
				curator,
				H160::from_low_u64_be(1000),
				PCall::<Test>::curator_index_to_info { index: 1.into() },
			)
			.execute_returns(crate::CuratorQueryResult {
				exist: true,
				info_hash,
				update_block: U256::from(1),
				curator: curator_account,
				status: 0u8,
			});
	});
}

#[test]
fn test_batch_curator_index_to_info() {
	new_test_ext().execute_with(|| {
		let curator: H160 = H160::from_low_u64_be(1001);
		let info_hash: H256 = H256::from([1u8; 32]);

		// Register the guardian
		PrecompilesValue::get()
			.prepare_test(
				curator,
				H160::from_low_u64_be(1000),
				PCall::<Test>::regist_curator { info_hash },
			)
			.execute_returns(());

		// Query the curator info by index
		let curator_account = TruncatedAddressMapping::into_account_id(curator);
		let curator_account: [u8; 32] = curator_account.into();
		let curator_account: H256 = curator_account.into();

		PrecompilesValue::get()
			.prepare_test(
				curator,
				H160::from_low_u64_be(1000),
				PCall::<Test>::batch_curator_index_to_info { start_id: 1.into(), end_id: 2.into() },
			)
			.execute_returns(vec![crate::CuratorQueryResult {
				exist: true,
				info_hash,
				update_block: U256::from(1),
				curator: curator_account,
				status: 0u8,
			}]);
	});
}
