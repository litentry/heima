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

use crate::{
	self as pallet_omni_bridge, ChainAsset, ChainType, Nonce, PayInRequest, PayOutRequest,
	ResourceId,
};
pub use frame_support::{
	assert_ok, derive_impl, parameter_types,
	traits::{
		fungible::{self, NativeFromLeft, NativeOrWithId},
		AsEnsureOriginWithArg, ConstU32, ConstU64,
	},
};
use sp_keyring::AccountKeyring;
use sp_runtime::{
	traits::{IdentifyAccount, IdentityLookup, Verify},
	BuildStorage,
};

pub type Signature = sp_runtime::MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Balance = u64;
pub type AssetId = u32;

pub const TEST_ASSET: AssetId = 1;

pub fn alice() -> AccountId {
	AccountKeyring::Alice.to_account_id()
}

pub fn bob() -> AccountId {
	AccountKeyring::Bob.to_account_id()
}

pub fn charlie() -> AccountId {
	AccountKeyring::Charlie.to_account_id()
}

pub fn dave() -> AccountId {
	AccountKeyring::Dave.to_account_id()
}

pub fn native_pay_in_request() -> PayInRequest<NativeOrWithId<AssetId>, Balance> {
	new_pay_in_request(NativeOrWithId::Native, 10)
}

pub fn asset_pay_in_request() -> PayInRequest<NativeOrWithId<AssetId>, Balance> {
	new_pay_in_request(NativeOrWithId::WithId(TEST_ASSET), 10)
}

pub fn new_chain_asset(asset: NativeOrWithId<AssetId>) -> ChainAsset<NativeOrWithId<AssetId>> {
	ChainAsset { chain: ChainType::Heima, asset }
}

pub fn native_resource_id() -> ResourceId {
	new_chain_asset(NativeOrWithId::Native).to_resource_id()
}

pub fn asset_resource_id() -> ResourceId {
	new_chain_asset(NativeOrWithId::WithId(TEST_ASSET)).to_resource_id()
}

pub fn native_pay_out_request(nonce: Nonce) -> PayOutRequest<AccountId, Balance> {
	new_pay_out_request(nonce, NativeOrWithId::Native, 10)
}

pub fn asset_pay_out_request(nonce: Nonce) -> PayOutRequest<AccountId, Balance> {
	new_pay_out_request(nonce, NativeOrWithId::WithId(TEST_ASSET), 10)
}

frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		Timestamp: pallet_timestamp,
		Assets: pallet_assets,
		OmniBridge: pallet_omni_bridge,
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

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<10000>;
	type WeightInfo = ();
}

impl pallet_assets::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type AssetIdParameter = AssetId;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<Self::AccountId>>;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type AssetDeposit = ConstU64<1>;
	type AssetAccountDeposit = ConstU64<10>;
	type MetadataDepositBase = ConstU64<1>;
	type MetadataDepositPerByte = ConstU64<1>;
	type ApprovalDeposit = ConstU64<1>;
	type StringLimit = ConstU32<50>;
	type Freezer = ();
	type WeightInfo = ();
	type CallbackHandle = ();
	type Extra = ();
	type RemoveItemsLimit = ConstU32<5>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

parameter_types! {
	pub TreasuryAccount: AccountId = dave(); // Dave as the treasury
}

// Keep it same as real runtime
impl pallet_omni_bridge::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetKind = NativeOrWithId<AssetId>;
	type Assets =
		fungible::UnionOf<Balances, Assets, NativeFromLeft, NativeOrWithId<AssetId>, AccountId>;
	type TreasuryAccount = TreasuryAccount;
	type SetAdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
}

pub fn new_test_ext(should_init: bool) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(alice(), 50), (bob(), 50), (charlie(), 50)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext: sp_io::TestExternalities = t.into();
	ext.execute_with(|| {
		System::set_block_number(1);
		assert_ok!(OmniBridge::set_admin(RuntimeOrigin::root(), alice()));
		assert_ok!(OmniBridge::add_relayer(RuntimeOrigin::signed(alice()), alice()));
		assert_ok!(OmniBridge::add_relayer(RuntimeOrigin::signed(alice()), bob()));
		assert_ok!(OmniBridge::add_relayer(RuntimeOrigin::signed(alice()), charlie()));
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), TEST_ASSET, bob(), true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(bob()), TEST_ASSET, alice(), 100));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(bob()), TEST_ASSET, bob(), 100));

		if should_init {
			assert_ok!(OmniBridge::set_resource_id(
				RuntimeOrigin::signed(alice()),
				new_chain_asset(NativeOrWithId::Native).to_resource_id(),
				new_chain_asset(NativeOrWithId::Native)
			));
			assert_ok!(OmniBridge::add_pay_in_pair(
				RuntimeOrigin::signed(alice()),
				NativeOrWithId::Native,
				ChainType::Ethereum(0)
			));
			assert_ok!(OmniBridge::set_pay_in_fee(
				RuntimeOrigin::signed(alice()),
				NativeOrWithId::Native,
				ChainType::Ethereum(0),
				2
			));
			assert_ok!(OmniBridge::set_resource_id(
				RuntimeOrigin::signed(alice()),
				new_chain_asset(NativeOrWithId::WithId(TEST_ASSET)).to_resource_id(),
				new_chain_asset(NativeOrWithId::WithId(TEST_ASSET))
			));
			assert_ok!(OmniBridge::add_pay_in_pair(
				RuntimeOrigin::signed(alice()),
				NativeOrWithId::WithId(TEST_ASSET),
				ChainType::Ethereum(0)
			));
			assert_ok!(OmniBridge::set_pay_in_fee(
				RuntimeOrigin::signed(alice()),
				NativeOrWithId::WithId(TEST_ASSET),
				ChainType::Ethereum(0),
				3
			));
		}
	});
	ext
}

pub fn new_pay_in_request(
	asset: NativeOrWithId<AssetId>,
	amount: Balance,
) -> PayInRequest<NativeOrWithId<AssetId>, Balance> {
	PayInRequest {
		asset,
		dest_chain: ChainType::Ethereum(0),
		dest_account: [1u8; 20].to_vec(),
		amount,
	}
}

pub fn new_pay_out_request(
	nonce: Nonce,
	asset: NativeOrWithId<AssetId>,
	amount: Balance,
) -> PayOutRequest<AccountId, Balance> {
	PayOutRequest {
		source_chain: ChainType::Ethereum(0),
		nonce,
		resource_id: new_chain_asset(asset).to_resource_id(),
		dest_account: alice(),
		amount,
	}
}
