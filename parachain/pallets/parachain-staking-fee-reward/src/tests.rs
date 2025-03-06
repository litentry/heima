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

use crate::mock::{roll_to, Balances, ExtBuilder, ParachainStakingFeeReward};
use pallet_parachain_staking::TransactionFeeRewardResource;

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
			// Reward did recorded
			assert_eq!(ParachainStakingFeeReward::round_accumulated_reward(3), 30);

			roll_to(29);
			// Reward distributed
			// DefaultCollatorCommission = 20%
			// DefaultParachainBondReservePercent = 30%
			assert_eq!(Balances::free_balance(1), 75 + 30);
			assert_eq!(Balances::free_balance(3), 70 + 30);
			// Reward still recorded
			assert_eq!(ParachainStakingFeeReward::round_accumulated_reward(3), 30);
			roll_to(31);
			// Reward removed
			assert_eq!(ParachainStakingFeeReward::round_accumulated_reward(3), 0);
		});
}

#[test]
fn query_reward() {
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

			assert_eq!(
				ParachainStakingFeeReward::round_accumulated_reward(3),
				ParachainStakingFeeReward::query_round_fee_reward(3)
			);
			// 5 block per round, 2 round delay
			roll_to(21);
			ParachainStakingFeeReward::on_transaction_fee_reward_notify(15);
			assert_eq!(
				ParachainStakingFeeReward::round_accumulated_reward(5),
				ParachainStakingFeeReward::query_round_fee_reward(5)
			);
		});
}
