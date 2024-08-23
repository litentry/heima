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

#![allow(dead_code, unused_imports, const_item_mutation)]
use crate::{
	self as pallet_score_staking, AccountIdConvert, Config, Perbill, PoolState, RoundSetting,
};
use frame_support::{
	assert_ok, construct_runtime, ord_parameter_types,
	pallet_prelude::GenesisBuild,
	parameter_types,
	traits::{OnFinalize, OnInitialize},
};
use frame_system::{EnsureRoot, EnsureSignedBy};
use sp_core::{ConstU128, ConstU32, H256};
use sp_keyring::AccountKeyring;
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	Percent,
};

pub type Signature = sp_runtime::MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;

pub type BlockNumber = u32;
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

pub type SignedExtra = (
	frame_system::CheckSpecVersion<Test>,
	frame_system::CheckTxVersion<Test>,
	frame_system::CheckGenesis<Test>,
	frame_system::CheckEra<Test>,
	frame_system::CheckNonce<Test>,
	frame_system::CheckWeight<Test>,
);

construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Balances: pallet_balances,
		ParachainStaking: pallet_parachain_staking,
		ScoreStaking: pallet_score_staking,
		Teebag: pallet_teebag,
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
}
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Index = u64;
	type RuntimeCall = RuntimeCall;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

pub type Balance = u128;
pub const UNIT: Balance = 1_000_000_000_000;

parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = ();
	type ReserveIdentifier = ();
	type HoldIdentifier = ();
	type FreezeIdentifier = ();
	type MaxHolds = ();
	type MaxFreezes = ();
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
	type OnAllDelegationRemoved = pallet_score_staking::Pallet<Test>;
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
	type TokenStakingAuthorizer = pallet_teebag::Pallet<Test>;
}

parameter_types! {
		pub const MinimumPeriod: u64 = 6000 / 2;
}

pub type Moment = u64;

impl pallet_timestamp::Config for Test {
	type Moment = Moment;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const MomentsPerDay: u64 = 86_400_000; // [ms/d]
}

impl pallet_teebag::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MomentsPerDay = MomentsPerDay;
	type SetAdminOrigin = EnsureRoot<Self::AccountId>;
	type MaxEnclaveIdentifier = ConstU32<1>;
	type MaxAuthorizedEnclave = ConstU32<2>;
	type WeightInfo = ();
}

impl pallet_score_staking::TokenStakingAuthorizer<Test> for pallet_teebag::Pallet<Test> {
	fn can_update_staking(sender: &<Test as frame_system::Config>::AccountId) -> bool {
		pallet_teebag::Pallet::<Test>::enclave_registry(sender).is_some()
	}
}

pub fn alice() -> AccountId {
	AccountKeyring::Alice.to_account_id()
}

pub fn bob() -> AccountId {
	AccountKeyring::Bob.to_account_id()
}

pub fn charlie() -> AccountId {
	AccountKeyring::Charlie.to_account_id()
}

pub fn new_test_ext(fast_round: bool) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> { balances: vec![(alice(), 2 * UNIT)] }
		.assimilate_storage(&mut t)
		.unwrap();

	let teebag = pallet_teebag::GenesisConfig::<Test> {
		allow_sgx_debug_mode: true,
		admin: Some(AccountKeyring::Alice.to_account_id()),
		mode: pallet_teebag::OperationalMode::Production,
	};
	teebag.assimilate_storage(&mut t).unwrap();

	let genesis_config: pallet_score_staking::GenesisConfig<Test> =
		crate::GenesisConfig { state: PoolState::Stopped, marker: Default::default() };

	GenesisBuild::<Test>::assimilate_storage(&genesis_config, &mut t).unwrap();

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

pub fn new_test_ext_with_parachain_staking() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.expect("Frame system builds valid default genesis config");

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(alice(), 2 * UNIT), (bob(), 10 * UNIT)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	pallet_parachain_staking::GenesisConfig::<Test> {
		candidates: vec![(alice(), 10)],
		delegations: vec![],
		inflation_config: pallet_parachain_staking::InflationInfo {
			expect: pallet_parachain_staking::Range { min: 700, ideal: 700, max: 700 },
			// not used
			annual: pallet_parachain_staking::Range {
				min: Perbill::from_percent(50),
				ideal: Perbill::from_percent(50),
				max: Perbill::from_percent(50),
			},
			// unrealistically high parameterization, only for testing
			round: pallet_parachain_staking::Range {
				min: Perbill::from_percent(5),
				ideal: Perbill::from_percent(5),
				max: Perbill::from_percent(5),
			},
		},
	}
	.assimilate_storage(&mut t)
	.expect("Parachain Staking's storage can be assimilated");

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
		assert_ok!(ScoreStaking::set_score_feeder(RuntimeOrigin::root(), alice()));
		assert_ok!(ScoreStaking::set_round_config(
			RuntimeOrigin::root(),
			RoundSetting { interval: 5, stake_coef_n: 1, stake_coef_m: 2 }
		));
	});

	ext
}

/// Run until a particular block.
pub fn run_to_block(n: u32) {
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
