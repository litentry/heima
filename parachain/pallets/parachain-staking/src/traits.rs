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

//! Inspired by:
//! - Moonbeam `pallet_parachain_staking`
//! implementations.

//! traits for parachain-staking
use frame_support::pallet_prelude::Weight;
pub trait OnCollatorPayout<AccountId, Balance> {
	fn on_collator_payout(
		for_round: crate::RoundIndex,
		collator_id: AccountId,
		amount: Balance,
	) -> Weight;
}
impl<AccountId, Balance> OnCollatorPayout<AccountId, Balance> for () {
	fn on_collator_payout(
		_for_round: crate::RoundIndex,
		_collator_id: AccountId,
		_amount: Balance,
	) -> Weight {
		Weight::zero()
	}
}

pub trait OnNewRound {
	fn on_new_round(round_index: crate::RoundIndex) -> Weight;
}
impl OnNewRound for () {
	fn on_new_round(_round_index: crate::RoundIndex) -> Weight {
		Weight::zero()
	}
}

pub trait IssuanceAdapter<Balance> {
	fn adapted_total_issuance() -> Balance;
}

use sp_runtime::traits::AtLeast32BitUnsigned;
impl<Balance: AtLeast32BitUnsigned> IssuanceAdapter<Balance> for () {
	fn adapted_total_issuance() -> Balance {
		0u32.into()
	}
}

pub trait OnAllDelegationRemoved<T: crate::Config> {
	fn on_all_delegation_removed(delegator: &T::AccountId) -> Result<(), &str>;
}

pub trait TransactionFeeRewardResource<Balance, RoundIndex, Account> {
	fn on_transaction_fee_reward_notify(amount: Balance, collator: Account) -> Result<(), &str>;

	fn query_round_fee_reward(round: RoundIndex, collator: Account) -> Balance;
}

impl<Balance: AtLeast32BitUnsigned> TransactionFeeRewardResource<Balance, RoundIndex, Account>
	for ()
{
	fn on_transaction_fee_reward_notify(amount: Balance, collator: Account) -> Result<(), &str> {
		Ok(())
	}

	fn query_round_fee_reward(round: RoundIndex, collator: Account) -> Balance {
		0u32.into()
	}
}
