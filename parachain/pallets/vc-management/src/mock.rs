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

#![cfg(test)]

use crate as pallet_vc_management;
use frame_support::{
	assert_ok, derive_impl,
	pallet_prelude::EnsureOrigin,
	parameter_types,
	traits::{ConstU32, ConstU64},
};
use frame_system::EnsureRoot;
use sp_runtime::{
	traits::{IdentifyAccount, IdentityLookup, Verify},
	BuildStorage,
};
use sp_std::marker::PhantomData;

pub type Signature = sp_runtime::MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

pub type Balance = u128;
type SystemAccountId = <Test as frame_system::Config>::AccountId;

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

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		Teebag: pallet_teebag,
		Timestamp: pallet_timestamp,
		Utility: pallet_utility,
		VCManagement: pallet_vc_management,
		VCMPExtrinsicWhitelist: pallet_group,
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

impl pallet_vc_management::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type TEECallOrigin = EnsureEnclaveSigner<Self>;
	type SetAdminOrigin = EnsureRoot<Self::AccountId>;
	type DelegateeAdminOrigin = EnsureRoot<Self::AccountId>;
	type ExtrinsicWhitelistOrigin = VCMPExtrinsicWhitelist;
}

parameter_types! {
	pub const MomentsPerDay: u64 = 86_400_000; // [ms/d]
}

impl pallet_teebag::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MomentsPerDay = MomentsPerDay;
	type SetAdminOrigin = EnsureRoot<Self::AccountId>;
	type MaxEnclaveIdentifier = ConstU32<3>;
	type MaxAuthorizedEnclave = ConstU32<3>;
	type WeightInfo = ();
}

impl pallet_group::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type GroupManagerOrigin = frame_system::EnsureRoot<Self::AccountId>;
}

impl pallet_utility::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	use pallet_teebag::test_util::{
		get_signer, TEST8_CERT, TEST8_SIGNER_PUB, TEST8_TIMESTAMP, URL,
	};

	let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let alice: SystemAccountId = get_signer(&[1u8; 32]);
	let eddie: SystemAccountId = get_signer(&[5u8; 32]);
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
		let _ = VCManagement::set_admin(RuntimeOrigin::root(), alice.clone());
		let _ = VCManagement::add_delegatee(RuntimeOrigin::root(), eddie);
		assert_ok!(Teebag::set_admin(RuntimeOrigin::root(), alice.clone()));
		assert_ok!(Teebag::set_mode(
			RuntimeOrigin::signed(alice.clone()),
			core_primitives::OperationalMode::Development
		));
		Timestamp::set_timestamp(TEST8_TIMESTAMP);
		let signer: SystemAccountId = get_signer(TEST8_SIGNER_PUB);
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
