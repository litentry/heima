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
use crate as pallet_evm_assertions;
use frame_support::{derive_impl, parameter_types, weights::Weight};
use sp_core::{H160, U256};
use sp_runtime::{
	traits::{IdentifyAccount, IdentityLookup, Verify},
	BuildStorage, MultiSignature,
};

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Balance = u128;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		EvmAssertions: pallet_evm_assertions,
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
	pub const MinimumPeriod: u64 = 6000 / 2;
}

impl pallet_evm_assertions::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type AssertionId = H160;
	type ContractDevOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type TEECallOrigin = frame_system::EnsureRoot<Self::AccountId>;
}

parameter_types! {
	pub WeightPerGas: Weight = Weight::from_parts(20_000, 0);
	// It will be the best if we can implement this in a more professional way
	pub ChainId: u64 = 2106u64;
	pub BlockGasLimit: U256 = U256::max_value();
	// // BlockGasLimit / MAX_POV_SIZE
	pub const GasLimitPovSizeRatio: u64 = 150_000_000 / (5 * 1024 * 1024);
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let test_storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let mut ext = sp_io::TestExternalities::new(test_storage);
	ext.execute_with(|| {
		System::set_block_number(1);
	});
	ext
}
