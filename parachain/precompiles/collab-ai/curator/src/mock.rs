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
	construct_runtime, derive_impl, parameter_types,
	traits::{ConstU32, ConstU64},
};
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use precompile_utils::precompile_set::{AddressU64, PrecompileAt, PrecompileSetBuilder};
use sp_core::H160;
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};

pub type Balance = u128;
pub type AccountId = AccountId32;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test {
		System: frame_system,
		Balances: pallet_balances,
		Evm: pallet_evm,
		Curator: pallet_curator,
		Timestamp: pallet_timestamp,
	}
);

parameter_types! {
	pub const MinimumCuratorDeposit: Balance = 10;
}

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

// Implement pallet_curator config trait for mock runtime.
impl pallet_curator::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MinimumCuratorDeposit = MinimumCuratorDeposit;
	type CuratorJudgeOrigin = frame_system::EnsureRoot<Self::AccountId>;
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
	type PrecompilesType = CuratorMockPrecompile<Self>;
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
	type SuicideQuickClearLimit = ConstU32<0>;
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

pub type CuratorMockPrecompile<R> =
	PrecompileSetBuilder<R, (PrecompileAt<AddressU64<1000>, CuratorPrecompile<R>>,)>;

parameter_types! {
	pub PrecompilesValue: CuratorMockPrecompile<Test> = CuratorMockPrecompile::new();
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
