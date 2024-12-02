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

use super::*;
use crate as pallet_asset_manager;
use parity_scale_codec::{Decode, Encode};

use frame_support::{construct_runtime, derive_impl, parameter_types};
use frame_system::EnsureRoot;
use scale_info::TypeInfo;
use sp_core::{RuntimeDebug, H256};
use sp_runtime::{traits::Hash as THash, BuildStorage};
use xcm::latest::prelude::*;

pub type AccountId = u64;
pub type Balance = u128;

construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		AssetManager: pallet_asset_manager,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
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

pub type AssetId = u32;
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum MockAssetType {
	#[codec(index = 0)]
	MockAsset(AssetId),
	#[codec(index = 1)]
	Xcm(Box<Location>),
}

impl Default for MockAssetType {
	fn default() -> Self {
		Self::MockAsset(0)
	}
}

impl From<MockAssetType> for AssetId {
	fn from(asset: MockAssetType) -> AssetId {
		match asset {
			MockAssetType::MockAsset(id) => id,
			MockAssetType::Xcm(id) => {
				let mut result: [u8; 4] = [0u8; 4];
				let hash: H256 = (*id).using_encoded(<Test as frame_system::Config>::Hashing::hash);
				result.copy_from_slice(&hash.as_fixed_bytes()[0..4]);
				u32::from_le_bytes(result)
			},
		}
	}
}

impl From<Option<Location>> for MockAssetType {
	fn from(location: Option<Location>) -> Self {
		match location {
			None => Self::Xcm(Box::default()),
			Some(multi) => Self::Xcm(Box::new(multi)),
		}
	}
}

impl From<MockAssetType> for Option<Location> {
	fn from(asset: MockAssetType) -> Option<Location> {
		match asset {
			MockAssetType::Xcm(location) => Some(*location),
			_ => None,
		}
	}
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u64;
	type AssetId = u32;
	type ForeignAssetType = MockAssetType;
	type ForeignAssetModifierOrigin = EnsureRoot<u64>;
	type Currency = Balances;
	type WeightInfo = ();
}

#[derive(Default)]
pub(crate) struct ExtBuilder {}

impl ExtBuilder {
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub(crate) fn events() -> Vec<super::Event<Test>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| if let RuntimeEvent::AssetManager(inner) = e { Some(inner) } else { None })
		.collect::<Vec<_>>()
}

pub fn expect_events(e: Vec<super::Event<Test>>) {
	assert_eq!(events(), e);
}
