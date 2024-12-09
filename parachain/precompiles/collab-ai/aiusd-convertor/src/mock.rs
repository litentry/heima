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
	assert_ok, construct_runtime, derive_impl,
	pallet_prelude::Weight,
	parameter_types,
	traits::{
		tokens::fungibles::{Inspect, Mutate},
		AsEnsureOriginWithArg, ConstU128, ConstU32, ConstU64,
	},
};
use pallet_aiusd_convertor as pallet_aiusd;
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use precompile_utils::precompile_set::{AddressU64, PrecompileAt, PrecompileSetBuilder};
use sp_core::{Get, H160};
use sp_runtime::{
	traits::{IdentifyAccount, IdentityLookup, Verify},
	AccountId32, BuildStorage,
};

pub type Signature = sp_runtime::MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

pub type Balance = u128;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test {
		System: frame_system,
		Timestamp: pallet_timestamp,
		Assets: pallet_assets,
		Balances: pallet_balances,
		Evm: pallet_evm,
		AIUSD: pallet_aiusd,
	}
);

parameter_types! {
	pub const AIUSDAssetId: u32 = 1;
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

pub struct ConvertingPool;
impl Get<AccountId32> for ConvertingPool {
	fn get() -> AccountId32 {
		let h160_address: H160 = H160::from_low_u64_be(1000);
		TruncatedAddressMapping::into_account_id(h160_address)
	}
}

impl pallet_aiusd::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type ConvertingPool = ConvertingPool;
	type AIUSDAssetId = AIUSDAssetId;
	type ManagerOrigin = frame_system::EnsureRoot<<Test as frame_system::Config>::AccountId>;
}

pub fn precompile_address() -> H160 {
	H160::from_low_u64_be(20480 + 115)
}

pub type AIUSDConvertorMockPrecompile<R> =
	PrecompileSetBuilder<R, (PrecompileAt<AddressU64<20595>, AIUSDConvertorPrecompile<R>>,)>;

pub type PCall<Runtime> = AIUSDConvertorPrecompileCall<Runtime>;

pub struct TruncatedAddressMapping;
impl AddressMapping<AccountId> for TruncatedAddressMapping {
	fn into_account_id(address: H160) -> AccountId {
		let mut data = [0u8; 32];
		data[0..20].copy_from_slice(&address[..]);
		AccountId::from(Into::<[u8; 32]>::into(data))
	}
}

parameter_types! {
	pub PrecompilesValue: AIUSDConvertorMockPrecompile<Test> = AIUSDConvertorMockPrecompile::new();
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
	type PrecompilesType = AIUSDConvertorMockPrecompile<Self>;
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

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);

		let h160_address: H160 = H160::from_low_u64_be(1001);
		let owner = TruncatedAddressMapping::into_account_id(h160_address);
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

		// Enable assert
		assert_ok!(AIUSD::enable_token(RuntimeOrigin::root(), source_asset_id, 1_000_000));
	});
	ext
}
