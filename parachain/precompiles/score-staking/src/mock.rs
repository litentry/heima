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

use super::*;
use frame_support::{
	assert_ok, construct_runtime, derive_impl, parameter_types,
	traits::{OnFinalize, OnInitialize},
	weights::Weight,
};
use frame_system::EnsureRoot;
use pallet_evm::{AddressMapping, EnsureAddressNever, EnsureAddressRoot};
use pallet_score_staking::{AccountIdConvert, PoolState, RoundSetting};
use precompile_utils::precompile_set::{AddressU64, PrecompileAt, PrecompileSetBuilder};
use sp_core::{ConstU128, ConstU32, ConstU64, H160, H256};
use sp_runtime::{traits::IdentityLookup, BuildStorage, Perbill, Percent};

pub type Balance = u128;
pub const UNIT: Balance = 1_000_000_000_000;
pub type AccountId = sp_runtime::AccountId32;

construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		Evm: pallet_evm,
		ParachainStaking: pallet_parachain_staking,
		ScoreStaking: pallet_score_staking,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type AccountId = AccountId;
	type Block = frame_system::mocking::MockBlock<Test>;
	type AccountData = pallet_balances::AccountData<Balance>;
	type Lookup = IdentityLookup<Self::AccountId>;
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
}
impl pallet_parachain_staking::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MonetaryGovernanceOrigin = EnsureRoot<AccountId>;
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
}

impl pallet_parachain_staking::OnAllDelegationRemoved<Test> for () {
	fn on_all_delegation_removed(
		_delegator: &<Test as frame_system::Config>::AccountId,
	) -> Result<(), &str> {
		Ok(())
	}
}

parameter_types! {
	pub const DefaultYearlyInflation: Perbill = Perbill::from_perthousand(5);
}

impl pallet_score_staking::Config for Test {
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type AccountIdConvert = IdentityAccountIdConvert;
	type AdminOrigin = EnsureRoot<AccountId>;
	type YearlyIssuance = ConstU128<{ 100_000_000 * UNIT }>;
	type YearlyInflation = DefaultYearlyInflation;
	type MaxScoreUserCount = ConstU32<2>;
}

pub fn precompile_address() -> H160 {
	H160::from_low_u64_be(20480 + 75)
}

pub type ScoreStakingMockPrecompile<R> =
	PrecompileSetBuilder<R, (PrecompileAt<AddressU64<20555>, ScoreStakingPrecompile<R>>,)>;

pub type PCall<Runtime> = ScoreStakingPrecompileCall<Runtime>;

pub struct TruncatedAddressMapping;
impl AddressMapping<AccountId> for TruncatedAddressMapping {
	fn into_account_id(address: H160) -> AccountId {
		let mut data = [0u8; 32];
		data[0..20].copy_from_slice(&address[..]);
		AccountId::from(Into::<[u8; 32]>::into(data))
	}
}

// silly for test purpose only
pub struct U8Wrapper(pub u8);
impl From<U8Wrapper> for H160 {
	fn from(x: U8Wrapper) -> H160 {
		H160::repeat_byte(x.0)
	}
}
impl From<U8Wrapper> for H256 {
	fn from(x: U8Wrapper) -> H256 {
		let h160 = H160::repeat_byte(x.0);
		let mut data = [0u8; 32];
		data[0..20].copy_from_slice(&h160[..]);
		data.into()
	}
}
impl From<U8Wrapper> for AccountId {
	fn from(x: U8Wrapper) -> AccountId {
		TruncatedAddressMapping::into_account_id(x.into())
	}
}

parameter_types! {
	pub PrecompilesValue: ScoreStakingMockPrecompile<Test> = ScoreStakingMockPrecompile::new();
	pub WeightPerGas: Weight = Weight::from_parts(1, 0);
}

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = TruncatedAddressMapping;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = ScoreStakingMockPrecompile<Self>;
	type PrecompilesValue = PrecompilesValue;
	type Timestamp = Timestamp;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = ();
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
	type OnCreate = ();
	type WeightInfo = ();
	type GasLimitPovSizeRatio = ConstU64<4>;
	type SuicideQuickClearLimit = ConstU32<0>;
}

parameter_types! {
	pub const VerifyPRuntime: bool = false;
	pub const VerifyRelaychainGenesisBlockHash: bool = false;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<1>;
	type WeightInfo = ();
}

pub fn alice() -> AccountId {
	U8Wrapper(1u8).into()
}

pub fn bob() -> AccountId {
	U8Wrapper(2u8).into()
}

pub fn new_test_ext(fast_round: bool) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	pallet_balances::GenesisConfig::<Test> { balances: vec![(alice(), 2 * UNIT)] }
		.assimilate_storage(&mut t)
		.unwrap();

	pallet_score_staking::GenesisConfig::<Test> {
		state: PoolState::Stopped,
		marker: Default::default(),
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext: sp_io::TestExternalities = t.into();
	ext.execute_with(|| {
		System::set_block_number(1);
		assert_ok!(ScoreStaking::set_score_feeder(RuntimeOrigin::root(), alice()));
		if fast_round {
			assert_ok!(ScoreStaking::set_round_config(
				RuntimeOrigin::root(),
				RoundSetting { interval: 5, stake_coef_n: 1, stake_coef_m: 2 }
			));
		}
	});
	ext
}

/// Run until a particular block.
pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		ScoreStaking::on_finalize(System::block_number());
		ParachainStaking::on_finalize(System::block_number());
		Balances::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Balances::on_initialize(System::block_number());
		ParachainStaking::on_initialize(System::block_number());
		ScoreStaking::on_initialize(System::block_number());
	}
}

pub struct IdentityAccountIdConvert;

impl AccountIdConvert<Test> for IdentityAccountIdConvert {
	fn convert(account: AccountId) -> <Test as frame_system::Config>::AccountId {
		account
	}
}
