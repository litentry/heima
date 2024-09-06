use crate::{self as pallet_halving_mint, Instance1, OnTokenMinted};
use frame_support::pallet_prelude::*;
use frame_support::{construct_runtime, parameter_types, PalletId};
use sp_core::{ConstU32, ConstU64, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		HalvingMint: pallet_halving_mint::<Instance1>,
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
}
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type Block = frame_system::mocking::MockBlock<Test>;
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 2;
	pub const MaxLocks: u32 = 10;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type Balance = u64;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = ();
	type ReserveIdentifier = ();
	type FreezeIdentifier = ();
	type MaxHolds = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
}

parameter_types! {
	pub const BeneficiaryId: PalletId = PalletId(*b"lty/hlvm");
}

impl pallet_halving_mint::Config<Instance1> for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ManagerOrigin = frame_system::EnsureRoot<u64>;
	type TotalIssuance = ConstU64<1000>;
	type HalvingInterval = ConstU32<10>;
	type BeneficiaryId = BeneficiaryId;
	type OnTokenMinted = TransferOnTokenMinted<Test>;
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

pub struct TransferOnTokenMinted<T>(sp_std::marker::PhantomData<T>);

impl<T> OnTokenMinted<T::AccountId, T::Balance> for TransferOnTokenMinted<T>
where
	T: frame_system::Config<AccountId = u64> + pallet_balances::Config<Balance = u64>,
{
	fn token_minted(beneficiary: T::AccountId, amount: T::Balance) -> Weight {
		let _ = Balances::transfer(RuntimeOrigin::signed(beneficiary), 1, amount);
		Weight::zero()
	}
}
