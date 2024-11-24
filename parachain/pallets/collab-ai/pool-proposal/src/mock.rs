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

use crate as pallet_pool_proposal;
use frame_support::{
	assert_ok, construct_runtime, parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU128, ConstU16, ConstU32, Everything},
};
use sp_core::{Get, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	AccountId32, BuildStorage, DispatchResult,
};

use pallet_collab_ai_common::*;

pub type Signature = sp_runtime::MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

pub type Balance = u128;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test {
		System: frame_system,
		Assets: pallet_assets,
		Balances: pallet_balances,
		Multisig: pallet_multisig,
		PoolProposal: pallet_pool_proposal,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const AIUSDAssetId: u32 = 1;
	pub const OfficialGapPeriod: u32 = 10;
	pub const MinimumProposalLastTime: u32 = 10;
	pub const MinimumPoolDeposit: Balance = 100;
	pub const MaxGuardianPerProposal: u32 = 2;
	pub const MaxGuardianSelectedPerProposal: u32 = 1;
	pub const MaximumPoolProposed: u32 = 1;

	pub const DepositBase: Balance = 1;
	pub const DepositFactor: Balance = 1;

	pub const StandardEpoch: u32 = 10;
}

impl pallet_multisig::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = ConstU32<3>;
	type WeightInfo = ();
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type Block = frame_system::mocking::MockBlock<Test>;
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<31>;
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl pallet_assets::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = u128;
	type AssetIdParameter = u128;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<Self::AccountId>>;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type AssetDeposit = ConstU128<1>;
	type AssetAccountDeposit = ConstU128<10>;
	type MetadataDepositBase = ConstU128<1>;
	type MetadataDepositPerByte = ConstU128<1>;
	type ApprovalDeposit = ConstU128<1>;
	type StringLimit = ConstU32<50>;
	type Freezer = ();
	type WeightInfo = ();
	type CallbackHandle = ();
	type Extra = ();
	type RemoveItemsLimit = ConstU32<5>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = ();
	type ReserveIdentifier = ();
	type FreezeIdentifier = ();
	type MaxHolds = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
}

pub struct PreInvestingPool;
impl Get<AccountId32> for PreInvestingPool {
	fn get() -> AccountId32 {
		AccountId32::new([1u8; 32])
	}
}

pub struct MockCuratorQuery;
impl CuratorQuery<AccountId> for MockCuratorQuery {
	/// All curator but banned ones
	fn is_curator(_account: AccountId) -> bool {
		true
	}

	/// Only verified one
	fn is_verified_curator(_account: AccountId) -> bool {
		true
	}
}

pub struct MockGuardianQuery;
impl GuardianQuery<AccountId> for MockGuardianQuery {
	/// All guardian but banned ones
	fn is_guardian(_account: AccountId) -> bool {
		true
	}

	/// Only verified one
	fn is_verified_guardian(_account: AccountId) -> bool {
		true
	}

	/// Get vote
	fn get_vote(_voter: AccountId, _guardian: AccountId) -> Option<GuardianVote> {
		Some(GuardianVote::Aye)
	}
}

pub struct MockInvestmentInjector;
impl InvestmentInjector<AccountId, u64, Balance> for MockInvestmentInjector {
	fn create_investing_pool(
		_pool_id: InvestingPoolIndex,
		_setting: PoolSetting<AccountId, u64, Balance>,
		_admin: AccountId,
	) -> DispatchResult {
		Ok(())
	}
	fn inject_investment(
		_pool_id: InvestingPoolIndex,
		_investments: Vec<(AccountId, Balance)>,
	) -> DispatchResult {
		Ok(())
	}
}

impl pallet_pool_proposal::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type Fungibles = Assets;
	type AIUSDAssetId = AIUSDAssetId;
	type OfficialGapPeriod = OfficialGapPeriod;
	type MinimumProposalLastTime = MinimumProposalLastTime;
	type MinimumPoolDeposit = MinimumPoolDeposit;
	type MaximumPoolProposed = MaximumPoolProposed;
	type StandardEpoch = StandardEpoch;
	type ProposalOrigin = EnsureSignedAndCurator<Self::AccountId, MockCuratorQuery>;
	type PublicVotingOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type GuardianVoteResource = MockGuardianQuery;
	type MaxGuardianPerProposal = MaxGuardianPerProposal;
	type MaxGuardianSelectedPerProposal = MaxGuardianSelectedPerProposal;
	type PreInvestingPool = PreInvestingPool;
	type InvestmentInjector = MockInvestmentInjector;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);

		let owner = AccountId32::from([1u8; 32]);
		let origin = RuntimeOrigin::root();

		// Create the AIUSD asset
		assert_ok!(pallet_assets::Pallet::<Test>::force_create(
			origin.clone(),
			1, // AIUSD asset id
			owner.clone(),
			true,
			1,
		));

		// Set total supply
		assert_ok!(pallet_assets::Pallet::<Test>::mint(
			RuntimeOrigin::signed(owner.clone()),
			1, // AIUSD asset id
			owner.clone(),
			1_000_000_000_000_000_000_000_000, // 1 000 000 (10^18 * 1000)
		));
	});
	ext
}

// Checks events against the latest. A contiguous set of events must be provided. They must
// include the most recent event, but do not have to include every past event.
pub fn assert_events(mut expected: Vec<RuntimeEvent>) {
	let mut actual: Vec<RuntimeEvent> =
		frame_system::Pallet::<Test>::events().iter().map(|e| e.event.clone()).collect();

	expected.reverse();

	for evt in expected {
		let next = actual.pop().expect("event expected");
		assert_eq!(next, evt, "Events don't match");
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
