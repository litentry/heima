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
use crate::{self as pallet_teebag, OperationalMode};
use frame_support::{
	assert_ok, construct_runtime, derive_impl, parameter_types,
	traits::{OnFinalize, OnInitialize},
};
use frame_system::EnsureRoot;
use sp_core::{ConstU32, H256};
use sp_keyring::AccountKeyring;
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage,
};

pub type Signature = sp_runtime::MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;

pub type SignedExtra = (
	frame_system::CheckSpecVersion<Test>,
	frame_system::CheckTxVersion<Test>,
	frame_system::CheckGenesis<Test>,
	frame_system::CheckEra<Test>,
	frame_system::CheckNonce<Test>,
	frame_system::CheckWeight<Test>,
);

construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		Timestamp: pallet_timestamp,
		Teebag: pallet_teebag,
		Utility: pallet_utility,
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

pub type Balance = u64;

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
	type MaxEnclaveIdentifier = ConstU32<2>;
	type MaxAuthorizedEnclave = ConstU32<2>;
	type WeightInfo = ();
}

impl pallet_utility::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup. RA from enclave compiled in debug mode is allowed
pub fn new_test_ext(is_dev_mode: bool) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(AccountKeyring::Alice.to_account_id(), 1 << 60)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut genesis_config: pallet_teebag::GenesisConfig<Test> = crate::GenesisConfig {
		allow_sgx_debug_mode: true,
		admin: Some(AccountKeyring::Alice.to_account_id()),
		mode: OperationalMode::Production,
	};

	if is_dev_mode {
		genesis_config.mode = OperationalMode::Development;
	}

	genesis_config.assimilate_storage(&mut t).unwrap();

	let mut ext: sp_io::TestExternalities = t.into();
	ext.execute_with(|| {
		System::set_block_number(1);
	});
	ext
}

/// Helper method for the OnTimestampSet to be called
pub fn set_timestamp(t: u64) {
	let _ = pallet_timestamp::Pallet::<Test>::set(RuntimeOrigin::none(), t);
}

/// Run until a particular block.
pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			System::on_finalize(System::block_number());
		}
		Timestamp::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
	}
}
