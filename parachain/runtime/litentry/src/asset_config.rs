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

use super::{
	weights, AccountId, AssetId, Balance, Balances, EnsureRootOrAllTechnicalCommittee, Runtime,
	RuntimeEvent,
};
use crate::{
	constants::currency::deposit, precompiles::ASSET_PRECOMPILE_ADDRESS_PREFIX, Decode, Encode,
};
use frame_support::{
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU32, NeverEnsureOrigin},
};
use pallet_evm_precompile_assets_erc20::AddressToAssetId;
use parity_scale_codec::Compact;
use runtime_common::currency::{DOLLARS, EXISTENTIAL_DEPOSIT};
use scale_info::TypeInfo;
use sp_core::{ConstU128, H160};
use sp_std::prelude::*;

pub fn get_all_module_accounts() -> Vec<AccountId> {
	// Add whitelist here, usually this is the system account like treasury
	vec![]
}

pub struct AssetsBenchmarkHelper;
#[cfg(feature = "runtime-benchmarks")]
impl<AssetIdParameter: From<u128>> pallet_assets::BenchmarkHelper<AssetIdParameter>
	for AssetsBenchmarkHelper
{
	fn create_asset_id_parameter(id: u32) -> AssetIdParameter {
		AssetId::from(id).into()
	}
}

parameter_types! {
	pub const AssetDeposit: Balance = 1 * DOLLARS;
	pub const AssetsStringLimit: u32 = 50;
	/// Key = 32 bytes, Value = 36 bytes (32+1+1+1+1)
	// https://github.com/paritytech/substrate/blob/069917b/frame/assets/src/lib.rs#L257L271
	pub const MetadataDepositBase: Balance = deposit(1, 68);
	pub const MetadataDepositPerByte: Balance = deposit(0, 1);
	pub const AssetAccountDeposit: Balance = deposit(1, 18);
}

impl AddressToAssetId<AssetId> for Runtime {
	fn address_to_asset_id(address: H160) -> Option<AssetId> {
		let mut data = [0u8; 16];
		let address_bytes: [u8; 20] = address.into();
		if ASSET_PRECOMPILE_ADDRESS_PREFIX.eq(&address_bytes[0..4]) {
			data.copy_from_slice(&address_bytes[4..20]);
			Some(u128::from_be_bytes(data))
		} else {
			None
		}
	}

	fn asset_id_to_address(asset_id: AssetId) -> H160 {
		let mut data = [0u8; 20];
		data[0..4].copy_from_slice(ASSET_PRECOMPILE_ADDRESS_PREFIX);
		data[4..20].copy_from_slice(&asset_id.to_be_bytes());
		H160::from(data)
	}
}

impl pallet_assets::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<NeverEnsureOrigin<AccountId>>; // will be created internally for now
	type ForceOrigin = EnsureRootOrAllTechnicalCommittee;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type AssetAccountDeposit = AssetAccountDeposit;
	type ApprovalDeposit = ConstU128<{ EXISTENTIAL_DEPOSIT }>;
	type StringLimit = AssetsStringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = ();
	type RemoveItemsLimit = ConstU32<1000>;
	type AssetIdParameter = Compact<AssetId>;
	type CallbackHandle = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = AssetsBenchmarkHelper;
}

#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum ForeignAssetType {
	Xcm(xcm::v4::Location),
}

impl pallet_asset_manager::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type ForeignAssetType = ForeignAssetType; // TODO
	type ForeignAssetModifierOrigin = EnsureRootOrAllTechnicalCommittee;
	type Currency = Balances;
	type WeightInfo = weights::pallet_asset_manager::WeightInfo<Runtime>;
}
