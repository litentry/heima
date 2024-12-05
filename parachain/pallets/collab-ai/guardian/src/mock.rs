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

use crate as pallet_guardian;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU16, ConstU32, Everything},
};
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};
// use sp_io::TestExternalities;
use frame_support::traits::ConstU128;
use sp_runtime::BuildStorage;

// Define mock runtime types
pub type Balance = u128;
pub type AccountId = AccountId32;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test {
		System: frame_system,
		Balances: pallet_balances,
		Guardian: pallet_guardian,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MinimumGuardianDeposit: Balance = 10;
}

// Implement frame_system config trait for mock runtime.
impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	// type BlockNumber = u64;  // Add this
	type Hash = H256;
	type Block = frame_system::mocking::MockBlock<Test>; // Add this
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	// type Header = sp_runtime::generic::Header<Self::BlockNumber, BlakeTwo256>;  // Add this
	type BlockHashCount = BlockHashCount;
	type Version = (); // Add this
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<31>;
	type OnSetCode = (); // Add this
	type MaxConsumers = ConstU32<16>; // Add this
}

// Implement pallet_balances config trait for mock runtime.
impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
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

// Helper function to initialize the test environment.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(AccountId32::from([1u8; 32]), 100), (AccountId32::from([2u8; 32]), 100)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
