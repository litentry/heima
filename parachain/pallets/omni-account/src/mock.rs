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

use crate::{self as pallet_omni_account, Encode, EnsureOmniAccount};
use core_primitives::{DefaultOmniAccountConverter, Identity, MemberAccount};
use frame_support::{
	assert_ok, derive_impl,
	pallet_prelude::EnsureOrigin,
	parameter_types,
	traits::{ConstU32, ConstU64},
};
use frame_system::EnsureRoot;
pub use pallet_teebag::test_util::get_signer;
use pallet_teebag::test_util::{TEST8_CERT, TEST8_SIGNER_PUB, TEST8_TIMESTAMP, URL};
use sp_keyring::AccountKeyring;
use sp_runtime::{
	traits::{IdentifyAccount, IdentityLookup, Verify},
	BuildStorage,
};
use sp_std::marker::PhantomData;

pub type Signature = sp_runtime::MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Balance = u64;
pub type SystemAccountId = <Test as frame_system::Config>::AccountId;

pub struct EnsureEnclaveSigner<T>(PhantomData<T>);
impl<T> EnsureOrigin<T::RuntimeOrigin> for EnsureEnclaveSigner<T>
where
	T: frame_system::Config + pallet_teebag::Config + pallet_timestamp::Config<Moment = u64>,
	<T as frame_system::Config>::AccountId: From<[u8; 32]>,
	<T as frame_system::Config>::Hash: From<[u8; 32]>,
{
	type Success = T::AccountId;
	fn try_origin(o: T::RuntimeOrigin) -> Result<Self::Success, T::RuntimeOrigin> {
		o.into().and_then(|o| match o {
			frame_system::RawOrigin::Signed(who)
				if pallet_teebag::EnclaveRegistry::<T>::contains_key(&who) =>
			{
				Ok(who)
			},
			r => Err(T::RuntimeOrigin::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<T::RuntimeOrigin, ()> {
		use pallet_teebag::test_util::{get_signer, TEST8_MRENCLAVE, TEST8_SIGNER_PUB};
		let signer: <T as frame_system::Config>::AccountId = get_signer(TEST8_SIGNER_PUB);
		let enclave = core_primitives::Enclave::default().with_mrenclave(TEST8_MRENCLAVE);
		let _ = pallet_teebag::Pallet::<T>::add_enclave_identifier_internal(
			enclave.worker_type,
			&signer,
		);
		if !pallet_teebag::EnclaveRegistry::<T>::contains_key(signer.clone()) {
			assert_ok!(pallet_teebag::Pallet::<T>::add_enclave(&signer, &enclave,));
		}
		Ok(frame_system::RawOrigin::Signed(signer).into())
	}
}

pub struct Accounts {
	pub native_account: AccountId,
	pub omni_account: AccountId,
	pub identity: Identity,
}

fn create_accounts(keyring: AccountKeyring) -> Accounts {
	let native_account = keyring.to_account_id();
	let identity = Identity::from(native_account.clone());
	Accounts { native_account, omni_account: identity.to_omni_account(), identity }
}

pub fn alice() -> Accounts {
	create_accounts(AccountKeyring::Alice)
}

pub fn bob() -> Accounts {
	create_accounts(AccountKeyring::Bob)
}

pub fn charlie() -> Accounts {
	create_accounts(AccountKeyring::Charlie)
}

pub fn public_member_account(accounts: Accounts) -> MemberAccount {
	MemberAccount::Public(accounts.identity)
}

pub fn private_member_account(accounts: Accounts) -> MemberAccount {
	MemberAccount::Private(accounts.identity.encode(), accounts.identity.hash())
}

frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		Teebag: pallet_teebag,
		Timestamp: pallet_timestamp,
		Utility: pallet_utility,
		OmniAccount: pallet_omni_account,
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

impl pallet_utility::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}

impl pallet_teebag::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MomentsPerDay = ConstU64<86_400_000>; // [ms/d]
	type SetAdminOrigin = EnsureRoot<Self::AccountId>;
	type MaxEnclaveIdentifier = ConstU32<3>;
	type MaxAuthorizedEnclave = ConstU32<3>;
	type WeightInfo = ();
}

impl pallet_omni_account::Config for Test {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type TEECallOrigin = EnsureEnclaveSigner<Self>;
	type MaxAccountStoreLength = ConstU32<3>;
	type OmniAccountOrigin = EnsureOmniAccount<Self::AccountId>;
	type OmniAccountConverter = DefaultOmniAccountConverter;
}

pub fn get_tee_signer() -> SystemAccountId {
	get_signer(TEST8_SIGNER_PUB)
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	pallet_balances::GenesisConfig::<Test> { balances: vec![(alice().native_account, 10)] }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext: sp_io::TestExternalities = t.into();
	ext.execute_with(|| {
		System::set_block_number(1);
		let signer = get_tee_signer();
		assert_ok!(Teebag::set_admin(RuntimeOrigin::root(), signer.clone()));
		assert_ok!(Teebag::set_mode(
			RuntimeOrigin::signed(signer.clone()),
			core_primitives::OperationalMode::Development
		));

		Timestamp::set_timestamp(TEST8_TIMESTAMP);
		if !pallet_teebag::EnclaveRegistry::<Test>::contains_key(signer.clone()) {
			assert_ok!(Teebag::register_enclave(
				RuntimeOrigin::signed(signer),
				core_primitives::WorkerType::Identity,
				core_primitives::WorkerMode::Sidechain,
				TEST8_CERT.to_vec(),
				URL.to_vec(),
				None,
				None,
				core_primitives::AttestationType::Ias,
			));
		}
	});
	ext
}
