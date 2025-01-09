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
use crate::mock_zero_delay::{ExtBuilder, RuntimeCall, RuntimeOrigin, Test, Utility};
use frame_support::assert_ok;

use crate::Call as ParachainStakingCall;

#[test]
fn batch_unstake_and_leave_delegators_works_if_zero_delay() {
	ExtBuilder::default()
		.with_balances(vec![(1, 130), (2, 110)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			// can execute immediately
			assert_ok!(Utility::batch_all(
				RuntimeOrigin::signed(2),
				vec![
					RuntimeCall::ParachainStaking(
						ParachainStakingCall::schedule_leave_delegators {}
					),
					RuntimeCall::ParachainStaking(ParachainStakingCall::execute_leave_delegators {
						delegator: 2
					}),
				]
			));
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
			// can execute immediately
			assert_ok!(Utility::batch_all(
				RuntimeOrigin::signed(2),
				vec![
					RuntimeCall::ParachainStaking(
						ParachainStakingCall::schedule_delegator_bond_less {
							candidate: 1,
							less: 1
						}
					),
					RuntimeCall::ParachainStaking(
						ParachainStakingCall::execute_delegation_request {
							delegator: 2,
							candidate: 1
						}
					),
				]
			));
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
			// can execute immediately
			assert_ok!(Utility::batch_all(
				RuntimeOrigin::signed(2),
				vec![
					RuntimeCall::ParachainStaking(
						ParachainStakingCall::schedule_revoke_delegation { collator: 1 }
					),
					RuntimeCall::ParachainStaking(
						ParachainStakingCall::execute_delegation_request {
							delegator: 2,
							candidate: 1
						}
					),
				]
			));
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
