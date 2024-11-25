use crate::mock::*;
use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency};
use pallet_balances::Error as BalanceError;
use sp_core::H256;
use sp_runtime::{AccountId32, ArithmeticError};

#[test]
fn test_propose_investing_pool_ok() {
	new_test_ext().execute_with(|| {
		let user_a: AccountId32 = AccountId32::from([1u8; 32]);
		let user_b: AccountId32 = AccountId32::from([2u8; 32]);
		Balances::make_free_balance_be(&user_a, 100_000_000_000_000_000_000);

		let max_pool_size = 1_000_000_000_000_000_000_000u128;
		let proposal_last_time = 100;
		let pool_last_time = 10000;
		let estimated_pool_reward = 2_000_000_000_000_000_000_000u128;
		let pool_info_hash: H256 = H256([2; 32]);
		let pool_info_hash_2: H256 = H256([3; 32]);

		// ProposalPublicTimeTooShort
		assert_noop!(
			PoolProposal::propose_investing_pool(
				RuntimeOrigin::signed(user_a.clone()),
				max_pool_size,
				1,
				pool_last_time,
				estimated_pool_reward,
				pool_info_hash
			),
			crate::Error::<Test>::ProposalPublicTimeTooShort
		);

		// No enough reserve token
		assert_err!(
			PoolProposal::propose_investing_pool(
				RuntimeOrigin::signed(user_b.clone()),
				max_pool_size,
				proposal_last_time,
				pool_last_time,
				estimated_pool_reward,
				pool_info_hash
			),
			BalanceError::<Test>::InsufficientBalance
		);

		// Worked
		assert_ok!(PoolProposal::propose_investing_pool(
			RuntimeOrigin::signed(user_a.clone()),
			max_pool_size,
			proposal_last_time,
			pool_last_time,
			estimated_pool_reward,
			pool_info_hash
		));

		assert_events(vec![RuntimeEvent::PoolProposal(crate::Event::PoolProposed {
			proposer: user_a.clone(),
			pool_proposal_index: 1u128,
			info_hash: pool_info_hash,
		})]);

		// Oversized
		assert_noop!(
			PoolProposal::propose_investing_pool(
				RuntimeOrigin::signed(user_a),
				max_pool_size,
				proposal_last_time,
				pool_last_time,
				estimated_pool_reward,
				pool_info_hash_2
			),
			crate::Error::<Test>::ProposalDepositDuplicatedOrOversized
		);
	})
}

#[test]
fn test_pre_stake_proposal_ok() {
	new_test_ext().execute_with(|| {
		let user_a: AccountId32 = AccountId32::from([1u8; 32]);
		let user_b: AccountId32 = AccountId32::from([2u8; 32]);
		Balances::make_free_balance_be(&user_a, 100_000_000_000_000_000_000);
		Balances::make_free_balance_be(&user_b, 100_000_000_000_000_000_000);

		let max_pool_size = 1_000_000_000_000_000_000_000u128;
		let proposal_last_time = 100;
		let pool_last_time = 10000;
		let estimated_pool_reward = 2_000_000_000_000_000_000_000u128;
		let pool_info_hash: H256 = H256([2; 32]);

		// Setup
		assert_ok!(PoolProposal::propose_investing_pool(
			RuntimeOrigin::signed(user_a.clone()),
			max_pool_size,
			proposal_last_time,
			pool_last_time,
			estimated_pool_reward,
			pool_info_hash
		));

		// Not enough token
		assert_noop!(
			PoolProposal::pre_stake_proposal(
				RuntimeOrigin::signed(user_a.clone()),
				1u128,
				2_000_000_000_000_000_000_000_000,
			),
			ArithmeticError::Underflow
		);

		// Pool not exist
		assert_noop!(
			PoolProposal::pre_stake_proposal(
				RuntimeOrigin::signed(user_a.clone()),
				2u128,
				500_000_000_000_000_000_000u128,
			),
			crate::Error::<Test>::ProposalNotExist
		);

		// Normal pre staking worked
		assert_ok!(PoolProposal::pre_stake_proposal(
			RuntimeOrigin::signed(user_a.clone()),
			1u128,
			500_000_000_000_000_000_000u128,
		));

		assert_events(vec![RuntimeEvent::PoolProposal(crate::Event::PoolPreInvested {
			user: user_a.clone(),
			pool_proposal_index: 1u128,
			amount: 500_000_000_000_000_000_000u128,
		})]);

		// Go to proposal expire time
		roll_to(150);
		// Proposal Expired, Not work
		assert_noop!(
			PoolProposal::pre_stake_proposal(
				RuntimeOrigin::signed(user_a),
				1u128,
				500_000_000_000_000_000_000u128,
			),
			crate::Error::<Test>::ProposalExpired
		);
	})
}

#[test]
fn test_withdraw_pre_investing_ok() {
	new_test_ext().execute_with(|| {
		let user_a: AccountId32 = AccountId32::from([1u8; 32]);
		let user_b: AccountId32 = AccountId32::from([2u8; 32]);
		Balances::make_free_balance_be(&user_a, 100_000_000_000_000_000_000);
		Balances::make_free_balance_be(&user_b, 100_000_000_000_000_000_000);

		let max_pool_size = 10_000_000_000_000_000_000u128;
		let proposal_last_time = 100;
		let pool_last_time = 10000;
		let estimated_pool_reward = 2_000_000_000_000_000_000_000u128;
		let pool_info_hash: H256 = H256([2; 32]);

		// Setup
		assert_ok!(PoolProposal::propose_investing_pool(
			RuntimeOrigin::signed(user_a.clone()),
			max_pool_size,
			proposal_last_time,
			pool_last_time,
			estimated_pool_reward,
			pool_info_hash
		));

		// Normal pre staking worked
		assert_ok!(PoolProposal::pre_stake_proposal(
			RuntimeOrigin::signed(user_a.clone()),
			1u128,
			5_000_000_000_000_000_000u128,
		));

		// Can not withdraw if pool is fullfiled
		assert_noop!(
			PoolProposal::withdraw_pre_investing(
				RuntimeOrigin::signed(user_a.clone()),
				1u128,
				1_000_000_000_000_000_000u128,
			),
			crate::Error::<Test>::InsufficientPreInvesting
		);

		// Fullfil the pool
		assert_ok!(PoolProposal::pre_stake_proposal(
			RuntimeOrigin::signed(user_a.clone()),
			1u128,
			495_000_000_000_000_000_000u128,
		));

		// pre investing and queued
		assert_events(vec![RuntimeEvent::PoolProposal(crate::Event::PoolPreInvested {
			user: user_a.clone(),
			pool_proposal_index: 1u128,
			amount: 5_000_000_000_000_000_000u128,
		})]);
		assert_events(vec![RuntimeEvent::PoolProposal(crate::Event::PoolPreStakeQueued {
			user: user_a.clone(),
			pool_proposal_index: 1u128,
			amount: 490_000_000_000_000_000_000u128,
		})]);

		// Withdraw succeed
		assert_ok!(PoolProposal::withdraw_pre_investing(
			RuntimeOrigin::signed(user_a.clone()),
			1u128,
			490_000_000_000_000_000_000u128,
		));
		assert_events(vec![RuntimeEvent::PoolProposal(crate::Event::PoolWithdrawed {
			user: user_a.clone(),
			pool_proposal_index: 1u128,
			amount: 490_000_000_000_000_000_000u128,
		})]);
	})
}

#[test]
fn test_public_vote_proposal_ok() {
	new_test_ext().execute_with(|| {
		// Pool not exist
		assert_noop!(
			PoolProposal::public_vote_proposal(RuntimeOrigin::root(), 1u128, true,),
			crate::Error::<Test>::ProposalNotExist
		);

		let user_a: AccountId32 = AccountId32::from([1u8; 32]);
		let user_b: AccountId32 = AccountId32::from([2u8; 32]);
		Balances::make_free_balance_be(&user_a, 100_000_000_000_000_000_000);
		Balances::make_free_balance_be(&user_b, 100_000_000_000_000_000_000);

		let max_pool_size = 10_000_000_000_000_000_000u128;
		let proposal_last_time = 100;
		let pool_last_time = 10000;
		let estimated_pool_reward = 2_000_000_000_000_000_000_000u128;
		let pool_info_hash: H256 = H256([2; 32]);

		// Setup
		assert_ok!(PoolProposal::propose_investing_pool(
			RuntimeOrigin::signed(user_a.clone()),
			max_pool_size,
			proposal_last_time,
			pool_last_time,
			estimated_pool_reward,
			pool_info_hash
		));

		// Wrong origin
		assert_noop!(
			PoolProposal::public_vote_proposal(RuntimeOrigin::signed(user_a), 1u128, true,),
			sp_runtime::DispatchError::BadOrigin
		);

		// Works
		assert_ok!(PoolProposal::public_vote_proposal(RuntimeOrigin::root(), 1u128, true,));
		assert_events(vec![RuntimeEvent::PoolProposal(crate::Event::ProposalPublicVoted {
			pool_proposal_index: 1u128,
			vote_result: true,
		})]);
	})
}

#[test]
fn test_guardian_participate_proposal_ok() {
	new_test_ext().execute_with(|| {
		let user_a: AccountId32 = AccountId32::from([1u8; 32]);
		assert_ok!(PoolProposal::guardian_participate_proposal(
			RuntimeOrigin::signed(user_a.clone()),
			1u128,
		));

		assert_events(vec![RuntimeEvent::PoolProposal(crate::Event::PorposalGuardian {
			pool_proposal_index: 1u128,
			guardian: user_a,
		})]);
	})
}

// TODO: test_bake_proposal_ok, test_dissolve_proposal_ok, solve pending test, types test
