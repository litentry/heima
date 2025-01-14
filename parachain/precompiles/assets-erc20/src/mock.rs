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

// Copyright 2019-2022 PureStake Inc.
// Copyright 2022      Stake Technologies
// This file is part of AssetsERC20 package, originally developed by Purestake Inc.
// AssetsERC20 package used in Litentry Network in terms of GPLv3.
//
// AssetsERC20 is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// AssetsERC20 is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with AssetsERC20.  If not, see <http://www.gnu.org/licenses/>.
//! Testing utilities.

use super::*;

use frame_support::{
	construct_runtime, derive_impl, parameter_types, traits::AsEnsureOriginWithArg, weights::Weight,
};

use frame_system::EnsureRoot;
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use precompile_utils::{
	mock_account,
	testing::{AddressInPrefixedSet, MockAccount},
};

use sp_core::{ConstU32, ConstU64, H160};
use sp_runtime::traits::IdentityLookup;
use sp_runtime::BuildStorage;

pub type AccountId = MockAccount;
pub type AssetId = u128;
pub type Balance = u128;

/// The local asset precompile address prefix. Addresses that match against this prefix will
/// be routed to Erc20AssetsPrecompileSet being marked as local
pub const ASSET_PRECOMPILE_ADDRESS_PREFIX: u32 = 0xfffffffe;

mock_account!(LocalAssetId(AssetId), |value: LocalAssetId| {
	AddressInPrefixedSet(ASSET_PRECOMPILE_ADDRESS_PREFIX, value.0).into()
});

// Implement the trait, where we convert AccountId to AssetID
impl AddressToAssetId<AssetId> for Runtime {
	/// The way to convert an account to assetId is by ensuring that the prefix is 0XFFFFFFFF
	/// and by taking the lowest 128 bits as the assetId
	fn address_to_asset_id(address: H160) -> Option<AssetId> {
		let address: MockAccount = address.into();
		if address.has_prefix_u32(ASSET_PRECOMPILE_ADDRESS_PREFIX) {
			Some(address.without_prefix())
		} else {
			None
		}
	}

	fn asset_id_to_address(asset_id: AssetId) -> H160 {
		LocalAssetId(asset_id).into()
	}
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountId = AccountId;
	type Block = frame_system::mocking::MockBlock<Runtime>;
	type AccountData = pallet_balances::AccountData<Balance>;
	type Lookup = IdentityLookup<Self::AccountId>;
}

parameter_types! {
	pub const ExistentialDeposit: Balance = 1;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const PrecompilesValue: Erc20AssetsPrecompileSet<Runtime> =
		Erc20AssetsPrecompileSet(PhantomData);
	pub WeightPerGas: Weight = Weight::from_parts(1, 0);
}

pub type PrecompileCall = Erc20AssetsPrecompileSetCall<Runtime, ()>;

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = AccountId;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = Erc20AssetsPrecompileSet<Self>;
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

// These parameters dont matter much as this will only be called by root with the forced arguments
// No deposit is substracted with those methods
parameter_types! {
	pub const AssetDeposit: Balance = 0;
	pub const AssetAccountDeposit: Balance = 0;
	pub const ApprovalDeposit: Balance = 0;
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 0;
	pub const MetadataDepositPerByte: Balance = 0;
}

impl pallet_assets::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = AssetAccountDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = AssetsStringLimit;
	type Freezer = ();
	type Extra = ();
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
	type RemoveItemsLimit = ConstU32<0>;
	type AssetIdParameter = AssetId;
	type CallbackHandle = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Runtime
	{
		System: frame_system,
		Balances: pallet_balances,
		Assets: pallet_assets,
		Evm: pallet_evm,
		Timestamp: pallet_timestamp,
	}
);

#[derive(Default)]
pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
}

impl ExtBuilder {
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

		pallet_balances::GenesisConfig::<Runtime> { balances: self.balances }
			.assimilate_storage(&mut t)
			.expect("Pallet balances storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
