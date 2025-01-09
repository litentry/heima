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
	mock_zero_delay::{ExtBuilder, ParachainStaking, RuntimeCall, RuntimeOrigin, Test, Utility},
	Error,
};
use frame_support::{assert_noop, assert_ok};

use crate::Call as ParachainStakingCall;

#[test]
fn batch_unstake_and_leave_delegators_works_if_zero_delay() {
	ExtBuilder::default()
		.with_balances(vec![(1, 130), (2, 110)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			// Execute immediately
			assert_ok!(ParachainStaking::schedule_leave_delegators(RuntimeOrigin::signed(2)));
			assert_noop!(
				ParachainStaking::execute_leave_delegators(RuntimeOrigin::signed(2), 2),
				Error::<Test>::DelegatorDNE
			);
		});
}

#[test]
fn batch_unstake_and_leave_candidates_works_if_zero_delay() {
	ExtBuilder::default()
		.with_balances(vec![(1, 110)])
		.with_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			// can execute immediately
			assert_ok!(Utility::batch_all(
				RuntimeOrigin::signed(1),
				vec![
					RuntimeCall::ParachainStaking(
						ParachainStakingCall::schedule_leave_candidates {}
					),
					RuntimeCall::ParachainStaking(ParachainStakingCall::execute_leave_candidates {
						candidate: 1
					}),
				]
			));
		});
}

#[test]
fn batch_unstake_and_delegator_bond_less_works_if_zero_delay() {
	ExtBuilder::default()
		.with_balances(vec![(1, 130), (2, 110)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			// Execute immediately
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				RuntimeOrigin::signed(2),
				1,
				1
			));
			assert_noop!(
				ParachainStaking::execute_delegation_request(RuntimeOrigin::signed(2), 2, 1),
				Error::<Test>::PendingDelegationRequestDNE
			);
		});
}

#[test]
fn batch_unstake_and_revoke_delegation_works_if_zero_delay() {
	ExtBuilder::default()
		.with_balances(vec![(1, 130), (2, 110)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			// Execute immediately
			assert_ok!(ParachainStaking::schedule_revoke_delegation(RuntimeOrigin::signed(2), 1));
			assert_noop!(
				ParachainStaking::execute_delegation_request(RuntimeOrigin::signed(2), 2, 1),
				Error::<Test>::DelegatorDNE
			);
		});
}

#[test]
fn batch_unstake_and_candidate_bond_less_works_if_zero_delay() {
	ExtBuilder::default()
		.with_balances(vec![(1, 110)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			// can execute immediately
			assert_ok!(Utility::batch_all(
				RuntimeOrigin::signed(1),
				vec![
					RuntimeCall::ParachainStaking(
						ParachainStakingCall::schedule_candidate_bond_less { less: 1 }
					),
					RuntimeCall::ParachainStaking(
						ParachainStakingCall::execute_candidate_bond_less { candidate: 1 }
					),
				]
			));
		});
}
