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

use crate as pallet_extrinsic_filter;
use frame_support::{
	derive_impl, parameter_types,
	traits::{ConstU64, Contains, Everything},
};
use frame_system::EnsureRoot;
use sp_runtime::BuildStorage;

pub type Balance = u128;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		ExtrinsicFilter: pallet_extrinsic_filter,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type BaseCallFilter = ExtrinsicFilter;
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

pub struct SafeModeFilter;
impl Contains<RuntimeCall> for SafeModeFilter {
	fn contains(call: &RuntimeCall) -> bool {
		matches!(call, RuntimeCall::System(_) | RuntimeCall::ExtrinsicFilter(_))
	}
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<10000>;
	type WeightInfo = ();
}

pub struct NormalModeFilter;
impl Contains<RuntimeCall> for NormalModeFilter {
	fn contains(call: &RuntimeCall) -> bool {
		matches!(
			call,
			RuntimeCall::System(_) | RuntimeCall::ExtrinsicFilter(_) | RuntimeCall::Timestamp(_)
		)
	}
}

impl pallet_extrinsic_filter::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type UpdateOrigin = EnsureRoot<Self::AccountId>;
	type SafeModeFilter = SafeModeFilter;
	type NormalModeFilter = NormalModeFilter;
	type TestModeFilter = Everything;
	type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	pallet_balances::GenesisConfig::<Test> { balances: vec![(1, 100)] }
		.assimilate_storage(&mut t)
		.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
	});
	ext
}
