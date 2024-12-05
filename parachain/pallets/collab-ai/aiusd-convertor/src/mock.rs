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

use crate as pallet_aiusd;
use frame_support::{
	assert_ok, construct_runtime, parameter_types,
	traits::{
		tokens::fungibles::{Inspect, Mutate},
		AsEnsureOriginWithArg, ConstU128, ConstU16, ConstU32, Everything,
	},
};
use sp_core::{Get, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	AccountId32, BuildStorage,
};

pub type Signature = sp_runtime::MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

pub type Balance = u128;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test {
		System: frame_system,
		Assets: pallet_assets,
		Balances: pallet_balances,
		AIUSD: pallet_aiusd,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const AIUSDAssetId: u32 = 1;
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
	type AssetId = u32;
	type AssetIdParameter = u32;
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

pub struct ConvertingPool;
impl Get<AccountId32> for ConvertingPool {
	fn get() -> AccountId32 {
		AccountId32::new([1u8; 32])
	}
}

impl pallet_aiusd::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type ConvertingPool = ConvertingPool;
	type AIUSDAssetId = AIUSDAssetId;
	type ManagerOrigin = frame_system::EnsureRoot<<Test as frame_system::Config>::AccountId>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);

		let owner = AccountId32::from([2u8; 32]);
		let origin = RuntimeOrigin::root();

		// Create the AIUSD asset
		assert_ok!(pallet_assets::Pallet::<Test>::force_create(
			origin.clone(),
			1, // AIUSD asset id
			owner.clone(),
			true,
			1,
		));
		// Create the target asset
		let source_asset_id = 2;
		assert_ok!(pallet_assets::Pallet::<Test>::force_create(
			origin,
			source_asset_id,
			owner.clone(),
			true,
			1,
		));

		// Check if these assets exists
		assert!(pallet_aiusd::InspectFungibles::<Test>::asset_exists(1));
		assert!(pallet_aiusd::InspectFungibles::<Test>::asset_exists(2));

		// Set total supply
		assert_ok!(pallet_aiusd::InspectFungibles::<Test>::mint_into(
			source_asset_id,
			&owner,
			1_000_000_000 // 1000 (10^6 * 1000)
		));
	});
	ext
}
