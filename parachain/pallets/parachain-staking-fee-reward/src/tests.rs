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

use crate::{
	assert_eq_events, assert_eq_last_events, assert_event_emitted, assert_last_event,
	assert_tail_eq,
	mock::{
		roll_one_block, roll_to, roll_to_round_begin, roll_to_round_end, AccountId, Balances,
		ExtBuilder, ParachainStaking, ParachainStakingFeeReward, RuntimeEvent as MetaEvent,
		RuntimeOrigin, Test,
	},
	AtStake, Bond, CollatorStatus, DelegationScheduledRequests, DelegatorAdded, Error, Event,
	Range,
};
use frame_support::{
	assert_noop, assert_ok,
	traits::{LockIdentifier, LockableCurrency, WithdrawReasons},
};
use pallet_parachain_staking::TransactionFeeRewardResource;
use sp_runtime::{traits::Zero, DispatchError, ModuleError, Perbill, Percent};

#[test]
fn reward_distributed() {
	ExtBuilder::default()
		.with_balances(vec![(1, 100), (2, 100), (3, 100), (4, 100)])
		.with_candidates(vec![(1, 25)])
		.with_delegations(vec![(3, 1, 30)])
		.build()
		.execute_with(|| {
			// 5 block per round, 2 round delay
			roll_to(11);
			ParachainStakingFeeReward::on_transaction_fee_reward_notify(10);
			roll_to(12);
			ParachainStakingFeeReward::on_transaction_fee_reward_notify(20);
			roll_to(15);
			// No reward distributed yet
			assert_eq!(Balances::free_balance(1), 75);
			assert_eq!(Balances::free_balance(3), 70);
		});
}
