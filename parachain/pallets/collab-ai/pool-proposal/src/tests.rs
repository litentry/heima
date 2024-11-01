use crate::mock::*;
use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency};
use sp_core::H256;
use sp_runtime::{AccountId32, TokenError};

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

		// Bad origin
		assert_noop!(
			PoolProposal::propose_investing_pool(
				RuntimeOrigin::signed(user_a.clone()),
				max_pool_size,
				proposal_last_time,
				pool_last_time,
				estimated_pool_reward,
				pool_info_hash
			),
			sp_runtime::DispatchError::BadOrigin
		);
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
			TokenError::FundsUnavailable
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
				pool_info_hash
			),
			crate::Error::<Test>::ProposalDepositDuplicatedOrOversized
		);
	})
}
