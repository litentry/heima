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
use pallet_collab_ai_common::GuardianVote;
use pallet_evm::AddressMapping;
use pallet_guardian::Event;
// use pallet_evm::AddressMapping::GuardianPrecompileCall;
use crate::GuardianPrecompileCall;
use precompile_utils::testing::PrecompileTesterExt;
use sp_core::{H160, H256};

pub type PCall<Runtime> = GuardianPrecompileCall<Runtime>;

#[test]
fn test_regist_guardian() {
	new_test_ext().execute_with(|| {
		let guardian: H160 = H160::from_low_u64_be(1001);
		let info_hash: H256 = H256::from([1u8; 32]);

		PrecompilesValue::get()
			.prepare_test(
				guardian,
				H160::from_low_u64_be(1000),
				PCall::<Test>::regist_guardian { info_hash },
			)
			.expect_no_logs()
			.execute_returns(());

		System::assert_last_event(RuntimeEvent::Guardian(Event::GuardianRegisted {
			guardian: TruncatedAddressMapping::into_account_id(guardian),
			guardian_index: 0,
			info_hash,
		}));
	});
}

#[test]
fn test_update_guardian() {
	new_test_ext().execute_with(|| {
		let guardian: H160 = H160::from_low_u64_be(1001);
		let info_hash: H256 = H256::from([1u8; 32]);
		let updated_hash: H256 = H256::from([2u8; 32]);

		PrecompilesValue::get()
			.prepare_test(
				guardian,
				H160::from_low_u64_be(1000),
				PCall::<Test>::regist_guardian { info_hash },
			)
			.execute_returns(());

		PrecompilesValue::get()
			.prepare_test(
				guardian,
				H160::from_low_u64_be(1000),
				PCall::<Test>::update_guardian { info_hash: updated_hash },
			)
			.execute_returns(());

		System::assert_last_event(RuntimeEvent::Guardian(Event::GuardianUpdated {
			guardian: TruncatedAddressMapping::into_account_id(guardian),
			guardian_index: 0,
			info_hash: updated_hash,
		}));
	});
}

#[test]
fn test_clean_guardian() {
	new_test_ext().execute_with(|| {
		let guardian: H160 = H160::from_low_u64_be(1001);
		let info_hash: H256 = H256::from([1u8; 32]);

		PrecompilesValue::get()
			.prepare_test(
				guardian,
				H160::from_low_u64_be(1000),
				PCall::<Test>::regist_guardian { info_hash },
			)
			.execute_returns(());

		PrecompilesValue::get()
			.prepare_test(guardian, H160::from_low_u64_be(1000), PCall::<Test>::clean_guardian {})
			.execute_returns(());

		System::assert_last_event(RuntimeEvent::Guardian(Event::GuardianCleaned {
			guardian: TruncatedAddressMapping::into_account_id(guardian),
			guardian_index: 0,
		}));
	});
}

#[test]
fn test_vote_for_guardian() {
	new_test_ext().execute_with(|| {
		let guardian: H160 = H160::from_low_u64_be(1001);
		let voter: H160 = H160::from_low_u64_be(1002);
		let info_hash: H256 = H256::from([1u8; 32]);

		// Register the guardian first
		PrecompilesValue::get()
			.prepare_test(
				guardian,
				H160::from_low_u64_be(1000),
				PCall::<Test>::regist_guardian { info_hash },
			)
			.execute_returns(());

		// Ensure the guardian is successfully registered before proceeding
		assert!(pallet_guardian::Pallet::<Test>::public_guardian_to_index(
			TruncatedAddressMapping::into_account_id(guardian)
		)
		.is_some());

		let guardian_account = TruncatedAddressMapping::into_account_id(guardian);
		let guardian_account: [u8; 32] = guardian_account.into();
		let guardian_account: H256 = guardian_account.into();
		// Cast the vote
		PrecompilesValue::get()
			.prepare_test(
				voter,
				H160::from_low_u64_be(1000),
				PCall::<Test>::vote {
					guardian: guardian_account,
					status: 1,
					potential_proposal_index: 0.into(),
				},
			)
			.execute_returns(());

		System::assert_last_event(RuntimeEvent::Guardian(Event::VoteGuardian {
			voter: TruncatedAddressMapping::into_account_id(voter),
			guardian_index: 0,
			guardian: TruncatedAddressMapping::into_account_id(guardian),
			status: Some(GuardianVote::Aye),
		}));
	});
}

#[test]
fn test_remove_all_votes() {
	new_test_ext().execute_with(|| {
		let guardian: H160 = H160::from_low_u64_be(1001);
		let voter: H160 = H160::from_low_u64_be(1002);
		let info_hash: H256 = H256::from([1u8; 32]);

		// Register the guardian first
		PrecompilesValue::get()
			.prepare_test(
				guardian,
				H160::from_low_u64_be(1000),
				PCall::<Test>::regist_guardian { info_hash },
			)
			.execute_returns(());

		// Ensure the guardian is successfully registered before proceeding
		assert!(pallet_guardian::Pallet::<Test>::public_guardian_to_index(
			TruncatedAddressMapping::into_account_id(guardian)
		)
		.is_some());

		let guardian_account = TruncatedAddressMapping::into_account_id(guardian);
		let guardian_account: [u8; 32] = guardian_account.into();
		let guardian_account: H256 = guardian_account.into();
		// Cast a vote
		PrecompilesValue::get()
			.prepare_test(
				voter,
				H160::from_low_u64_be(1000),
				PCall::<Test>::vote {
					guardian: guardian_account,
					status: 1,
					potential_proposal_index: 0.into(),
				},
			)
			.execute_returns(());

		// Remove all votes
		PrecompilesValue::get()
			.prepare_test(voter, H160::from_low_u64_be(1000), PCall::<Test>::remove_all_votes {})
			.execute_returns(());

		System::assert_last_event(RuntimeEvent::Guardian(Event::RemoveAllVote {
			voter: TruncatedAddressMapping::into_account_id(voter),
		}));
	});
}
