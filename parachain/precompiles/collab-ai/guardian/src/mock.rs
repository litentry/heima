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

use crate::*;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU128, ConstU16, ConstU32, ConstU64, Everything},
};
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use precompile_utils::precompile_set::{AddressU64, PrecompileAt, PrecompileSetBuilder};
use sp_core::{H160, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, BuildStorage,
};

pub type Balance = u128;
pub type AccountId = AccountId32;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test {
		System: frame_system,
		Balances: pallet_balances,
		Evm: pallet_evm,
		Guardian: pallet_guardian,
		Timestamp: pallet_timestamp,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MinimumGuardianDeposit: Balance = 10;
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Block = frame_system::mocking::MockBlock<Test>; // Add this
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

// Implement pallet_guardian config trait for mock runtime.
impl pallet_guardian::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MinimumGuardianDeposit = MinimumGuardianDeposit;
	type GuardianJudgeOrigin = frame_system::EnsureRoot<Self::AccountId>;
}

// Setup EVM configuration
parameter_types! {
	pub WeightPerGas: u64 = 1;
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
	type PrecompilesType = GuardianMockPrecompile<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ();
	type Timestamp = Timestamp;
	type OnChargeTransaction = ();
	type BlockGasLimit = ();
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
	type OnCreate = ();
	type WeightInfo = ();
	type GasLimitPovSizeRatio = ConstU64<4>;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<1>;
	type WeightInfo = ();
}

pub struct TruncatedAddressMapping;
impl pallet_evm::AddressMapping<AccountId> for TruncatedAddressMapping {
	fn into_account_id(address: H160) -> AccountId {
		let mut data = [0u8; 32];
		data[0..20].copy_from_slice(&address[..]);
		AccountId::from(Into::<[u8; 32]>::into(data))
	}
}

pub type GuardianMockPrecompile<R> =
	PrecompileSetBuilder<R, (PrecompileAt<AddressU64<1000>, GuardianPrecompile<R>>,)>;

parameter_types! {
	pub PrecompilesValue: GuardianMockPrecompile<Test> = GuardianMockPrecompile::new();
}

// Helper function to initialize the test environment.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	// Add initial balances for the involved accounts
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(TruncatedAddressMapping::into_account_id(H160::from_low_u64_be(1000)), 1_000_000),
			(TruncatedAddressMapping::into_account_id(H160::from_low_u64_be(1001)), 1_000_000),
			(TruncatedAddressMapping::into_account_id(H160::from_low_u64_be(1002)), 1_000_000),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
