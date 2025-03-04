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

use crate as pallet_parachain_staking_fee_reward;
use crate::Config;
use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{OnFinalize, OnInitialize},
};
use pallet_parachain_staking::{InflationInfo, Range};
use sp_runtime::{BuildStorage, Perbill, Percent};
pub type AccountId = u64;
pub type Balance = u128;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		ParachainStaking: pallet_parachain_staking,
		ParachainStakingFeeReward: pallet_parachain_staking_fee_reward,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = frame_system::mocking::MockBlock<Test>;
	type AccountData = pallet_balances::AccountData<Balance>;
}

parameter_types! {
	pub const ExistentialDeposit: Balance = 1;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type Balance = Balance;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
}

parameter_types! {
	pub const MinBlocksPerRound: u32 = 3;
	pub const DefaultBlocksPerRound: u32 = 5;
	pub const LeaveCandidatesDelay: u32 = 2;
	pub const CandidateBondLessDelay: u32 = 2;
	pub const LeaveDelegatorsDelay: u32 = 2;
	pub const RevokeDelegationDelay: u32 = 2;
	pub const DelegationBondLessDelay: u32 = 2;
	pub const RewardPaymentDelay: u32 = 2;
	pub const MinSelectedCandidates: u32 = 5;
	pub const MaxTopDelegationsPerCandidate: u32 = 4;
	pub const MaxBottomDelegationsPerCandidate: u32 = 4;
	pub const MaxDelegationsPerDelegator: u32 = 4;
	pub const DefaultCollatorCommission: Perbill = Perbill::from_percent(20);
	pub const DefaultParachainBondReservePercent: Percent = Percent::from_percent(30);
	pub const MinCollatorStk: u128 = 10;
	pub const MinDelegatorStk: u128 = 5;
	pub const MinDelegation: u128 = 3;
	pub const RewardPaymentRecordRound: u32 = 3;
}

impl pallet_parachain_staking::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MonetaryGovernanceOrigin = frame_system::EnsureRoot<AccountId>;
	type MinBlocksPerRound = MinBlocksPerRound;
	type DefaultBlocksPerRound = DefaultBlocksPerRound;
	type LeaveCandidatesDelay = LeaveCandidatesDelay;
	type CandidateBondLessDelay = CandidateBondLessDelay;
	type LeaveDelegatorsDelay = LeaveDelegatorsDelay;
	type RevokeDelegationDelay = RevokeDelegationDelay;
	type DelegationBondLessDelay = DelegationBondLessDelay;
	type RewardPaymentDelay = RewardPaymentDelay;
	type MinSelectedCandidates = MinSelectedCandidates;
	type MaxTopDelegationsPerCandidate = MaxTopDelegationsPerCandidate;
	type MaxBottomDelegationsPerCandidate = MaxBottomDelegationsPerCandidate;
	type MaxDelegationsPerDelegator = MaxDelegationsPerDelegator;
	type DefaultCollatorCommission = DefaultCollatorCommission;
	type DefaultParachainBondReservePercent = DefaultParachainBondReservePercent;
	type MinCollatorStk = MinCollatorStk;
	type MinCandidateStk = MinCollatorStk;
	type MinDelegatorStk = MinDelegatorStk;
	type MinDelegation = MinDelegation;
	type OnCollatorPayout = ();
	type OnNewRound = ();
	type WeightInfo = ();
	type IssuanceAdapter = ();
	type OnAllDelegationRemoved = ();
	type RoundFeeRewardResource = ParachainStakingFeeReward;
}

impl pallet_parachain_staking::OnAllDelegationRemoved<Test> for () {
	fn on_all_delegation_removed(
		_delegator: &<Test as frame_system::Config>::AccountId,
	) -> Result<(), &str> {
		Ok(())
	}
}

impl Config for Test {
	type RewardPaymentRecordRound = RewardPaymentRecordRound;
}

pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
	// [collator, amount]
	collators: Vec<(AccountId, Balance)>,
	// [delegator, collator, delegation_amount]
	delegations: Vec<(AccountId, AccountId, Balance, Percent)>,
	// inflation config
	inflation: InflationInfo<Balance>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: vec![],
			delegations: vec![],
			collators: vec![],
			inflation: InflationInfo {
				expect: Range { min: 700, ideal: 700, max: 700 },
				// not used
				annual: Range {
					min: Perbill::from_percent(50),
					ideal: Perbill::from_percent(50),
					max: Perbill::from_percent(50),
				},
				// All zero, so only RoundFeeRewardResource contribute to reward amount
				round: Range {
					min: Perbill::from_percent(0),
					ideal: Perbill::from_percent(0),
					max: Perbill::from_percent(0),
				},
			},
		}
	}
}

impl ExtBuilder {
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub(crate) fn with_candidates(mut self, collators: Vec<(AccountId, Balance)>) -> Self {
		self.collators = collators;
		self
	}

	pub(crate) fn with_delegations(
		mut self,
		delegations: Vec<(AccountId, AccountId, Balance)>,
	) -> Self {
		self.delegations =
			delegations.into_iter().map(|d| (d.0, d.1, d.2, Percent::zero())).collect();
		self
	}

	pub(crate) fn with_auto_compounding_delegations(
		mut self,
		delegations: Vec<(AccountId, AccountId, Balance, Percent)>,
	) -> Self {
		self.delegations = delegations;
		self
	}

	#[allow(dead_code)]
	pub(crate) fn with_inflation(mut self, inflation: InflationInfo<Balance>) -> Self {
		self.inflation = inflation;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

		pallet_balances::GenesisConfig::<Test> { balances: self.balances }
			.assimilate_storage(&mut t)
			.expect("Pallet balances storage can be assimilated");
		pallet_parachain_staking::GenesisConfig::<Test> {
			candidates: self.collators,
			delegations: self.delegations,
			inflation_config: self.inflation,
		}
		.assimilate_storage(&mut t)
		.expect("Parachain Staking's storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

/// Rolls forward one block. Returns the new block number.
pub(crate) fn roll_one_block() -> u64 {
	ParachainStaking::on_finalize(System::block_number());
	Balances::on_finalize(System::block_number());
	System::on_finalize(System::block_number());
	System::set_block_number(System::block_number() + 1);
	System::on_initialize(System::block_number());
	Balances::on_initialize(System::block_number());
	ParachainStaking::on_initialize(System::block_number());
	System::block_number()
}

/// Rolls to the desired block. Returns the number of blocks played.
pub(crate) fn roll_to(n: u64) -> u64 {
	let mut num_blocks = 0;
	let mut block = System::block_number();
	while block < n {
		block = roll_one_block();
		num_blocks += 1;
	}
	num_blocks
}

/// Rolls block-by-block to the beginning of the specified round.
/// This will complete the block in which the round change occurs.
/// Returns the number of blocks played.
pub(crate) fn roll_to_round_begin(round: u64) -> u64 {
	let block = (round - 1) * DefaultBlocksPerRound::get() as u64;
	roll_to(block)
}

/// Rolls block-by-block to the end of the specified round.
/// The block following will be the one in which the specified round change occurs.
pub(crate) fn roll_to_round_end(round: u64) -> u64 {
	let block = round * DefaultBlocksPerRound::get() as u64 - 1;
	roll_to(block)
}
