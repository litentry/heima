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

use frame_support::{dispatch::*, weights::Weight};
pub use pallet_balances::Call as BalancesCall;
use parity_scale_codec::Decode;
use sp_runtime::{BuildStorage, SaturatedConversion};
pub use sp_std::cell::RefCell;

use core_primitives::{AccountId, Balance, BlockNumber};

use crate::{currency::UNIT, BaseRuntimeRequirements};

pub const ALICE: [u8; 32] = [1u8; 32];
pub const BOB: [u8; 32] = [2u8; 32];
pub const CHARLIE: [u8; 32] = [3u8; 32];

pub mod relay;

pub fn alice() -> AccountId {
	AccountId::from(ALICE)
}

pub fn bob() -> AccountId {
	AccountId::from(BOB)
}

pub fn charlie() -> AccountId {
	AccountId::from(CHARLIE)
}

pub const PARA_A_USER_INITIAL_BALANCE: u128 = 500_000_000_000 * UNIT;
pub const PARA_B_USER_INITIAL_BALANCE: u128 = 600_000_000_000 * UNIT;

pub struct ExtBuilder<R> {
	phantom: sp_std::marker::PhantomData<R>,
	balances: Vec<(AccountId, Balance)>,
	parachain_id: u32,
}

impl<R> Default for ExtBuilder<R> {
	fn default() -> Self {
		Self { phantom: Default::default(), balances: vec![], parachain_id: 1 }
	}
}

impl<R> ExtBuilder<R>
where
	R: BaseRuntimeRequirements,
{
	pub fn balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	#[allow(dead_code)]
	pub fn parachain_id(mut self, parachain_id: u32) -> Self {
		self.parachain_id = parachain_id;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<R>::default().build_storage().unwrap();

		pallet_balances::GenesisConfig::<R> {
			balances: self
				.balances
				.into_iter()
				.map(|(account_id, initial_balance)| {
					(
						<R as frame_system::Config>::AccountId::decode(&mut account_id.as_ref())
							.unwrap(),
						// <R as pallet_balances::Config<R>>::Balance::from(initial_balance)
						initial_balance.saturated_into(),
					)
				})
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		parachain_info::GenesisConfig::<R> {
			parachain_id: self.parachain_id.into(),
			..Default::default()
		}
		.assimilate_storage(&mut t)
		.unwrap();

		pallet_xcm::GenesisConfig::<R> { safe_xcm_version: Some(2), ..Default::default() }
			.assimilate_storage(&mut t)
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		let block: BlockNumber = 1;
		// let block = <R as frame_system::Config>::BlockNumber::from(block_number);
		ext.execute_with(|| frame_system::Pallet::<R>::set_block_number(block.into()));
		ext
	}
}

/// create a transaction info struct from weight. Handy to avoid building the whole struct.
pub fn info_from_weight(w: Weight) -> DispatchInfo {
	// pays_fee: Pays::Yes -- class: DispatchClass::Normal
	DispatchInfo { weight: w, ..Default::default() }
}

pub fn post_info_from_weight(w: Weight) -> PostDispatchInfo {
	PostDispatchInfo { actual_weight: Some(w), pays_fee: Default::default() }
}

pub fn run_with_system_weight<F, R>(w: Weight, mut assertions: F)
where
	F: FnMut(),
	R: BaseRuntimeRequirements,
{
	let mut t: sp_io::TestExternalities =
		frame_system::GenesisConfig::<R>::default().build_storage().unwrap().into();
	t.execute_with(|| {
		frame_system::Pallet::<R>::set_block_consumed_resources(w, 0);
		assertions()
	});
}

/// This macro expects the passed runtime(litentry litmus rococo) to contain
/// `cumulus_pallet_xcmp_queue` and `cumulus_pallet_dmp_queue`.
#[macro_export]
macro_rules! decl_test_chain {
	($runtime:ident) => {
		use core_primitives::Weight;
		use frame_support::{construct_runtime, match_types, parameter_types};
		use runtime_common::tests::setup::{
			alice, bob,
			relay::{LocalOriginConverter, OnlyParachains, UniversalLocation},
			ExtBuilder, PARA_A_USER_INITIAL_BALANCE, PARA_B_USER_INITIAL_BALANCE,
		};
		use xcm::prelude::Parent;
		use xcm_simulator::{
			decl_test_network, decl_test_parachain, decl_test_relay_chain, TestExt,
		};
		pub mod relay_chain {
			// declared by `decl_test_network`
			use super::RelayChainXcmRouter;
			runtime_common::decl_test_relay_chain_runtime!();
		}
		use sp_runtime::BuildStorage;

		pub fn relay_ext() -> sp_io::TestExternalities {
			use relay_chain::{Runtime, System};
			let t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
			let mut ext = sp_io::TestExternalities::new(t);
			ext.execute_with(|| {
				System::set_block_number(1);
			});
			ext
		}

		decl_test_parachain! {
			pub struct ParaA {
				Runtime = $runtime,
				XcmpMessageHandler = cumulus_pallet_xcmp_queue::Pallet::<$runtime>,
				DmpMessageHandler = cumulus_pallet_dmp_queue::Pallet::<$runtime>,
				new_ext = ExtBuilder::<$runtime>::default()
				.balances(vec![
					(alice(), PARA_A_USER_INITIAL_BALANCE),
				]).parachain_id(1).build(),
			}
		}

		decl_test_parachain! {
			pub struct ParaB {
				Runtime = $runtime,
				XcmpMessageHandler = cumulus_pallet_xcmp_queue::Pallet::<$runtime>,
				DmpMessageHandler = cumulus_pallet_dmp_queue::Pallet::<$runtime>,
				new_ext = ExtBuilder::<$runtime>::default()
				.balances(vec![
					(bob(), PARA_B_USER_INITIAL_BALANCE),
				]).parachain_id(2).build(),
			}
		}

		decl_test_relay_chain! {
			pub struct Relay {
				Runtime = relay_chain::Runtime,
				RuntimeCall = relay_chain::RuntimeCall,
				RuntimeEvent = relay_chain::RuntimeEvent,
				XcmConfig = relay_chain::XcmConfig,
				MessageQueue = relay_chain::MessageQueue,
				System = relay_chain::System,
				new_ext = relay_ext(),
			}
		}

		decl_test_network! {
			pub struct TestNet {
				relay_chain = Relay,
				parachains = vec![
					(1, ParaA),
					(2, ParaB),
				],
			}
		}
	};
}
