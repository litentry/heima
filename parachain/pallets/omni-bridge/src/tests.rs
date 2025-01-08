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
		assert_eq!(OmniBridge::finalized_pay_out_nonce(ChainType::Ethereum(0)), 0);
		assert_eq!(OmniBridge::finalized_vote_nonce(ChainType::Ethereum(0)), 0);
		assert_eq!(
			hex::encode(native_resource_id()),
			"9ee6dfb61a2fb903df487c401663825643bb825d41695e63df8af6162ab145a6".to_string()
		);
	});
}

#[test]
fn pay_in_with_pay_in_pair_not_listed_fails() {
	new_test_ext(false).execute_with(|| {
		assert_noop!(
			OmniBridge::pay_in(RuntimeOrigin::signed(alice()), native_pay_in_request()),
			Error::<Test>::PayInPairNotAllowed
		);
	});
}

#[test]
fn pay_in_with_pay_in_fee_not_set_fails() {
	new_test_ext(false).execute_with(|| {
		assert_ok!(OmniBridge::add_pay_in_pair(
			RuntimeOrigin::signed(alice()),
			NativeOrWithId::Native,
			ChainType::Ethereum(0)
		));
		assert_noop!(
			OmniBridge::pay_in(RuntimeOrigin::signed(alice()), native_pay_in_request()),
			Error::<Test>::PayInFeeNotSet
		);
	});
}

#[test]
fn pay_in_with_too_low_pay_in_amount_fails() {
	new_test_ext(true).execute_with(|| {
		assert_ok!(OmniBridge::set_pay_in_fee(
			RuntimeOrigin::signed(alice()),
			NativeOrWithId::Native,
			ChainType::Ethereum(0),
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
				source_account: alice(),
				nonce: 1,
				asset: NativeOrWithId::Native,
				resource_id: native_resource_id(),
				dest_chain: ChainType::Ethereum(0),
				dest_account: [1u8; 20].to_vec(),
				amount: 8, // 10(amount) - 2(fee)
			}
			.into(),
		);

		assert_ok!(OmniBridge::pay_in(RuntimeOrigin::signed(alice()), asset_pay_in_request()));
		System::assert_last_event(
			Event::PaidIn {
				source_account: alice(),
				nonce: 2, // increased
				asset: NativeOrWithId::WithId(TEST_ASSET),
				resource_id: asset_resource_id(),
				dest_chain: ChainType::Ethereum(0),
				dest_account: [1u8; 20].to_vec(),
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
			OmniBridge::request_pay_out(
				RuntimeOrigin::signed(dave()),
				native_pay_out_request(1),
				true,
			),
			Error::<Test>::RequireRelayer
		);
	});
}

#[test]
fn pay_out_with_wrong_source_chain_fails() {
	new_test_ext(true).execute_with(|| {
		let mut req = native_pay_out_request(1);
		req.source_chain = ChainType::Heima;
		assert_noop!(
			OmniBridge::request_pay_out(RuntimeOrigin::signed(alice()), req, true,),
			Error::<Test>::ChainTypeInvalid
		);
	});
}

#[test]
fn pay_out_with_threshold_1_works() {
	new_test_ext(true).execute_with(|| {
		assert_ok!(OmniBridge::request_pay_out(
			RuntimeOrigin::signed(alice()),
			native_pay_out_request(1),
			true,
		));
		let hash1 = native_pay_out_request(1).hash();
		assert_eq!(Balances::free_balance(alice()), 60);
		assert_eq!(OmniBridge::finalized_pay_out_nonce(ChainType::Ethereum(0)), 1);
		assert_eq!(OmniBridge::finalized_vote_nonce(ChainType::Ethereum(0)), 1);

		assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 1), None);
		assert_eq!(OmniBridge::pay_out_votes(hash1), None);
		assert_eq!(OmniBridge::pay_out_requests(hash1), None);

		System::assert_has_event(
			Event::PayOutVoted {
				who: alice(),
				source_chain: ChainType::Ethereum(0),
				nonce: 1,
				asset: NativeOrWithId::Native,
				dest_account: alice(),
				amount: 10,
				aye: true,
			}
			.into(),
		);
		System::assert_has_event(
			Event::PaidOut {
				source_chain: ChainType::Ethereum(0),
				nonce: 1,
				asset: NativeOrWithId::Native,
				dest_account: alice(),
				amount: 10,
			}
			.into(),
		);

		assert_ok!(OmniBridge::request_pay_out(
			RuntimeOrigin::signed(bob()),
			asset_pay_out_request(2),
			true,
		));
		let hash2 = asset_pay_out_request(2).hash();
		assert_eq!(Assets::balance(TEST_ASSET, alice()), 110);
		assert_eq!(OmniBridge::finalized_pay_out_nonce(ChainType::Ethereum(0)), 2);
		assert_eq!(OmniBridge::finalized_vote_nonce(ChainType::Ethereum(0)), 2);

		assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 2), None);
		assert_eq!(OmniBridge::pay_out_votes(hash2), None);
		assert_eq!(OmniBridge::pay_out_requests(hash2), None);

		// alice relaying the same request will error out
		assert_noop!(
			OmniBridge::request_pay_out(
				RuntimeOrigin::signed(alice()),
				asset_pay_out_request(2),
				true,
			),
			Error::<Test>::PayOutNonceFinalized
		);

		// bob requests with nay
		assert_ok!(OmniBridge::request_pay_out(
			RuntimeOrigin::signed(bob()),
			native_pay_out_request(3),
			false,
		));
		let hash3 = native_pay_out_request(3).hash();
		assert_eq!(OmniBridge::finalized_pay_out_nonce(ChainType::Ethereum(0)), 2);
		assert_eq!(OmniBridge::finalized_vote_nonce(ChainType::Ethereum(0)), 2);

		assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 3), Some(vec![hash3]));
		assert_eq!(
			OmniBridge::pay_out_votes(hash3),
			Some(PayOutVote { ayes: vec![], nays: vec![bob()], status: VoteStatus::Pending })
		);
		assert_eq!(OmniBridge::pay_out_requests(hash3), Some(native_pay_out_request(3)));

		// charlie approves it
		assert_ok!(OmniBridge::request_pay_out(
			RuntimeOrigin::signed(charlie()),
			native_pay_out_request(3),
			true,
		));
		assert_eq!(OmniBridge::finalized_pay_out_nonce(ChainType::Ethereum(0)), 3);
		assert_eq!(OmniBridge::finalized_vote_nonce(ChainType::Ethereum(0)), 3);

		assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 2), None);
		assert_eq!(OmniBridge::pay_out_votes(hash3), None);
		assert_eq!(OmniBridge::pay_out_requests(hash3), None);

		assert_eq!(Balances::free_balance(alice()), 70);
		System::assert_has_event(
			Event::PaidOut {
				source_chain: ChainType::Ethereum(0),
				nonce: 3,
				asset: NativeOrWithId::Native,
				dest_account: alice(),
				amount: 10,
			}
			.into(),
		);

		// charlie sends request with non-consecutive nonce (3 -> 6), which is allowed here,
		// but it probably implies some internal error with charlie
		assert_ok!(OmniBridge::request_pay_out(
			RuntimeOrigin::signed(charlie()),
			native_pay_out_request(6),
			true,
		));
		let hash4 = native_pay_out_request(4).hash();
		let hash5 = native_pay_out_request(5).hash();
		let hash6 = native_pay_out_request(6).hash();

		assert_eq!(OmniBridge::finalized_pay_out_nonce(ChainType::Ethereum(0)), 3); // blocked at 3
		assert_eq!(OmniBridge::finalized_vote_nonce(ChainType::Ethereum(0)), 6); // updated

		assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 4), None);
		assert_eq!(OmniBridge::pay_out_votes(hash4), None);
		assert_eq!(OmniBridge::pay_out_requests(hash4), None);
		assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 5), None);
		assert_eq!(OmniBridge::pay_out_votes(hash5), None);
		assert_eq!(OmniBridge::pay_out_requests(hash5), None);
		assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 6), Some(vec![hash6]));
		assert_eq!(
			OmniBridge::pay_out_votes(hash6),
			Some(PayOutVote { ayes: vec![charlie()], nays: vec![], status: VoteStatus::Passed })
		);
		assert_eq!(OmniBridge::pay_out_requests(hash6), Some(native_pay_out_request(6)));

		assert_eq!(Balances::free_balance(alice()), 80);

		// alice sends request with nonce 4
		assert_ok!(OmniBridge::request_pay_out(
			RuntimeOrigin::signed(alice()),
			native_pay_out_request(4),
			true,
		));
		assert_eq!(OmniBridge::finalized_pay_out_nonce(ChainType::Ethereum(0)), 4); // updated to 4
		assert_eq!(OmniBridge::finalized_vote_nonce(ChainType::Ethereum(0)), 6); // unchanged

		assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 4), None);
		assert_eq!(OmniBridge::pay_out_votes(hash4), None);
		assert_eq!(OmniBridge::pay_out_requests(hash4), None);
		assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 5), None);
		assert_eq!(OmniBridge::pay_out_votes(hash5), None);
		assert_eq!(OmniBridge::pay_out_requests(hash5), None);
		assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 6), Some(vec![hash6]));
		assert_eq!(
			OmniBridge::pay_out_votes(hash6),
			Some(PayOutVote { ayes: vec![charlie()], nays: vec![], status: VoteStatus::Passed })
		);
		assert_eq!(OmniBridge::pay_out_requests(hash6), Some(native_pay_out_request(6)));

		assert_eq!(Balances::free_balance(alice()), 90);

		// bob sends request with nonce 5
		assert_ok!(OmniBridge::request_pay_out(
			RuntimeOrigin::signed(alice()),
			native_pay_out_request(5),
			true,
		));

		assert_eq!(OmniBridge::finalized_pay_out_nonce(ChainType::Ethereum(0)), 6); // updated to 6
		assert_eq!(OmniBridge::finalized_vote_nonce(ChainType::Ethereum(0)), 6);

		assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 4), None);
		assert_eq!(OmniBridge::pay_out_votes(hash4), None);
		assert_eq!(OmniBridge::pay_out_requests(hash4), None);
		assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 5), None);
		assert_eq!(OmniBridge::pay_out_votes(hash5), None);
		assert_eq!(OmniBridge::pay_out_requests(hash5), None);
		assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 6), None);
		assert_eq!(OmniBridge::pay_out_votes(hash6), None);
		assert_eq!(OmniBridge::pay_out_requests(hash6), None);

		assert_eq!(Balances::free_balance(alice()), 100);
	});
}

#[test]
fn pay_out_with_threshold_2_works() {
	new_test_ext(true).execute_with(|| {
		assert_ok!(OmniBridge::set_relayer_threshold(RuntimeOrigin::root(), 2));
		assert_ok!(OmniBridge::request_pay_out(
			RuntimeOrigin::signed(alice()),
			native_pay_out_request(1),
			true,
		));
		let hash1 = native_pay_out_request(1).hash();
		assert_eq!(OmniBridge::finalized_pay_out_nonce(ChainType::Ethereum(0)), 0);
		assert_eq!(Balances::free_balance(alice()), 50);

		assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 1), Some(vec![hash1]));
		assert_eq!(
			OmniBridge::pay_out_votes(hash1),
			Some(PayOutVote { ayes: vec![alice()], nays: vec![], status: VoteStatus::Pending })
		);
		assert_eq!(OmniBridge::pay_out_requests(hash1), Some(native_pay_out_request(1)));
		System::assert_has_event(
			Event::PayOutVoted {
				who: alice(),
				source_chain: ChainType::Ethereum(0),
				nonce: 1,
				asset: NativeOrWithId::Native,
				dest_account: alice(),
				amount: 10,
				aye: true,
			}
			.into(),
		);

		assert_ok!(OmniBridge::request_pay_out(
			RuntimeOrigin::signed(bob()),
			native_pay_out_request(1),
			false,
		));
		assert_eq!(OmniBridge::finalized_pay_out_nonce(ChainType::Ethereum(0)), 0);
		assert_eq!(Balances::free_balance(alice()), 50);

		assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 1), Some(vec![hash1]));
		assert_eq!(
			OmniBridge::pay_out_votes(hash1),
			Some(PayOutVote {
				ayes: vec![alice()],
				nays: vec![bob()],
				status: VoteStatus::Pending,
			})
		);
		assert_eq!(OmniBridge::pay_out_requests(hash1), Some(native_pay_out_request(1)));
		System::assert_has_event(
			Event::PayOutVoted {
				who: bob(),
				source_chain: ChainType::Ethereum(0),
				nonce: 1,
				asset: NativeOrWithId::Native,
				dest_account: alice(),
				amount: 10,
				aye: false,
			}
			.into(),
		);

		assert_ok!(OmniBridge::request_pay_out(
			RuntimeOrigin::signed(charlie()),
			native_pay_out_request(1),
			true,
		));
		assert_eq!(OmniBridge::finalized_pay_out_nonce(ChainType::Ethereum(0)), 1);
		assert_eq!(Balances::free_balance(alice()), 60);

		assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 1), None);
		assert_eq!(OmniBridge::pay_out_votes(hash1), None);
		assert_eq!(OmniBridge::pay_out_requests(hash1), None);
		System::assert_has_event(
			Event::PayOutVoted {
				who: charlie(),
				source_chain: ChainType::Ethereum(0),
				nonce: 1,
				asset: NativeOrWithId::Native,
				dest_account: alice(),
				amount: 10,
				aye: true,
			}
			.into(),
		);
		System::assert_has_event(
			Event::PaidOut {
				source_chain: ChainType::Ethereum(0),
				nonce: 1,
				asset: NativeOrWithId::Native,
				dest_account: alice(),
				amount: 10,
			}
			.into(),
		);

		// alice requests with nonce 2, but a different request (with bob and charlie)
		assert_ok!(OmniBridge::request_pay_out(
			RuntimeOrigin::signed(alice()),
			asset_pay_out_request(2),
			true,
		));
		assert_ok!(OmniBridge::request_pay_out(
			RuntimeOrigin::signed(bob()),
			native_pay_out_request(2),
			true,
		));

		let hash2_asset = asset_pay_out_request(2).hash();
		let hash2_native = native_pay_out_request(2).hash();

		// both requests are recorded
		assert_eq!(
			OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 2),
			Some(vec![hash2_asset, hash2_native])
		);
		assert_eq!(
			OmniBridge::pay_out_votes(hash2_asset),
			Some(PayOutVote { ayes: vec![alice()], nays: vec![], status: VoteStatus::Pending })
		);
		assert_eq!(
			OmniBridge::pay_out_votes(hash2_native),
			Some(PayOutVote { ayes: vec![bob()], nays: vec![], status: VoteStatus::Pending })
		);
		assert_eq!(OmniBridge::pay_out_requests(hash2_asset), Some(asset_pay_out_request(2)));
		assert_eq!(OmniBridge::pay_out_requests(hash2_native), Some(native_pay_out_request(2)));

		assert_ok!(OmniBridge::request_pay_out(
			RuntimeOrigin::signed(charlie()),
			native_pay_out_request(2),
			true,
		));

		assert_eq!(OmniBridge::finalized_pay_out_nonce(ChainType::Ethereum(0)), 2); // updated to 2
		assert_eq!(OmniBridge::finalized_vote_nonce(ChainType::Ethereum(0)), 2);

		assert_eq!(Balances::free_balance(alice()), 70);
		assert_eq!(Assets::balance(TEST_ASSET, alice()), 100);

		// should be fully cleaned up
		assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 2), None);
		assert_eq!(OmniBridge::pay_out_votes(hash2_asset), None);
		assert_eq!(OmniBridge::pay_out_votes(hash2_native), None);
		assert_eq!(OmniBridge::pay_out_requests(hash2_asset), None);
		assert_eq!(OmniBridge::pay_out_requests(hash2_native), None);
	});
}

#[test]
fn pay_out_with_threshold_2_fail_fast_works() {
	new_test_ext(true).execute_with(|| {
		assert_ok!(OmniBridge::set_relayer_threshold(RuntimeOrigin::root(), 2));
		let hash1 = native_pay_out_request(1).hash();
		assert_ok!(OmniBridge::request_pay_out(
			RuntimeOrigin::signed(alice()),
			native_pay_out_request(1),
			false,
		));
		assert_eq!(OmniBridge::finalized_pay_out_nonce(ChainType::Ethereum(0)), 0);
		assert_eq!(Balances::free_balance(alice()), 50);

		assert_ok!(OmniBridge::request_pay_out(
			RuntimeOrigin::signed(bob()),
			native_pay_out_request(1),
			false,
		));
		assert_eq!(OmniBridge::finalized_pay_out_nonce(ChainType::Ethereum(0)), 1);
		assert_eq!(Balances::free_balance(alice()), 50);

		assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 1), None);
		assert_eq!(OmniBridge::pay_out_votes(hash1), None);
		assert_eq!(OmniBridge::pay_out_requests(hash1), None);

		System::assert_has_event(
			Event::PayOutRejected {
				source_chain: ChainType::Ethereum(0),
				nonce: 1,
				asset: NativeOrWithId::Native,
				dest_account: alice(),
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
		use std::collections::HashMap;

		let mut hashes = HashMap::new();
		for i in 1..4 {
			hashes.insert(i, native_pay_out_request(i).hash());
		}

		assert_ok!(OmniBridge::set_relayer_threshold(RuntimeOrigin::root(), 2));
		for i in 1..4 {
			assert_ok!(OmniBridge::request_pay_out(
				RuntimeOrigin::signed(alice()),
				native_pay_out_request(i),
				true,
			));
			assert_eq!(OmniBridge::finalized_pay_out_nonce(ChainType::Ethereum(0)), 0);
			assert_eq!(Balances::free_balance(alice()), 50);
			assert_eq!(
				OmniBridge::pay_out_votes(hashes[&i]),
				Some(PayOutVote { ayes: vec![alice()], nays: vec![], status: VoteStatus::Pending })
			);
			assert_eq!(OmniBridge::pay_out_requests(hashes[&i]), Some(native_pay_out_request(i)));
			assert_eq!(
				OmniBridge::pay_out_hashes(ChainType::Ethereum(0), i),
				Some(vec![hashes[&i]])
			);
		}

		assert_ok!(OmniBridge::request_pay_out(
			RuntimeOrigin::signed(charlie()),
			native_pay_out_request(1),
			false,
		));
		assert_eq!(
			OmniBridge::pay_out_votes(hashes[&1]),
			Some(PayOutVote {
				ayes: vec![alice()],
				nays: vec![charlie()],
				status: VoteStatus::Pending,
			})
		);

		for i in 2..4 {
			assert_ok!(OmniBridge::request_pay_out(
				RuntimeOrigin::signed(charlie()),
				native_pay_out_request(i),
				true,
			));
			assert_eq!(OmniBridge::finalized_pay_out_nonce(ChainType::Ethereum(0)), 0); // not updated
																			   // request with nonce 2 and 3 should be passed and executed
			assert_eq!(
				OmniBridge::pay_out_votes(hashes[&i]),
				Some(PayOutVote {
					ayes: vec![alice(), charlie()],
					nays: vec![],
					status: VoteStatus::Passed,
				})
			);
			assert_eq!(
				OmniBridge::pay_out_hashes(ChainType::Ethereum(0), i),
				Some(vec![hashes[&i]])
			);
			assert_eq!(OmniBridge::finalized_vote_nonce(ChainType::Ethereum(0)), i);
			System::assert_has_event(
				Event::FinalizedVoteNonceUpdated { source_chain: ChainType::Ethereum(0), nonce: i }
					.into(),
			);
		}
		assert_eq!(
			OmniBridge::pay_out_votes(hashes[&1]),
			Some(PayOutVote {
				ayes: vec![alice()],
				nays: vec![charlie()],
				status: VoteStatus::Pending,
			})
		);
		assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), 1), Some(vec![hashes[&1]]));

		assert_eq!(OmniBridge::finalized_vote_nonce(ChainType::Ethereum(0)), 3);
		assert_eq!(Balances::free_balance(alice()), 70);

		assert_ok!(OmniBridge::request_pay_out(
			RuntimeOrigin::signed(bob()),
			native_pay_out_request(1),
			false,
		));
		assert_eq!(Balances::free_balance(alice()), 70);
		assert_eq!(OmniBridge::finalized_vote_nonce(ChainType::Ethereum(0)), 3);
		assert_eq!(OmniBridge::finalized_pay_out_nonce(ChainType::Ethereum(0)), 3); // updated to 3

		// should be fully cleaned up
		for i in 1..4 {
			assert_eq!(OmniBridge::pay_out_hashes(ChainType::Ethereum(0), i), None);
			assert_eq!(OmniBridge::pay_out_votes(hashes[&i]), None);
			assert_eq!(OmniBridge::pay_out_requests(hashes[&i]), None);
		}
		System::assert_has_event(
			Event::FinalizedPayOutNonceUpdated { source_chain: ChainType::Ethereum(0), nonce: 3 }
				.into(),
		);
		System::assert_has_event(
			Event::PayOutRejected {
				source_chain: ChainType::Ethereum(0),
				nonce: 1,
				asset: NativeOrWithId::Native,
				dest_account: alice(),
				amount: 10,
			}
			.into(),
		);
	});
}
