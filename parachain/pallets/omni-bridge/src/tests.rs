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
		assert_eq!(OmniBridge::finalized_pay_out_nonce(foreign_chain()), 0);
		assert_eq!(OmniBridge::finalized_vote_nonce(foreign_chain()), 0);
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

#[test]
fn pay_out_with_not_relayer_fails() {
	new_test_ext(true).execute_with(|| {
		assert_noop!(
			OmniBridge::propose_pay_out(
				RuntimeOrigin::signed(dave()),
				(foreign_chain(), native_symbol()),
				[1u8; 20].to_vec(),
				1,
				native_pay_out_request(),
				true,
			),
			Error::<Test>::RequireRelayer
		);
	});
}

#[test]
fn pay_out_with_wrong_symbol_fails() {
	new_test_ext(true).execute_with(|| {
		assert_noop!(
			OmniBridge::propose_pay_out(
				RuntimeOrigin::signed(alice()),
				(foreign_chain(), asset_symbol()), // wrong symbol
				[1u8; 20].to_vec(),
				1,
				native_pay_out_request(),
				true,
			),
			Error::<Test>::AssetSymbolInvalid
		);
	});
}

#[test]
fn pay_out_with_threshold_1_works() {
	new_test_ext(true).execute_with(|| {
		assert_ok!(OmniBridge::propose_pay_out(
			RuntimeOrigin::signed(alice()),
			(foreign_chain(), native_symbol()),
			[1u8; 20].to_vec(),
			1,
			native_pay_out_request(),
			true,
		));
		assert_eq!(Balances::free_balance(alice()), 60);
		assert_eq!(OmniBridge::finalized_pay_out_nonce(foreign_chain()), 1);
		assert_eq!(OmniBridge::finalized_vote_nonce(foreign_chain()), 1);
		assert_eq!(OmniBridge::pay_out_votes((foreign_chain(), 1), native_pay_out_request()), None);
		System::assert_has_event(
			Event::PayOutVoted {
				who: alice(),
				source_chain: foreign_chain(),
				nonce: 1,
				req: native_pay_out_request(),
				aye: true,
			}
			.into(),
		);
		System::assert_has_event(
			Event::PaidOut {
				to: alice(),
				nonce: 1,
				asset: NativeOrWithId::Native,
				source_asset: (foreign_chain(), native_symbol()),
				source_address: [1u8; 20].to_vec(),
				amount: 10,
			}
			.into(),
		);

		assert_ok!(OmniBridge::propose_pay_out(
			RuntimeOrigin::signed(bob()),
			(foreign_chain(), asset_symbol()),
			[1u8; 20].to_vec(),
			2,
			asset_pay_out_request(),
			true,
		));
		assert_eq!(Assets::balance(TEST_ASSET, alice()), 110);
		assert_eq!(OmniBridge::finalized_pay_out_nonce(foreign_chain()), 2);
		assert_eq!(OmniBridge::finalized_vote_nonce(foreign_chain()), 2);
		assert_eq!(OmniBridge::pay_out_votes((foreign_chain(), 2), asset_pay_out_request()), None);

		// alice relaying the same request will error out
		assert_noop!(
			OmniBridge::propose_pay_out(
				RuntimeOrigin::signed(alice()),
				(foreign_chain(), asset_symbol()),
				[1u8; 20].to_vec(),
				2,
				asset_pay_out_request(),
				true,
			),
			Error::<Test>::PayOutNonceFinalized
		);

		// bob requests with nay
		assert_ok!(OmniBridge::propose_pay_out(
			RuntimeOrigin::signed(bob()),
			(foreign_chain(), native_symbol()),
			[1u8; 20].to_vec(),
			3,
			native_pay_out_request(),
			false,
		));
		assert_eq!(OmniBridge::finalized_pay_out_nonce(foreign_chain()), 2);
		assert_eq!(OmniBridge::finalized_vote_nonce(foreign_chain()), 2);
		assert_eq!(
			OmniBridge::pay_out_votes((foreign_chain(), 3), native_pay_out_request()),
			Some(PayOutVote { ayes: vec![], nays: vec![bob()], status: VoteStatus::Pending })
		);

		// charlie approves it
		assert_ok!(OmniBridge::propose_pay_out(
			RuntimeOrigin::signed(charlie()),
			(foreign_chain(), native_symbol()),
			[1u8; 20].to_vec(),
			3,
			native_pay_out_request(),
			true,
		));
		assert_eq!(OmniBridge::finalized_pay_out_nonce(foreign_chain()), 3);
		assert_eq!(OmniBridge::finalized_vote_nonce(foreign_chain()), 3);
		assert_eq!(OmniBridge::pay_out_votes((foreign_chain(), 3), native_pay_out_request()), None,);
		assert_eq!(Balances::free_balance(alice()), 70);
		System::assert_has_event(
			Event::PaidOut {
				to: alice(),
				nonce: 3,
				asset: NativeOrWithId::Native,
				source_asset: (foreign_chain(), native_symbol()),
				source_address: [1u8; 20].to_vec(),
				amount: 10,
			}
			.into(),
		);

		// charlie sends request with non-consecutive nonce (3 -> 6), which is allowed here,
		// but it probably implies some internal error with charlie
		assert_ok!(OmniBridge::propose_pay_out(
			RuntimeOrigin::signed(charlie()),
			(foreign_chain(), native_symbol()),
			[1u8; 20].to_vec(),
			6,
			native_pay_out_request(),
			true,
		));
		assert_eq!(OmniBridge::finalized_pay_out_nonce(foreign_chain()), 3); // blocked at 3
		assert_eq!(OmniBridge::finalized_vote_nonce(foreign_chain()), 6); // updated
		assert_eq!(OmniBridge::pay_out_votes((foreign_chain(), 4), native_pay_out_request()), None,);
		assert_eq!(OmniBridge::pay_out_votes((foreign_chain(), 5), native_pay_out_request()), None,);
		assert_eq!(
			OmniBridge::pay_out_votes((foreign_chain(), 6), native_pay_out_request()),
			Some(PayOutVote { ayes: vec![charlie()], nays: vec![], status: VoteStatus::Passed })
		);
		assert_eq!(Balances::free_balance(alice()), 80);

		// alice sends request with nonce 4
		assert_ok!(OmniBridge::propose_pay_out(
			RuntimeOrigin::signed(alice()),
			(foreign_chain(), native_symbol()),
			[1u8; 20].to_vec(),
			4,
			native_pay_out_request(),
			true,
		));
		assert_eq!(OmniBridge::finalized_pay_out_nonce(foreign_chain()), 4); // updated to 4
		assert_eq!(OmniBridge::finalized_vote_nonce(foreign_chain()), 6); // unchanged
		assert_eq!(Balances::free_balance(alice()), 90);
		assert_eq!(OmniBridge::pay_out_votes((foreign_chain(), 4), native_pay_out_request()), None,);
		assert_eq!(OmniBridge::pay_out_votes((foreign_chain(), 5), native_pay_out_request()), None,);
		assert_eq!(
			OmniBridge::pay_out_votes((foreign_chain(), 6), native_pay_out_request()),
			Some(PayOutVote { ayes: vec![charlie()], nays: vec![], status: VoteStatus::Passed })
		);

		// bob sends request with nonce 5
		assert_ok!(OmniBridge::propose_pay_out(
			RuntimeOrigin::signed(alice()),
			(foreign_chain(), native_symbol()),
			[1u8; 20].to_vec(),
			5,
			native_pay_out_request(),
			true,
		));
		assert_eq!(OmniBridge::finalized_pay_out_nonce(foreign_chain()), 6); // updated to 6
		assert_eq!(OmniBridge::finalized_vote_nonce(foreign_chain()), 6);
		assert_eq!(Balances::free_balance(alice()), 100);
		assert_eq!(OmniBridge::pay_out_votes((foreign_chain(), 4), native_pay_out_request()), None,);
		assert_eq!(OmniBridge::pay_out_votes((foreign_chain(), 5), native_pay_out_request()), None,);
		assert_eq!(OmniBridge::pay_out_votes((foreign_chain(), 6), native_pay_out_request()), None,);
	});
}

#[test]
fn pay_out_with_threshold_2_works() {
	new_test_ext(true).execute_with(|| {
		assert_ok!(OmniBridge::set_relayer_threshold(RuntimeOrigin::root(), 2));
		assert_ok!(OmniBridge::propose_pay_out(
			RuntimeOrigin::signed(alice()),
			(foreign_chain(), native_symbol()),
			[1u8; 20].to_vec(),
			1,
			native_pay_out_request(),
			true,
		));
		assert_eq!(OmniBridge::finalized_pay_out_nonce(foreign_chain()), 0);
		assert_eq!(Balances::free_balance(alice()), 50);
		assert_eq!(
			OmniBridge::pay_out_votes((foreign_chain(), 1), native_pay_out_request()),
			Some(PayOutVote { ayes: vec![alice()], nays: vec![], status: VoteStatus::Pending })
		);
		System::assert_has_event(
			Event::PayOutVoted {
				who: alice(),
				source_chain: foreign_chain(),
				nonce: 1,
				req: native_pay_out_request(),
				aye: true,
			}
			.into(),
		);

		assert_ok!(OmniBridge::propose_pay_out(
			RuntimeOrigin::signed(bob()),
			(foreign_chain(), native_symbol()),
			[1u8; 20].to_vec(),
			1,
			native_pay_out_request(),
			false,
		));
		assert_eq!(OmniBridge::finalized_pay_out_nonce(foreign_chain()), 0);
		assert_eq!(Balances::free_balance(alice()), 50);
		assert_eq!(
			OmniBridge::pay_out_votes((foreign_chain(), 1), native_pay_out_request()),
			Some(PayOutVote {
				ayes: vec![alice()],
				nays: vec![bob()],
				status: VoteStatus::Pending,
			})
		);
		System::assert_has_event(
			Event::PayOutVoted {
				who: bob(),
				source_chain: foreign_chain(),
				nonce: 1,
				req: native_pay_out_request(),
				aye: false,
			}
			.into(),
		);

		assert_ok!(OmniBridge::propose_pay_out(
			RuntimeOrigin::signed(charlie()),
			(foreign_chain(), native_symbol()),
			[1u8; 20].to_vec(),
			1,
			native_pay_out_request(),
			true,
		));
		assert_eq!(OmniBridge::finalized_pay_out_nonce(foreign_chain()), 1);
		assert_eq!(Balances::free_balance(alice()), 60);
		assert_eq!(OmniBridge::pay_out_votes((foreign_chain(), 1), native_pay_out_request()), None);
		System::assert_has_event(
			Event::PayOutVoted {
				who: charlie(),
				source_chain: foreign_chain(),
				nonce: 1,
				req: native_pay_out_request(),
				aye: true,
			}
			.into(),
		);
		System::assert_has_event(
			Event::PaidOut {
				to: alice(),
				nonce: 1,
				asset: NativeOrWithId::Native,
				source_asset: (foreign_chain(), native_symbol()),
				source_address: [1u8; 20].to_vec(),
				amount: 10,
			}
			.into(),
		);

		// alice requests with nonce 2, but a different request (with bob and charlie)
		assert_ok!(OmniBridge::propose_pay_out(
			RuntimeOrigin::signed(alice()),
			(foreign_chain(), asset_symbol()),
			[1u8; 20].to_vec(),
			2,
			asset_pay_out_request(),
			true,
		));
		assert_ok!(OmniBridge::propose_pay_out(
			RuntimeOrigin::signed(bob()),
			(foreign_chain(), native_symbol()),
			[1u8; 20].to_vec(),
			2,
			native_pay_out_request(),
			true,
		));

		// both requests are recorded in different vote entries
		assert_eq!(
			OmniBridge::pay_out_votes((foreign_chain(), 2), asset_pay_out_request()),
			Some(PayOutVote { ayes: vec![alice()], nays: vec![], status: VoteStatus::Pending })
		);
		assert_eq!(
			OmniBridge::pay_out_votes((foreign_chain(), 2), native_pay_out_request()),
			Some(PayOutVote { ayes: vec![bob()], nays: vec![], status: VoteStatus::Pending })
		);

		assert_ok!(OmniBridge::propose_pay_out(
			RuntimeOrigin::signed(charlie()),
			(foreign_chain(), native_symbol()),
			[1u8; 20].to_vec(),
			2,
			native_pay_out_request(),
			true,
		));

		assert_eq!(OmniBridge::finalized_pay_out_nonce(foreign_chain()), 2); // updated to 2
		assert_eq!(OmniBridge::finalized_vote_nonce(foreign_chain()), 2);
		assert_eq!(Balances::free_balance(alice()), 70);
		assert_eq!(Assets::balance(TEST_ASSET, alice()), 100);
		// native payout request is removed, but asset payout request still exists (and never gets removed)
		assert_eq!(OmniBridge::pay_out_votes((foreign_chain(), 2), native_pay_out_request()), None);
		assert_eq!(
			OmniBridge::pay_out_votes((foreign_chain(), 2), asset_pay_out_request()),
			Some(PayOutVote { ayes: vec![alice()], nays: vec![], status: VoteStatus::Pending })
		);
	});
}

#[test]
fn pay_out_with_threshold_2_fail_fast_works() {
	new_test_ext(true).execute_with(|| {
		assert_ok!(OmniBridge::set_relayer_threshold(RuntimeOrigin::root(), 2));
		assert_ok!(OmniBridge::propose_pay_out(
			RuntimeOrigin::signed(alice()),
			(foreign_chain(), native_symbol()),
			[1u8; 20].to_vec(),
			1,
			native_pay_out_request(),
			false,
		));
		assert_eq!(OmniBridge::finalized_pay_out_nonce(foreign_chain()), 0);
		assert_eq!(Balances::free_balance(alice()), 50);
		assert_eq!(
			OmniBridge::pay_out_votes((foreign_chain(), 1), native_pay_out_request()),
			Some(PayOutVote { ayes: vec![], nays: vec![alice()], status: VoteStatus::Pending })
		);

		assert_ok!(OmniBridge::propose_pay_out(
			RuntimeOrigin::signed(bob()),
			(foreign_chain(), native_symbol()),
			[1u8; 20].to_vec(),
			1,
			native_pay_out_request(),
			false,
		));
		assert_eq!(OmniBridge::finalized_pay_out_nonce(foreign_chain()), 1);
		assert_eq!(Balances::free_balance(alice()), 50);
		assert_eq!(OmniBridge::pay_out_votes((foreign_chain(), 1), native_pay_out_request()), None);
		System::assert_has_event(
			Event::PayOutRejected {
				to: alice(),
				nonce: 1,
				asset: NativeOrWithId::Native,
				source_asset: (foreign_chain(), native_symbol()),
				source_address: [1u8; 20].to_vec(),
				amount: 10,
			}
			.into(),
		);
	});
}

// this test:
// - alice relays payout request 1, 2, 3 - all with aye
// - charlie relays payout request 1 with nay, 2 and 3 with aye
// - bob relays payout request with 1 with nay later
#[test]
fn pay_out_with_ahead_of_relayer_works() {
	new_test_ext(true).execute_with(|| {
		assert_ok!(OmniBridge::set_relayer_threshold(RuntimeOrigin::root(), 2));
		for i in 1..4 {
			assert_ok!(OmniBridge::propose_pay_out(
				RuntimeOrigin::signed(alice()),
				(foreign_chain(), native_symbol()),
				[1u8; 20].to_vec(),
				i,
				native_pay_out_request(),
				true,
			));
			assert_eq!(OmniBridge::finalized_pay_out_nonce(foreign_chain()), 0);
			assert_eq!(Balances::free_balance(alice()), 50);
			assert_eq!(
				OmniBridge::pay_out_votes((foreign_chain(), i), native_pay_out_request()),
				Some(PayOutVote { ayes: vec![alice()], nays: vec![], status: VoteStatus::Pending })
			);
		}

		assert_ok!(OmniBridge::propose_pay_out(
			RuntimeOrigin::signed(charlie()),
			(foreign_chain(), native_symbol()),
			[1u8; 20].to_vec(),
			1,
			native_pay_out_request(),
			false,
		));
		assert_eq!(
			OmniBridge::pay_out_votes((foreign_chain(), 1), native_pay_out_request()),
			Some(PayOutVote {
				ayes: vec![alice()],
				nays: vec![charlie()],
				status: VoteStatus::Pending,
			})
		);

		for i in 2..4 {
			assert_ok!(OmniBridge::propose_pay_out(
				RuntimeOrigin::signed(charlie()),
				(foreign_chain(), native_symbol()),
				[1u8; 20].to_vec(),
				i,
				native_pay_out_request(),
				true,
			));
			assert_eq!(OmniBridge::finalized_pay_out_nonce(foreign_chain()), 0); // not updated
																		// request with nonce 2 and 3 should be passed and executed
			assert_eq!(
				OmniBridge::pay_out_votes((foreign_chain(), i), native_pay_out_request()),
				Some(PayOutVote {
					ayes: vec![alice(), charlie()],
					nays: vec![],
					status: VoteStatus::Passed,
				})
			);
			assert_eq!(OmniBridge::finalized_vote_nonce(foreign_chain()), i);
			System::assert_has_event(
				Event::FinalizedVoteNonceUpdated { source_chain: foreign_chain(), nonce: i }.into(),
			);
		}
		assert_eq!(
			OmniBridge::pay_out_votes((foreign_chain(), 1), native_pay_out_request()),
			Some(PayOutVote {
				ayes: vec![alice()],
				nays: vec![charlie()],
				status: VoteStatus::Pending,
			})
		);
		assert_eq!(OmniBridge::finalized_vote_nonce(foreign_chain()), 3);
		assert_eq!(Balances::free_balance(alice()), 70);

		assert_ok!(OmniBridge::propose_pay_out(
			RuntimeOrigin::signed(bob()),
			(foreign_chain(), native_symbol()),
			[1u8; 20].to_vec(),
			1,
			native_pay_out_request(),
			false,
		));
		assert_eq!(Balances::free_balance(alice()), 70);
		assert_eq!(OmniBridge::finalized_vote_nonce(foreign_chain()), 3);
		assert_eq!(OmniBridge::finalized_pay_out_nonce(foreign_chain()), 3); // updated to 3
		for i in 1..4 {
			assert_eq!(
				OmniBridge::pay_out_votes((foreign_chain(), i), native_pay_out_request()),
				None
			);
		}
		System::assert_has_event(
			Event::FinalizedPayOutNonceUpdated { source_chain: foreign_chain(), nonce: 3 }.into(),
		);
		System::assert_has_event(
			Event::PayOutRejected {
				to: alice(),
				nonce: 1,
				asset: NativeOrWithId::Native,
				source_asset: (foreign_chain(), native_symbol()),
				source_address: [1u8; 20].to_vec(),
				amount: 10,
			}
			.into(),
		);
	});
}
