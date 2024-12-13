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

use crate::{self as pallet_halving_mint, Config, Instance1, OnTokenMinted};
use frame_support::pallet_prelude::*;
use frame_support::traits::tokens::{fungibles::Mutate, Preservation};
use frame_support::{
	construct_runtime, derive_impl, parameter_types, traits::AsEnsureOriginWithArg, PalletId,
};
use frame_system::{EnsureRoot, EnsureSigned};
use sp_core::{ConstU32, ConstU64};
use sp_runtime::BuildStorage;

type AccountId = u64;
type Balance = u64;
type AssetId = u64;

construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Assets: pallet_assets,
		Balances: pallet_balances,
		HalvingMint: pallet_halving_mint::<Instance1>,
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

parameter_types! {
	pub const AssetDeposit: Balance = 0;
	pub const AssetAccountDeposit: Balance = 0;
	pub const ApprovalDeposit: Balance = 0;
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 0;
	pub const MetadataDepositPerByte: Balance = 0;
}

impl pallet_assets::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = AssetAccountDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = AssetsStringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = ();
	type RemoveItemsLimit = ConstU32<0>;
	type AssetIdParameter = AssetId;
	type CallbackHandle = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

parameter_types! {
	pub const BeneficiaryId: PalletId = PalletId(*b"lty/hlvm");
	pub const TestAssetId: AssetId = 1;
}

impl pallet_halving_mint::Config<Instance1> for Test {
	type RuntimeEvent = RuntimeEvent;
	type AssetBalance = Balance;
	type AssetId = AssetId;
	type Assets = Assets;
	type ManagerOrigin = frame_system::EnsureRoot<u64>;
	type TotalIssuance = ConstU64<1000>;
	type HalvingInterval = ConstU32<10>;
	type BeneficiaryId = BeneficiaryId;
	type OnTokenMinted = TransferOnTokenMinted<Test, Instance1>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(HalvingMint::beneficiary_account(), 10)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	let mut ext: sp_io::TestExternalities = t.into();
	ext.execute_with(|| {
		System::set_block_number(1);
	});
	ext
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		HalvingMint::on_initialize(System::block_number());
	}
}

pub struct TransferOnTokenMinted<T, I: 'static = ()>(sp_std::marker::PhantomData<(T, I)>);

impl<T, I> OnTokenMinted<T::AssetId, T::AccountId, T::AssetBalance> for TransferOnTokenMinted<T, I>
where
	T: frame_system::Config<AccountId = AccountId> + Config<I>,
	T::Assets: Mutate<T::AccountId>,
{
	fn token_minted(
		asset_id: T::AssetId,
		beneficiary: T::AccountId,
		amount: T::AssetBalance,
	) -> Weight {
		let _ = T::Assets::transfer(asset_id, &beneficiary, &1, amount, Preservation::Expendable)
			.unwrap();
		Weight::zero()
	}
}
