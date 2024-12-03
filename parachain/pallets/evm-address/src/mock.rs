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

use crate as pallet_evm_address;
use fp_evm::GenesisAccount;
use frame_support::{
	derive_impl, parameter_types,
	traits::{ConstU32, FindAuthor},
	weights::Weight,
	ConsensusEngineId,
};
use frame_system::RawOrigin;
use hex_literal::hex;
use parity_scale_codec::Encode;
use sp_core::{H160, U256};
use sp_runtime::{
	traits::{IdentifyAccount, IdentityLookup, Verify},
	BuildStorage, MultiSignature,
};
use sp_std::vec;
use std::collections::BTreeMap;

pub type Balance = u128;
pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		Timestamp: pallet_timestamp,
		EVM: pallet_evm,
		Ethereum: pallet_ethereum,
		EVMAddress: pallet_evm_address,
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

parameter_types! {
	pub const MinimumPeriod: u64 = 6000 / 2;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

use pallet_evm::{EnsureAddressOrigin, FeeCalculator};
pub struct EnsureAddressEqualAndStore<T>(sp_std::marker::PhantomData<T>);
impl<T, OuterOrigin> EnsureAddressOrigin<OuterOrigin> for EnsureAddressEqualAndStore<T>
where
	T: pallet_evm_address::Config<EVMId = H160>,
	OuterOrigin: Into<Result<RawOrigin<T::AccountId>, OuterOrigin>> + From<RawOrigin<T::AccountId>>,
{
	type Success = T::AccountId;

	fn try_address_origin(
		address: &H160,
		origin: OuterOrigin,
	) -> Result<T::AccountId, OuterOrigin> {
		origin.into().and_then(|o| match o {
			// In practice, the root should withdraw to treasury account or something
			RawOrigin::Root => Err(OuterOrigin::from(RawOrigin::Root)),
			RawOrigin::Signed(account_id) => {
				// AddressMapping revert logic check here
				if H160::from_slice(&account_id.encode()[0..20]) == *address {
					match pallet_evm_address::Pallet::<T>::add_address_mapping(
						*address,
						account_id.clone(),
					) {
						Ok(_) => Ok(account_id),
						Err(_) => Err(OuterOrigin::from(RawOrigin::Signed(account_id))),
					}
				} else {
					Err(OuterOrigin::from(RawOrigin::Signed(account_id)))
				}
			},
			r => Err(OuterOrigin::from(r)),
		})
	}
}

parameter_types! {
	pub WeightPerGas: Weight = Weight::from_parts(20_000, 0);
	// It will be the best if we can implement this in a more professional way
	pub ChainId: u64 = 2106u64;
	pub BlockGasLimit: U256 = U256::max_value();
	// // BlockGasLimit / MAX_POV_SIZE
	pub const GasLimitPovSizeRatio: u64 = 150_000_000 / (5 * 1024 * 1024);
}
use pallet_evm::AddressMapping;
pub struct EVMAddressMapping<T>(sp_std::marker::PhantomData<T>);
impl<T> AddressMapping<T::AccountId> for EVMAddressMapping<T>
where
	T: pallet_evm_address::Config<EVMId = H160> + frame_system::Config<AccountId = AccountId>,
{
	fn into_account_id(address: H160) -> T::AccountId {
		match pallet_evm_address::Pallet::<T>::get_address_mapped(address) {
			Some(r) => r,
			None => TruncatedAddressMapping::into_account_id(address),
		}
	}
}
pub struct TruncatedAddressMapping;
impl AddressMapping<AccountId> for TruncatedAddressMapping {
	fn into_account_id(address: H160) -> AccountId {
		let mut data = [0u8; 32];
		data[0..20].copy_from_slice(&address[..]);
		AccountId::from(Into::<[u8; 32]>::into(data))
	}
}

pub struct FindAuthorTruncated;
impl FindAuthor<H160> for FindAuthorTruncated {
	fn find_author<'a, I>(_digests: I) -> Option<H160>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		let author = sp_runtime::AccountId32::new(hex![
			"d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
		]);
		Some(H160::from_slice(&author.encode()[0..20]))
	}
}

pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> (U256, Weight) {
		// Return some meaningful gas price and weight
		(1.into(), Weight::zero())
	}
}

impl pallet_evm::Config for Test {
	type FeeCalculator = FixedGasPrice;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	// type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressEqualAndStore<Self>;
	type WithdrawOrigin = EnsureAddressEqualAndStore<Self>;
	// From evm address to parachain addressOnCreate
	type AddressMapping = EVMAddressMapping<Self>;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	// Minimal effort, no precompile for now
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type ChainId = ChainId;
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type Timestamp = Timestamp;
	type OnCreate = ();
	type FindAuthor = FindAuthorTruncated;
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type WeightInfo = ();
	type SuicideQuickClearLimit = ConstU32<0>;
}

use pallet_ethereum::PostLogContent;
parameter_types! {
	pub const PostBlockAndTxnHashes: PostLogContent = PostLogContent::BlockAndTxnHashes;
}

impl pallet_ethereum::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
	type PostLogContent = PostBlockAndTxnHashes;
	// Maximum length (in bytes) of revert message to include in Executed event
	type ExtraDataLength = ConstU32<256>;
}

impl pallet_evm_address::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type EVMId = H160;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	let mut accounts = BTreeMap::new();
	pub const ALICE: AccountId = sp_runtime::AccountId32::new(hex![
		"d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
	]);
	let alice_evm = H160::from_slice(&ALICE.encode()[0..20]);
	accounts.insert(
		alice_evm,
		GenesisAccount {
			nonce: U256::from(1),
			balance: U256::from(10_000_000_000_000_000u128),
			storage: Default::default(),
			code: vec![],
		},
	);
	pallet_balances::GenesisConfig::<Test> {
		// Create the block author account with some balance.
		balances: vec![(ALICE, 8_000_000_000_000_000_000u128)],
	}
	.assimilate_storage(&mut t)
	.expect("Pallet balances storage can be assimilated");

	pallet_evm::GenesisConfig::<Test> { accounts, ..Default::default() }
		.assimilate_storage(&mut t)
		.expect("Pallet evm storage can be assimilated");

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
	});
	ext
}
