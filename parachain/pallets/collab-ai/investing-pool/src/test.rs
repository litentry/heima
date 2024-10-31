use super::{
	mock::{
		assert_events, new_test_ext, Assets, Balances, HavlingMintId, RuntimeEvent, RuntimeOrigin,
		InvestingPool, InvestingPoolId, System, Test, ENDOWED_BALANCE, USER_A, USER_B, USER_C,
	},
	*,
};
use frame_support::{assert_noop, assert_ok};

fn next_block() {
	System::set_block_number(System::block_number() + 1);
	InvestingPool::begin_block(System::block_number());
}

fn fast_forward_to(n: u64) {
	while System::block_number() < n {
		next_block();
	}
}

#[test]
fn can_not_create_pool_already_started_or_existed() {
	new_test_ext().execute_with(|| {
		// Create stable investing pool
		let pool_setup: PoolSetting<u64, u64> = PoolSetting {
			start_time: 100u64,
			epoch: 10u128,
			epoch_range: 100u64,
			setup_time: 200u64,
			pool_cap: 1_000_000_000u64,
		};
		assert_noop!(
			InvestingPool::create_investing_pool(RuntimeOrigin::root(), 1u128, pool_setup.clone()),
			Error::<Test>::PoolAlreadyExisted
		);
		// Transfer and check result
		fast_forward_to(101);
		assert_noop!(
			InvestingPool::create_investing_pool(RuntimeOrigin::root(), 2u128, pool_setup.clone()),
			Error::<Test>::PoolAlreadyStarted
		);
		// Create another pool is fine
		let another_pool_setup: PoolSetting<u64, u64> = PoolSetting {
			start_time: 150u64,
			epoch: 10u128,
			epoch_range: 100u64,
			setup_time: 200u64,
			pool_cap: 1_000_000_000u64,
		};
		assert_ok!(InvestingPool::create_investing_pool(
			RuntimeOrigin::root(),
			2u128,
			another_pool_setup
		));
		assert_events(vec![RuntimeEvent::InvestingPool(Event::InvestingPoolCreated {
			pool_id: 2u128,
			start_time: 150u64,
			epoch: 10u128,
			epoch_range: 100u64,
			setup_time: 200u64,
			pool_cap: 1_000_000_000u64,
		})]);
	})
}

// TODO: update_metadata test
// Currently metadata does nothing but description

#[test]
fn update_reward_successful_and_failed() {
	new_test_ext().execute_with(|| {
		let stable_token_pool: u64 = InvestingPoolId::get().into_account_truncating();

		// update epoch 0 reward with amount of 2000
		assert_ok!(InvestingPool::update_reward(RuntimeOrigin::root(), 1u128, 0u128, 2000u64));
		assert_events(vec![RuntimeEvent::InvestingPool(Event::RewardUpdated {
			pool_id: 1u128,
			epoch: 0u128,
			amount: 2000u64,
		})]);
		// Investing pool reward storage efffective
		assert_eq!(InvestingPool::stable_investing_pool_reward(1u128), 2000u64);
		assert_eq!(InvestingPool::stable_investing_pool_epoch_reward(1u128, 0u128), Some(2000u64));
		assert_eq!(InvestingPool::stable_investing_pool_epoch_reward(1u128, 1u128), None);
		// Investing pool balance effective
		assert_eq!(Assets::balance(1u32, stable_token_pool), 2000u64);

		// Can not update epoch reward twice
		assert_noop!(
			InvestingPool::update_reward(RuntimeOrigin::root(), 1u128, 0u128, 1000u64),
			Error::<Test>::RewardAlreadyExisted
		);

		// Pool started at 100, epoch range = 100, epoch = 10
		// So Blocknumber 301 => Epoch 2 started/Epoch 1 ended
		System::set_block_number(301u64);
		// Can not update epoch already ended
		assert_noop!(
			InvestingPool::update_reward(RuntimeOrigin::root(), 1u128, 1u128, 1000u64),
			Error::<Test>::EpochAlreadyEnded
		);

		// Epoch reward can not be updated to non-existing pool
		assert_noop!(
			InvestingPool::update_reward(RuntimeOrigin::root(), 9999u128, 1u128, 1000u64),
			Error::<Test>::PoolNotExisted
		);
		// Epoch reward can not be updated to epoch index not existing
		assert_noop!(
			InvestingPool::update_reward(RuntimeOrigin::root(), 1u128, 11u128, 1000u64),
			Error::<Test>::EpochNotExist
		);

		// Epoch reward update for "last epoch" (pool end time's next epoch) always success
		// Pool epoch = 10
		System::set_block_number(9999999u64);
		assert_ok!(InvestingPool::update_reward(RuntimeOrigin::root(), 1u128, 10u128, 2000u64));
		assert_events(vec![RuntimeEvent::InvestingPool(Event::RewardUpdated {
			pool_id: 1u128,
			epoch: 10u128,
			amount: 2000u64,
		})]);

		// Can not update reward if no AIUSD registed
		System::set_block_number(301u64);
		<AIUSDAssetId<Test>>::kill();
		assert_noop!(
			InvestingPool::update_reward(RuntimeOrigin::root(), 1u128, 5u128, 2000u64),
			Error::<Test>::NoAssetId
		);
	})
}

#[test]
fn stake_successful_and_failed() {
	new_test_ext().execute_with(|| {
		let stable_token_pool: u64 = InvestingPoolId::get().into_account_truncating();

		// Can not stake non-exist pool
		assert_noop!(
			InvestingPool::stake(RuntimeOrigin::signed(USER_A), 2u128, 2000u64),
			Error::<Test>::PoolNotExisted
		);

		// Can not stake non-started pool
		assert_noop!(
			InvestingPool::stake(RuntimeOrigin::signed(USER_A), 1u128, 2000u64),
			Error::<Test>::PoolNotStarted
		);

		// Can not stake ended pool
		System::set_block_number(9999999u64);
		assert_noop!(
			InvestingPool::stake(RuntimeOrigin::signed(USER_A), 1u128, 2000u64),
			Error::<Test>::PoolAlreadyEnded
		);

		// Success, check user/global native checkpoint storage
		// check pending set up storage
		System::set_block_number(301u64);
		// Can not stake oversized
		assert_noop!(
			InvestingPool::stake(RuntimeOrigin::signed(USER_A), 1u128, 50_000_001u64),
			Error::<Test>::PoolCapLimit
		);

		assert_ok!(InvestingPool::stake(RuntimeOrigin::signed(USER_A), 1u128, 2000u64));
		assert_events(vec![RuntimeEvent::InvestingPool(Event::Staked {
			who: USER_A,
			pool_id: 1u128,
			target_effective_time: 600u64,
			amount: 2000u64,
		})]);
		assert_eq!(Assets::balance(1u32, USER_A), ENDOWED_BALANCE - 2000u64);
		assert_eq!(Assets::balance(1u32, stable_token_pool), 2000u64);
		let global_investing_info = InvestingPool::native_checkpoint().unwrap();
		assert_eq!(
			global_investing_info,
			InvestingWeightInfo { effective_time: 301u64, amount: 2000u64, last_add_time: 301u64 }
		);
		let user_a_investing_info = InvestingPool::user_native_checkpoint(USER_A).unwrap();
		assert_eq!(
			user_a_investing_info,
			InvestingWeightInfo { effective_time: 301u64, amount: 2000u64, last_add_time: 301u64 }
		);
		let pending_set_up = InvestingPool::pending_setup();
		assert_eq!(pending_set_up.len(), 1);
		let pending_set_up_element = pending_set_up.get(0).unwrap();
		// Pool set up time = 200
		// So user enter at 301 need to wait till 600 to make it effective and receiving Stable
		// investing reward
		assert_eq!(
			*pending_set_up_element,
			InvestingWeightInfoWithOwner {
				who: USER_A,
				pool_id: 1u128,
				investing_info: InvestingWeightInfo {
					effective_time: 600u64,
					amount: 2000u64,
					last_add_time: 600u64
				}
			}
		);

		// Second user B stake
		System::set_block_number(399u64);
		fast_forward_to(411u64);
		assert_ok!(InvestingPool::stake(RuntimeOrigin::signed(USER_B), 1u128, 1000u64));
		assert_events(vec![RuntimeEvent::InvestingPool(Event::Staked {
			who: USER_B,
			pool_id: 1u128,
			target_effective_time: 700u64,
			amount: 1000u64,
		})]);

		assert_eq!(Assets::balance(1u32, USER_B), ENDOWED_BALANCE - 1000u64);
		assert_eq!(Assets::balance(1u32, stable_token_pool), 2000u64 + 1000u64);
		let global_investing_info = InvestingPool::native_checkpoint().unwrap();
		// Synthetic (301, 2000), (411, 1000) = (337.6666, 3000)
		assert_eq!(
			global_investing_info,
			InvestingWeightInfo { effective_time: 337u64, amount: 3000u64, last_add_time: 411u64 }
		);
		// user a unchanged
		let user_a_investing_info = InvestingPool::user_native_checkpoint(USER_A).unwrap();
		assert_eq!(
			user_a_investing_info,
			InvestingWeightInfo { effective_time: 301u64, amount: 2000u64, last_add_time: 301u64 }
		);
		// user b
		let user_b_investing_info = InvestingPool::user_native_checkpoint(USER_B).unwrap();
		assert_eq!(
			user_b_investing_info,
			InvestingWeightInfo { effective_time: 411u64, amount: 1000u64, last_add_time: 411u64 }
		);
		// Pending set up storage change
		let pending_set_up = InvestingPool::pending_setup();
		assert_eq!(pending_set_up.len(), 2);
		// pending set up is ordered by effective time, so user_b's request is at index 1 while
		// user_a is at index 0
		let pending_set_up_element = pending_set_up.get(1).unwrap();
		// Pool set up time = 200
		// So user enter at 411 need to wait till 700 to make it effective and receiving Stable
		// investing reward
		assert_eq!(
			*pending_set_up_element,
			InvestingWeightInfoWithOwner {
				who: USER_B,
				pool_id: 1u128,
				investing_info: InvestingWeightInfo {
					effective_time: 700u64,
					amount: 1000u64,
					last_add_time: 700u64
				}
			}
		);

		// Can not stake if no AIUSD registed
		<AIUSDAssetId<Test>>::kill();
		assert_noop!(
			InvestingPool::stake(RuntimeOrigin::signed(USER_C), 1u128, 3000u64),
			Error::<Test>::NoAssetId
		);
		assert_ok!(InvestingPool::regist_aiusd(RuntimeOrigin::root(), 1u32));

		// Can not stake oversized
		assert_noop!(
			InvestingPool::stake(
				RuntimeOrigin::signed(USER_A),
				1u128,
				50_000_001u64 - 2000u64 - 1000u64
			),
			Error::<Test>::PoolCapLimit
		);
		assert_ok!(InvestingPool::stake(
			RuntimeOrigin::signed(USER_A),
			1u128,
			50_000_000u64 - 2000u64 - 1000u64
		));
	})
}

#[test]
fn solve_pending_stake_and_hook_works() {
	new_test_ext().execute_with(|| {
		// Success, check user/global native checkpoint storage
		// check pending set up storage
		System::set_block_number(301u64);
		assert_ok!(InvestingPool::stake(RuntimeOrigin::signed(USER_A), 1u128, 2000u64));
		// Pool set up time = 200
		// So user enter at 301 need to wait till 600 to make it effective and receiving Stable
		System::set_block_number(590u64);
		// Try trigger hook
		fast_forward_to(610u64);
		assert_events(vec![RuntimeEvent::InvestingPool(Event::PendingInvestingSolved {
			who: USER_A,
			pool_id: 1u128,
			effective_time: 600u64,
			amount: 2000u64,
		})]);
		// No more pending
		let pending_set_up = InvestingPool::pending_setup();
		assert_eq!(pending_set_up.len(), 0);
		// Check stable investing checkpoint
		let global_investing_info = InvestingPool::stable_investing_pool_checkpoint(1u128).unwrap();
		assert_eq!(
			global_investing_info,
			InvestingWeightInfo { effective_time: 600u64, amount: 2000u64, last_add_time: 600u64 }
		);
		let user_a_investing_info =
			InvestingPool::user_stable_investing_pool_checkpoint(USER_A, 1u128).unwrap();
		assert_eq!(
			user_a_investing_info,
			InvestingWeightInfo { effective_time: 600u64, amount: 2000u64, last_add_time: 600u64 }
		);

		// Second user B stake
		assert_ok!(InvestingPool::stake(RuntimeOrigin::signed(USER_B), 1u128, 1000u64));
		// Any one can trigger manual, but right now no effect
		let pending_set_up = InvestingPool::pending_setup();
		assert_eq!(pending_set_up.len(), 1);
		assert_ok!(InvestingPool::solve_pending_stake(RuntimeOrigin::signed(USER_C)));
		let pending_set_up = InvestingPool::pending_setup();
		assert_eq!(pending_set_up.len(), 1);

		// Pool set up time = 200, current block = 610
		// So user enter at 301 need to wait till 900 to make it effective and receiving Stable
		// set block number without triggering hook
		System::set_block_number(910u64);
		// Global investing no changed
		let global_investing_info = InvestingPool::stable_investing_pool_checkpoint(1u128).unwrap();
		assert_eq!(
			global_investing_info,
			InvestingWeightInfo { effective_time: 600u64, amount: 2000u64, last_add_time: 600u64 }
		);
		// User b investing is still none
		assert!(InvestingPool::user_stable_investing_pool_checkpoint(USER_B, 1u128).is_none());

		// Any one can trigger manual
		// But effective time will be the time when triggered, which is 910
		assert_ok!(InvestingPool::solve_pending_stake(RuntimeOrigin::signed(USER_C)));
		assert_events(vec![RuntimeEvent::InvestingPool(Event::PendingInvestingSolved {
			who: USER_B,
			pool_id: 1u128,
			effective_time: 910u64,
			amount: 1000u64,
		})]);
		let pending_set_up = InvestingPool::pending_setup();
		// Pending solved
		assert_eq!(pending_set_up.len(), 0);
		// User B stable investing checkpoint updated
		let user_b_investing_info =
			InvestingPool::user_stable_investing_pool_checkpoint(USER_B, 1u128).unwrap();
		// The effective time is delayed accordingly
		assert_eq!(
			user_b_investing_info,
			InvestingWeightInfo { effective_time: 910u64, amount: 1000u64, last_add_time: 910u64 }
		);
		// Global investing check
		// (600, 2000), (910, 1000) -> (703.333, 3000)
		let global_investing_info = InvestingPool::stable_investing_pool_checkpoint(1u128).unwrap();
		assert_eq!(
			global_investing_info,
			InvestingWeightInfo { effective_time: 703u64, amount: 3000u64, last_add_time: 910u64 }
		);
	})
}

#[test]
fn claim_native_successful_and_failed() {
	new_test_ext().execute_with(|| {
		let native_token_pool: u64 = HavlingMintId::get().into_account_truncating();

		System::set_block_number(301u64);
		assert_ok!(InvestingPool::stake(RuntimeOrigin::signed(USER_A), 1u128, 2000u64));
		System::set_block_number(401u64);
		assert_ok!(InvestingPool::stake(RuntimeOrigin::signed(USER_A), 1u128, 2000u64));
		assert_ok!(InvestingPool::stake(RuntimeOrigin::signed(USER_B), 1u128, 1000u64));
		// at block 401:
		// User_A : (351, 4000) with last_add_time = 401
		// User_B : (401, 1000) with last_add_time = 401
		// Global : (361 ,5000) with last_add_time = 401

		// Just for convenience, suppose there are already 100 times ENDOWED_BALANCE  native token
		// reward
		assert_eq!(
			Balances::set_balance(&native_token_pool, 100 * ENDOWED_BALANCE),
			100 * ENDOWED_BALANCE
		);

		System::set_block_number(601u64);
		// User_a try claim before 401, failed since it is not allowed to claim before last_add_time
		// TODO:: TypeIncompatibleOrArithmeticError is not specific enough
		assert_noop!(
			InvestingPool::claim_native(RuntimeOrigin::signed(USER_A), 400u64),
			Error::<Test>::TypeIncompatibleOrArithmeticError
		);

		// A normal claim until 501 at time 601
		assert_ok!(InvestingPool::claim_native(RuntimeOrigin::signed(USER_A), 501u64));
		// total weight = 5000 * (501 - 361) = 700,000
		// claim weight = 4000 * (501 - 351) = 600,000
		// reward = 100 * ENDOWED_BALANCE * claim weight / total weight
		// 8571428571.428
		assert_events(vec![
			RuntimeEvent::Balances(pallet_balances::Event::Transfer {
				from: native_token_pool,
				to: USER_A,
				amount: 8_571_428_571u64,
			}),
			RuntimeEvent::InvestingPool(Event::NativeRewardClaimed {
				who: USER_A,
				until_time: 501u64,
				reward_amount: 8_571_428_571u64,
			}),
		]);
		// After claim
		// User_A : (501, 4000) with last_add_time = 401
		// User_B : (401, 1000) with last_add_time = 401
		// Global : weight before = (501 - 361) * 5000 = 700,000
		// Global : weight after = 700,000 - 600,000 = 100,000
		// Global : synthetic (501 - (100,000 / 5000), 5000) = (481, 5000)
		// check user a
		let user_a_investing_info = InvestingPool::user_native_checkpoint(USER_A).unwrap();
		assert_eq!(
			user_a_investing_info,
			InvestingWeightInfo { effective_time: 501u64, amount: 4000u64, last_add_time: 401u64 }
		);
		// check global
		let global_investing_info = InvestingPool::native_checkpoint().unwrap();
		assert_eq!(
			global_investing_info,
			InvestingWeightInfo { effective_time: 481u64, amount: 5000u64, last_add_time: 401u64 }
		);

		// Can not claim future
		assert_noop!(
			InvestingPool::claim_native(RuntimeOrigin::signed(USER_A), 602u64),
			Error::<Test>::EpochRewardNotUpdated
		);
	})
}

#[test]
fn claim_stable_successful_and_failed() {
	new_test_ext().execute_with(|| {
		let stable_token_pool: u64 = InvestingPoolId::get().into_account_truncating();

		System::set_block_number(301u64);
		// effective time = 600
		assert_ok!(InvestingPool::stake(RuntimeOrigin::signed(USER_A), 1u128, 2000u64));
		System::set_block_number(401u64);
		// effective time = 700
		assert_ok!(InvestingPool::stake(RuntimeOrigin::signed(USER_A), 1u128, 2000u64));
		assert_ok!(InvestingPool::stake(RuntimeOrigin::signed(USER_B), 1u128, 1000u64));

		// triggering pending by hook
		System::set_block_number(599u64);
		fast_forward_to(610u64);

		// Notice: Can not update reward if EpochAlreadyEnded
		// Pool: start_time: 100u64,
		//	     epoch: 10u128,
		//     	 epoch_range: 100u64,
		//       setup_time: 200u64,
		// 610 => current epoch 5
		let stable_token_pool_balance = Assets::balance(1u32, stable_token_pool);
		assert_ok!(InvestingPool::update_reward(RuntimeOrigin::root(), 1u128, 5u128, 2000u64));
		assert_eq!(Assets::balance(1u32, stable_token_pool), stable_token_pool_balance + 2000u64);

		// Only user a yet, claiming from 600 to 650
		// Stable investing
		// User_A : (600, 2000) with last_add_time = 600
		// Global : (600 ,2000) with last_add_time = 600
		// claimed weight: 100% out of total!
		System::set_block_number(699u64);
		assert_ok!(InvestingPool::claim_stable(RuntimeOrigin::signed(USER_A), 1u128, 650u64));
		assert_eq!(Assets::balance(1u32, stable_token_pool), stable_token_pool_balance);
		assert_events(vec![
			RuntimeEvent::Assets(pallet_assets::Event::Transferred {
				asset_id: 1u32,
				from: stable_token_pool,
				to: USER_A,
				amount: 2000u64,
			}),
			RuntimeEvent::InvestingPool(Event::StableRewardClaimed {
				who: USER_A,
				pool_id: 1u128,
				until_time: 650u64,
				reward_amount: 2000u64,
			}),
		]);
		// Check stable investing checkpoint storage
		// check user
		let user_a_investing_info =
			InvestingPool::user_stable_investing_pool_checkpoint(USER_A, 1u128).unwrap();
		assert_eq!(
			user_a_investing_info,
			InvestingWeightInfo { effective_time: 650u64, amount: 2000u64, last_add_time: 600u64 }
		);
		// check global
		let global_investing_info = InvestingPool::stable_investing_pool_checkpoint(1u128).unwrap();
		assert_eq!(
			global_investing_info,
			InvestingWeightInfo { effective_time: 650u64, amount: 2000u64, last_add_time: 600u64 }
		);

		fast_forward_to(710u64);
		// 710 => current epoch 6
		assert_ok!(InvestingPool::update_reward(RuntimeOrigin::root(), 1u128, 6u128, 4000u64));
		assert_eq!(Assets::balance(1u32, stable_token_pool), stable_token_pool_balance + 4000u64);
		System::set_block_number(799u64);
		// Stable investing
		// User_A : (650, 2000) + (700, 2000) = (675, 4000) with last_add_time = 700
		// User_B : (700, 1000) with last_add_time = 700
		// Global : (650, 2000) + (700, 2000) + (700, 1000) = (680, 5000) with last_add_time = 600
		// Claimed weight: (750 - 675) * 4000 = 300_000
		// Total weight: (750 - 680) * 5000 = 350_000
		// claimed reward = 4000 * 300_000 / 350_000 = 3428.571
		assert_ok!(InvestingPool::claim_stable(RuntimeOrigin::signed(USER_A), 1u128, 750u64));

		assert_events(vec![
			RuntimeEvent::Assets(pallet_assets::Event::Transferred {
				asset_id: 1u32,
				from: stable_token_pool,
				to: USER_A,
				amount: 3429u64,
			}),
			RuntimeEvent::InvestingPool(Event::StableRewardClaimed {
				who: USER_A,
				pool_id: 1u128,
				until_time: 750u64,
				reward_amount: 3429u64,
			}),
		]);

		// Check stable investing checkpoint storage
		// check user
		let user_a_investing_info =
			InvestingPool::user_stable_investing_pool_checkpoint(USER_A, 1u128).unwrap();
		assert_eq!(
			user_a_investing_info,
			InvestingWeightInfo { effective_time: 750u64, amount: 4000u64, last_add_time: 700u64 }
		);
		// check global
		let global_investing_info = InvestingPool::stable_investing_pool_checkpoint(1u128).unwrap();
		assert_eq!(
			global_investing_info,
			InvestingWeightInfo { effective_time: 740u64, amount: 5000u64, last_add_time: 700u64 }
		);
	})
}

#[test]
fn withdraw_works() {
	new_test_ext().execute_with(|| {})
}

#[test]
fn regist_aiusd_works() {
	new_test_ext().execute_with(|| {})
}

#[test]
fn get_epoch_index() {
	new_test_ext().execute_with(|| {})
}

#[test]
fn get_epoch_begin_time() {
	new_test_ext().execute_with(|| {})
}

// pending swap storage does get a proper order for multiple pools and can handle multiple pending
// orders double add at same time behavior of (native) stable checkpoint
// if user does not claim reward, and reward update is percentage fair, user is impact
// if user does not claim reward, and reward update is percentage improved in future, user is
// benefited if user does not claim reward, and reward update is percentage downgraded in future,
// user is harmed claim reward behavior of native and stable checkpoint
// withdraw behavior of native and stable checkingpoint