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
use core::str::from_utf8;
use frame_support::{assert_err, assert_ok};
use heima_primitives::YEARS;
use pallet_parachain_staking::Delegator;
use pallet_score_staking::{Error, Event, ScorePayment};
use precompile_utils::testing::*;
use sp_runtime::Perbill;

fn round_reward() -> Balance {
	(Perbill::from_perthousand(5) * 100_000_000 * UNIT / (YEARS as u128)) * 5
}

fn precompiles() -> ScoreStakingMockPrecompile<Test> {
	PrecompilesValue::get()
}

#[test]
fn claim_is_ok() {
	new_test_ext(true).execute_with(|| {
		run_to_block(2);
		assert_ok!(ScoreStaking::start_pool(RuntimeOrigin::root()));

		run_to_block(3);
		pallet_parachain_staking::DelegatorState::<Test>::insert(
			alice(),
			Delegator::new(bob(), bob(), 1000),
		);
		pallet_parachain_staking::Total::<Test>::put(1000);

		assert_ok!(ScoreStaking::update_score(
			RuntimeOrigin::signed(alice()),
			alice().into(),
			2000
		));

		// run to next reward distribution round, alice should win all rewards
		run_to_block(7);
		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::<Test>::RewardCalculated {
			total: round_reward(),
			distributed: round_reward(),
		}));
		assert_eq!(
			ScoreStaking::scores(alice()).unwrap(),
			ScorePayment {
				score: 2000,
				total_reward: round_reward(),
				last_round_reward: round_reward(),
				unpaid_reward: round_reward(),
			}
		);

		precompiles()
			.prepare_test(
				U8Wrapper(1u8), // alice
				precompile_address(),
				PCall::<Test>::claim { amount: 200u128.into() },
			)
			.expect_no_logs()
			.execute_returns(());

		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::<Test>::RewardClaimed {
			who: alice(),
			amount: 200,
		}));
		assert_eq!(
			ScoreStaking::scores(alice()).unwrap(),
			ScorePayment {
				score: 2000,
				total_reward: round_reward(),
				last_round_reward: round_reward(),
				unpaid_reward: round_reward() - 200,
			}
		);

		precompiles()
			.prepare_test(
				U8Wrapper(1u8), // alice
				precompile_address(),
				PCall::<Test>::claim_all {},
			)
			.expect_no_logs()
			.execute_returns(());

		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::<Test>::RewardClaimed {
			who: alice(),
			amount: round_reward() - 200,
		}));
		assert_eq!(
			ScoreStaking::scores(alice()).unwrap(),
			ScorePayment {
				score: 2000,
				total_reward: round_reward(),
				last_round_reward: round_reward(),
				unpaid_reward: 0,
			}
		);

		// continue to claim will error out
		assert_err!(
			ScoreStaking::claim(RuntimeOrigin::signed(alice()), 100),
			Error::<Test>::InsufficientBalance
		);

		precompiles()
			.prepare_test(
				U8Wrapper(1u8), // alice
				precompile_address(),
				PCall::<Test>::claim { amount: 100u128.into() },
			)
			.execute_reverts(|output| from_utf8(output).unwrap().contains("InsufficientBalance"));
	});
}
