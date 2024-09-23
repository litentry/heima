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

#![allow(dead_code, unused_imports)]

use crate::{mock::*, Error, Event, PoolState, RoundInfo, RoundSetting, ScorePayment, Scores};
use core_primitives::{Identity, DAYS, YEARS};
use frame_support::{assert_err, assert_noop, assert_ok};
use pallet_parachain_staking::{Delegator, OnAllDelegationRemoved};
use pallet_teebag::{Enclave, WorkerType};
use sp_runtime::Perbill;
use sp_std::vec;

fn round_reward() -> Balance {
	(Perbill::from_perthousand(5) * 100_000_000 * UNIT / (YEARS as u128)) * 5
}

fn calculate_round_reward(
	user_score: u128,
	total_score: u128,
	user_stake: Balance,
	total_stake: Balance,
) -> Balance {
	round_reward()
		.saturating_mul(user_score)
		.saturating_div(total_score)
		.saturating_mul(num_integer::Roots::sqrt(&user_stake))
		.saturating_div(num_integer::Roots::sqrt(&total_stake))
}

#[test]
fn default_state_works() {
	new_test_ext(false).execute_with(|| {
		assert_eq!(ScoreStaking::state(), PoolState::Stopped);
		assert_eq!(
			ScoreStaking::round_config(),
			RoundSetting { interval: 7 * DAYS, stake_coef_n: 1, stake_coef_m: 2 }
		);
		assert_eq!(ScoreStaking::score_feeder().unwrap(), alice());
		assert_eq!(ScoreStaking::round(), RoundInfo { index: 0, start_block: 0 });
	})
}

#[test]
fn start_stop_pool_works() {
	new_test_ext(true).execute_with(|| {
		run_to_block(2);
		assert_ok!(ScoreStaking::start_pool(RuntimeOrigin::root()));
		assert_eq!(ScoreStaking::state(), PoolState::Running);
		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::<Test>::PoolStarted {
			start_block: 2,
		}));
		assert_eq!(ScoreStaking::round(), RoundInfo { index: 1, start_block: 2 });

		run_to_block(6);
		assert_eq!(ScoreStaking::round(), RoundInfo { index: 1, start_block: 2 });

		run_to_block(7);
		assert_eq!(ScoreStaking::round(), RoundInfo { index: 2, start_block: 7 });

		run_to_block(8);
		assert_ok!(ScoreStaking::stop_pool(RuntimeOrigin::root()));
		assert_eq!(ScoreStaking::state(), PoolState::Stopped);
		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::<Test>::PoolStopped {}));
		assert_eq!(ScoreStaking::round(), RoundInfo { index: 2, start_block: 7 });

		run_to_block(20);
		assert_ok!(ScoreStaking::start_pool(RuntimeOrigin::root()));
		assert_eq!(ScoreStaking::state(), PoolState::Running);
		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::<Test>::PoolStarted {
			start_block: 20,
		}));
		assert_eq!(ScoreStaking::round(), RoundInfo { index: 3, start_block: 20 });
	})
}

#[test]
fn set_round_config_works() {
	new_test_ext(true).execute_with(|| {
		run_to_block(2);
		assert_ok!(ScoreStaking::start_pool(RuntimeOrigin::root()));
		assert_eq!(ScoreStaking::round(), RoundInfo { index: 1, start_block: 2 });

		run_to_block(3);
		assert_ok!(ScoreStaking::set_round_config(
			RuntimeOrigin::root(),
			RoundSetting { interval: 3, stake_coef_n: 1, stake_coef_m: 2 }
		));

		assert_eq!(ScoreStaking::round(), RoundInfo { index: 1, start_block: 2 });
		assert_eq!(ScoreStaking::round_config().interval, 3);

		run_to_block(5);
		assert_eq!(ScoreStaking::round(), RoundInfo { index: 2, start_block: 5 });

		run_to_block(6);
		assert_ok!(ScoreStaking::set_round_config(
			RuntimeOrigin::root(),
			RoundSetting { interval: 5, stake_coef_n: 1, stake_coef_m: 2 }
		));
		assert_eq!(ScoreStaking::round(), RoundInfo { index: 2, start_block: 5 });

		run_to_block(9);
		assert_eq!(ScoreStaking::round(), RoundInfo { index: 2, start_block: 5 });

		run_to_block(10);
		assert_eq!(ScoreStaking::round(), RoundInfo { index: 3, start_block: 10 });
	});
}

#[test]
fn default_mint_works() {
	new_test_ext(true).execute_with(|| {
		run_to_block(2);
		assert_ok!(ScoreStaking::start_pool(RuntimeOrigin::root()));

		// run to next reward distribution round
		run_to_block(7);
		System::assert_last_event(RuntimeEvent::ScoreStaking(
			Event::<Test>::RewardDistributionStarted { round_index: 2 },
		));
	});
}

#[test]
fn score_update_checks_staking() {
	new_test_ext(true).execute_with(|| {
		run_to_block(2);
		assert_ok!(ScoreStaking::start_pool(RuntimeOrigin::root()));

		run_to_block(3);
		assert_err!(
			ScoreStaking::update_score(RuntimeOrigin::signed(alice()), alice().into(), 2000),
			Error::<Test>::UserNotStaked
		);
	});
}

#[test]
#[allow(clippy::identity_op)]
fn score_staking_works() {
	new_test_ext_with_parachain_staking().execute_with(|| {
		let enclave = Enclave::new(WorkerType::Identity);
		pallet_teebag::EnclaveRegistry::<Test>::insert(alice(), enclave);

		run_to_block(2);
		assert_ok!(ScoreStaking::start_pool(RuntimeOrigin::root()));

		let alice_staking = 900;
		let mut alice_score = 500;

		run_to_block(3);
		pallet_parachain_staking::DelegatorState::<Test>::insert(
			alice(),
			Delegator::new(bob(), bob(), alice_staking),
		);
		pallet_parachain_staking::Total::<Test>::put(alice_staking);

		assert_ok!(ScoreStaking::update_score(
			RuntimeOrigin::signed(alice()),
			alice().into(),
			alice_score
		));
		assert_eq!(
			ScoreStaking::scores(alice()).unwrap(),
			ScorePayment {
				score: alice_score,
				total_reward: 0,
				last_round_reward: 0,
				unpaid_reward: 0,
				total_staking_amount: 0
			}
		);

		assert_eq!(ScoreStaking::total_score(), alice_score);
		assert_eq!(ScoreStaking::score_user_count(), 1);

		alice_score = 2000;

		assert_ok!(ScoreStaking::update_score(
			RuntimeOrigin::signed(alice()),
			alice().into(),
			alice_score
		));
		assert_eq!(ScoreStaking::scores(alice()).unwrap().score, alice_score);
		assert_eq!(ScoreStaking::total_score(), alice_score);
		assert_eq!(ScoreStaking::score_user_count(), 1);

		// run to next reward distribution round, alice should win all rewards
		run_to_block(7);
		System::assert_last_event(RuntimeEvent::ScoreStaking(
			Event::<Test>::RewardDistributionStarted { round_index: 2 },
		));
		// total reward first distribution
		let mut alice_total_reward = 0;
		let total_staking = pallet_parachain_staking::Total::<Test>::get();
		let total_score = ScoreStaking::total_score();
		let round_reward = calculate_round_reward(
			alice_score.into(),
			total_score.into(),
			alice_staking,
			total_staking,
		);
		alice_total_reward += round_reward;

		assert_ok!(ScoreStaking::update_total_staking_amount(
			RuntimeOrigin::signed(alice()),
			alice(),
			alice_staking
		));
		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::TotalStakingAmountUpdated {
			account_id: alice(),
			amount: alice_staking,
		}));

		assert_ok!(ScoreStaking::distribute_rewards(RuntimeOrigin::signed(alice()), 2));
		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::RewardDistributionCompleted {
			round_index: 2,
		}));

		assert_eq!(
			ScoreStaking::scores(alice()).unwrap(),
			ScorePayment {
				score: alice_score,
				total_reward: alice_total_reward,
				last_round_reward: round_reward,
				unpaid_reward: alice_total_reward,
				total_staking_amount: alice_staking
			}
		);

		// alice's winning should accumulate
		run_to_block(12);
		System::assert_last_event(RuntimeEvent::ScoreStaking(
			Event::<Test>::RewardDistributionStarted { round_index: 3 },
		));
		// total reward second distribution
		let total_staking = pallet_parachain_staking::Total::<Test>::get();
		let total_score = ScoreStaking::total_score();
		let round_reward = calculate_round_reward(
			alice_score.into(),
			total_score.into(),
			alice_staking,
			total_staking,
		);
		alice_total_reward += round_reward;

		assert_ok!(ScoreStaking::distribute_rewards(RuntimeOrigin::signed(alice()), 3));
		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::RewardDistributionCompleted {
			round_index: 3,
		}));

		assert_eq!(
			ScoreStaking::scores(alice()).unwrap(),
			ScorePayment {
				score: alice_score,
				total_reward: alice_total_reward,
				last_round_reward: round_reward,
				unpaid_reward: alice_total_reward,
				total_staking_amount: alice_staking
			}
		);

		let other_staking = 1000;

		// increase the total staked amount, alice should win less
		run_to_block(13);
		pallet_parachain_staking::Total::<Test>::put(alice_staking + other_staking);

		run_to_block(17);
		System::assert_last_event(RuntimeEvent::ScoreStaking(
			Event::<Test>::RewardDistributionStarted { round_index: 4 },
		));

		assert_ok!(ScoreStaking::distribute_rewards(RuntimeOrigin::signed(alice()), 4));
		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::RewardDistributionCompleted {
			round_index: 4,
		}));

		let total_staking = pallet_parachain_staking::Total::<Test>::get();
		let total_score = ScoreStaking::total_score();
		let round_reward = calculate_round_reward(
			alice_score.into(),
			total_score.into(),
			alice_staking,
			total_staking,
		);
		// total reward third distribution
		alice_total_reward += round_reward;

		assert_eq!(
			ScoreStaking::scores(alice()).unwrap(),
			ScorePayment {
				score: alice_score,
				total_reward: alice_total_reward,
				last_round_reward: round_reward,
				unpaid_reward: alice_total_reward,
				total_staking_amount: alice_staking
			}
		);

		run_to_block(18);

		// add bob's score
		let mut bob_staking = 1600;
		let mut bob_score = 1000;

		assert_ok!(ParachainStaking::delegate(RuntimeOrigin::signed(bob()), alice(), bob_staking));
		assert_eq!(
			pallet_parachain_staking::Total::<Test>::get(),
			alice_staking + bob_staking + other_staking
		);
		assert_ok!(ScoreStaking::update_score(
			RuntimeOrigin::signed(alice()),
			bob().into(),
			bob_score
		));
		assert_eq!(ScoreStaking::total_score(), alice_score + bob_score);
		assert_eq!(ScoreStaking::score_user_count(), 2);

		run_to_block(22);
		System::assert_last_event(RuntimeEvent::ScoreStaking(
			Event::<Test>::RewardDistributionStarted { round_index: 5 },
		));
		assert_ok!(ScoreStaking::update_total_staking_amount(
			RuntimeOrigin::signed(alice()),
			bob(),
			bob_staking
		));
		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::TotalStakingAmountUpdated {
			account_id: bob(),
			amount: bob_staking,
		}));

		assert_ok!(ScoreStaking::distribute_rewards(RuntimeOrigin::signed(alice()), 5));
		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::RewardDistributionCompleted {
			round_index: 5,
		}));

		// total rewards fourth distribution
		let total_staking = pallet_parachain_staking::Total::<Test>::get();
		let total_score = ScoreStaking::total_score();

		let alice_round_reward = calculate_round_reward(
			alice_score.into(),
			total_score.into(),
			alice_staking,
			total_staking,
		);
		alice_total_reward += alice_round_reward;

		let mut bob_total_reward = 0;
		let bob_round_reward = calculate_round_reward(
			bob_score.into(),
			total_score.into(),
			bob_staking,
			total_staking,
		);
		bob_total_reward += bob_round_reward;

		assert_eq!(
			ScoreStaking::scores(alice()).unwrap(),
			ScorePayment {
				score: alice_score,
				total_reward: alice_total_reward,
				last_round_reward: alice_round_reward,
				unpaid_reward: alice_total_reward,
				total_staking_amount: alice_staking
			}
		);
		assert_eq!(
			ScoreStaking::scores(bob()).unwrap(),
			ScorePayment {
				score: bob_score,
				total_reward: bob_total_reward,
				last_round_reward: bob_round_reward,
				unpaid_reward: bob_total_reward,
				total_staking_amount: bob_staking
			}
		);

		// update more scores will error out
		pallet_parachain_staking::DelegatorState::<Test>::insert(
			charlie(),
			Delegator::new(alice(), alice(), 7000),
		);
		assert_err!(
			ScoreStaking::update_score(RuntimeOrigin::signed(alice()), charlie().into(), 1000),
			Error::<Test>::MaxScoreUserCountReached
		);

		run_to_block(23);

		assert_ok!(ParachainStaking::schedule_revoke_delegation(
			RuntimeOrigin::signed(bob()),
			alice()
		));

		run_to_block(27);

		System::assert_last_event(RuntimeEvent::ScoreStaking(
			Event::<Test>::RewardDistributionStarted { round_index: 6 },
		));
		assert_ok!(ScoreStaking::distribute_rewards(RuntimeOrigin::signed(alice()), 6));
		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::RewardDistributionCompleted {
			round_index: 6,
		}));

		// total rewards fifth distribution
		let total_staking = pallet_parachain_staking::Total::<Test>::get();
		let total_score = ScoreStaking::total_score();

		let alice_round_reward = calculate_round_reward(
			alice_score.into(),
			total_score.into(),
			alice_staking,
			total_staking,
		);
		alice_total_reward += alice_round_reward;

		let bob_round_reward = calculate_round_reward(
			bob_score.into(),
			total_score.into(),
			bob_staking,
			total_staking,
		);
		bob_total_reward += bob_round_reward;

		assert_eq!(
			ScoreStaking::scores(alice()).unwrap(),
			ScorePayment {
				score: alice_score,
				total_reward: alice_total_reward,
				last_round_reward: alice_round_reward,
				unpaid_reward: alice_total_reward,
				total_staking_amount: alice_staking
			}
		);
		assert_eq!(
			ScoreStaking::scores(bob()).unwrap(),
			ScorePayment {
				score: bob_score,
				total_reward: bob_total_reward,
				last_round_reward: bob_round_reward,
				unpaid_reward: bob_total_reward,
				total_staking_amount: bob_staking
			}
		);

		run_to_block(30);
		assert_ok!(ParachainStaking::execute_delegation_request(
			RuntimeOrigin::signed(bob()),
			bob(),
			alice()
		));
		bob_staking = 0;
		bob_score = 0;

		run_to_block(33);

		System::assert_last_event(RuntimeEvent::ScoreStaking(
			Event::<Test>::RewardDistributionStarted { round_index: 7 },
		));
		assert_ok!(ScoreStaking::distribute_rewards(RuntimeOrigin::signed(alice()), 7));
		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::RewardDistributionCompleted {
			round_index: 7,
		}));

		// total reward sixth distribution
		let total_staking = pallet_parachain_staking::Total::<Test>::get();
		let total_score = ScoreStaking::total_score();

		let alice_round_reward = calculate_round_reward(
			alice_score.into(),
			total_score.into(),
			alice_staking,
			total_staking,
		);
		alice_total_reward += alice_round_reward;

		let bob_round_reward = calculate_round_reward(
			bob_score.into(),
			total_score.into(),
			bob_staking,
			total_staking,
		);
		bob_total_reward += bob_round_reward;

		assert_eq!(
			ScoreStaking::scores(alice()).unwrap(),
			ScorePayment {
				score: alice_score,
				total_reward: alice_total_reward,
				last_round_reward: alice_round_reward,
				unpaid_reward: alice_total_reward,
				total_staking_amount: alice_staking
			}
		);
		assert_eq!(
			ScoreStaking::scores(bob()).unwrap(),
			ScorePayment {
				score: bob_score,
				total_reward: bob_total_reward,
				last_round_reward: bob_round_reward,
				unpaid_reward: bob_total_reward,
				total_staking_amount: bob_staking
			}
		);

		// remove increased stake (keep only alice's stake)
		pallet_parachain_staking::Total::<Test>::put(alice_staking);

		run_to_block(37);

		System::assert_last_event(RuntimeEvent::ScoreStaking(
			Event::<Test>::RewardDistributionStarted { round_index: 8 },
		));
		assert_ok!(ScoreStaking::distribute_rewards(RuntimeOrigin::signed(alice()), 8));
		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::RewardDistributionCompleted {
			round_index: 8,
		}));

		// total reward sixth distribution
		let total_staking = pallet_parachain_staking::Total::<Test>::get();
		let total_score = ScoreStaking::total_score();

		let alice_round_reward = calculate_round_reward(
			alice_score.into(),
			total_score.into(),
			alice_staking,
			total_staking,
		);
		alice_total_reward += alice_round_reward;

		assert_eq!(
			ScoreStaking::scores(alice()).unwrap(),
			ScorePayment {
				score: alice_score,
				total_reward: alice_total_reward,
				last_round_reward: alice_round_reward,
				unpaid_reward: alice_total_reward,
				total_staking_amount: alice_staking
			}
		);
		// bob should not participate in the reward calculation
		assert_eq!(
			ScoreStaking::scores(bob()).unwrap(),
			ScorePayment {
				score: 0, // bob's score should be cleared
				total_reward: bob_total_reward,
				last_round_reward: 0,
				unpaid_reward: bob_total_reward,
				total_staking_amount: bob_staking
			}
		);
		assert_eq!(ScoreStaking::total_score(), alice_score);
		assert_eq!(ScoreStaking::score_user_count(), 2); // entry is not yet removed

		// remove_score works
		assert_ok!(ScoreStaking::remove_score(RuntimeOrigin::signed(alice()), bob().into()));
		assert_eq!(ScoreStaking::total_score(), alice_score);
		assert_eq!(ScoreStaking::score_user_count(), 1);
	});
}

#[test]
fn claim_works() {
	new_test_ext(true).execute_with(|| {
		let enclave = Enclave::new(WorkerType::Identity);
		pallet_teebag::EnclaveRegistry::<Test>::insert(alice(), enclave);

		run_to_block(2);
		assert_ok!(ScoreStaking::start_pool(RuntimeOrigin::root()));

		run_to_block(3);
		let alice_staking = 1000;
		pallet_parachain_staking::DelegatorState::<Test>::insert(
			alice(),
			Delegator::new(bob(), bob(), alice_staking),
		);
		pallet_parachain_staking::Total::<Test>::put(alice_staking);

		assert_ok!(ScoreStaking::update_score(
			RuntimeOrigin::signed(alice()),
			alice().into(),
			2000
		));

		// run to next reward distribution round, alice should win all rewards
		run_to_block(7);
		System::assert_last_event(RuntimeEvent::ScoreStaking(
			Event::<Test>::RewardDistributionStarted { round_index: 2 },
		));
		assert_eq!(
			ScoreStaking::scores(alice()).unwrap(),
			ScorePayment {
				score: 2000,
				total_reward: 0,
				last_round_reward: 0,
				unpaid_reward: 0,
				total_staking_amount: 0
			}
		);
		assert_ok!(ScoreStaking::update_total_staking_amount(
			RuntimeOrigin::signed(alice()),
			alice(),
			alice_staking
		));
		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::TotalStakingAmountUpdated {
			account_id: alice(),
			amount: alice_staking,
		}));

		assert_ok!(ScoreStaking::distribute_rewards(RuntimeOrigin::signed(alice()), 2));
		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::RewardDistributionCompleted {
			round_index: 2,
		}));
		assert_eq!(
			ScoreStaking::scores(alice()).unwrap(),
			ScorePayment {
				score: 2000,
				total_reward: round_reward(),
				last_round_reward: round_reward(),
				unpaid_reward: round_reward(),
				total_staking_amount: alice_staking
			}
		);

		assert_ok!(ScoreStaking::claim(RuntimeOrigin::signed(alice()), 200));
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
				total_staking_amount: alice_staking
			}
		);

		assert_ok!(ScoreStaking::claim_all(RuntimeOrigin::signed(alice())));
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
				total_staking_amount: alice_staking
			}
		);

		// continue to claim will error out
		assert_err!(
			ScoreStaking::claim(RuntimeOrigin::signed(alice()), 100),
			Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn distribute_rewards_works() {
	new_test_ext_with_parachain_staking().execute_with(|| {
		let enclave = Enclave::new(WorkerType::Identity);
		pallet_teebag::EnclaveRegistry::<Test>::insert(alice(), enclave);

		run_to_block(2);
		assert_ok!(ScoreStaking::start_pool(RuntimeOrigin::root()));

		let mut alice_staking = 900;
		let alice_score = 500;

		run_to_block(3);
		pallet_parachain_staking::DelegatorState::<Test>::insert(
			alice(),
			Delegator::new(bob(), bob(), alice_staking),
		);
		pallet_parachain_staking::Total::<Test>::put(alice_staking);
		assert_ok!(ScoreStaking::update_score(
			RuntimeOrigin::signed(alice()),
			alice().into(),
			alice_score
		));

		// run to next reward distribution round, alice should win all rewards
		run_to_block(7);

		System::assert_last_event(RuntimeEvent::ScoreStaking(
			Event::<Test>::RewardDistributionStarted { round_index: 2 },
		));

		let alice_id_graph_staking = 1900;

		assert_ok!(ScoreStaking::update_total_staking_amount(
			RuntimeOrigin::signed(alice()),
			alice(),
			alice_id_graph_staking
		));
		alice_staking = alice_id_graph_staking;
		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::TotalStakingAmountUpdated {
			account_id: alice(),
			amount: alice_staking,
		}));

		assert_ok!(ScoreStaking::distribute_rewards(RuntimeOrigin::signed(alice()), 2));
		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::RewardDistributionCompleted {
			round_index: 2,
		}));

		let mut alice_total_reward = 0;
		let total_staking = pallet_parachain_staking::Total::<Test>::get();
		let total_score = ScoreStaking::total_score();
		let round_reward = calculate_round_reward(
			alice_score.into(),
			total_score.into(),
			alice_staking,
			total_staking,
		);
		alice_total_reward += round_reward;

		assert_eq!(
			ScoreStaking::scores(alice()).unwrap(),
			ScorePayment {
				score: alice_score,
				total_reward: alice_total_reward,
				last_round_reward: round_reward,
				unpaid_reward: alice_total_reward,
				total_staking_amount: alice_staking
			}
		);

		run_to_block(12);

		System::assert_last_event(RuntimeEvent::ScoreStaking(
			Event::<Test>::RewardDistributionStarted { round_index: 3 },
		));

		assert_ok!(ScoreStaking::distribute_rewards(RuntimeOrigin::signed(alice()), 3));
		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::RewardDistributionCompleted {
			round_index: 3,
		}));

		let total_staking = pallet_parachain_staking::Total::<Test>::get();
		let total_score = ScoreStaking::total_score();
		let round_reward = calculate_round_reward(
			alice_score.into(),
			total_score.into(),
			alice_staking,
			total_staking,
		);
		alice_total_reward += round_reward;

		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::RewardDistributionCompleted {
			round_index: 3,
		}));

		assert_eq!(
			ScoreStaking::scores(alice()).unwrap(),
			ScorePayment {
				score: alice_score,
				total_reward: alice_total_reward,
				last_round_reward: round_reward,
				unpaid_reward: alice_total_reward,
				total_staking_amount: alice_staking
			}
		);
	})
}

#[test]
fn distribute_rewards_round_rewards_already_distributed_works() {
	new_test_ext_with_parachain_staking().execute_with(|| {
		let enclave = Enclave::new(WorkerType::Identity);
		pallet_teebag::EnclaveRegistry::<Test>::insert(alice(), enclave);

		run_to_block(2);
		assert_ok!(ScoreStaking::start_pool(RuntimeOrigin::root()));

		let alice_staking = 900;
		let alice_score = 500;

		run_to_block(3);
		pallet_parachain_staking::DelegatorState::<Test>::insert(
			alice(),
			Delegator::new(bob(), bob(), alice_staking),
		);
		pallet_parachain_staking::Total::<Test>::put(alice_staking);
		assert_ok!(ScoreStaking::update_score(
			RuntimeOrigin::signed(alice()),
			alice().into(),
			alice_score
		));

		// run to next reward distribution round, alice should win all rewards
		run_to_block(7);

		System::assert_last_event(RuntimeEvent::ScoreStaking(
			Event::<Test>::RewardDistributionStarted { round_index: 2 },
		));
		assert_ok!(ScoreStaking::distribute_rewards(RuntimeOrigin::signed(alice()), 2));
		System::assert_last_event(RuntimeEvent::ScoreStaking(Event::RewardDistributionCompleted {
			round_index: 2,
		}));

		assert_noop!(
			ScoreStaking::distribute_rewards(RuntimeOrigin::signed(alice()), 2),
			Error::<Test>::RoundRewardsAlreadyDistributed
		);
	})
}

#[test]
fn distribute_rewards_origin_check_works() {
	new_test_ext(false).execute_with(|| {
		assert_noop!(
			ScoreStaking::distribute_rewards(RuntimeOrigin::signed(bob()), 1),
			sp_runtime::DispatchError::BadOrigin
		);
	})
}

#[test]
fn on_all_delegation_removed_works() {
	new_test_ext(true).execute_with(|| {
		let bob = bob();
		pallet_parachain_staking::DelegatorState::<Test>::insert(
			bob.clone(),
			Delegator::new(alice(), alice(), 1600),
		);
		assert_ok!(ScoreStaking::update_score(
			RuntimeOrigin::signed(alice()),
			Identity::from(bob.clone()),
			1000
		));

		assert_eq!(ScoreStaking::total_score(), 1000);
		assert_eq!(Scores::<Test>::get(&bob).unwrap().score, 1000);

		assert_ok!(ScoreStaking::on_all_delegation_removed(&bob));
		assert_eq!(ScoreStaking::total_score(), 0);
		assert_eq!(Scores::<Test>::get(&bob).unwrap().score, 0);
	});
}
