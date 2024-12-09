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

mod parachain;
mod relay_chain;

use crate::{self as runtime_common, currency::UNIT, BaseRuntimeRequirements};
use core_primitives::AccountId;

use frame_support::dispatch::{DispatchInfo, PostDispatchInfo};
use sp_runtime::BuildStorage;
use xcm::latest::prelude::*;
use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain, TestExt};

pub fn alice() -> AccountId {
	AccountId::from([1u8; 32])
}

pub fn bob() -> AccountId {
	AccountId::from([2u8; 32])
}

pub fn charlie() -> AccountId {
	AccountId::from([3u8; 32])
}

pub const PARA_A_USER_INITIAL_BALANCE: u128 = 500_000_000_000 * UNIT;
pub const PARA_B_USER_INITIAL_BALANCE: u128 = 600_000_000_000 * UNIT;

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

decl_test_parachain! {
	pub struct ParaA {
		Runtime = parachain::Runtime,
		XcmpMessageHandler = parachain::MsgQueue,
		DmpMessageHandler = parachain::MsgQueue,
		new_ext = para_ext(1),
	}
}

decl_test_parachain! {
	pub struct ParaB {
		Runtime = parachain::Runtime,
		XcmpMessageHandler = parachain::MsgQueue,
		DmpMessageHandler = parachain::MsgQueue,
		new_ext = para_ext(2),
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
	pub struct MockNet {
		relay_chain = Relay,
		parachains = vec![
			(1, ParaA),
			(2, ParaB),
		],
	}
}

pub fn para_ext(para_id: u32) -> sp_io::TestExternalities {
	use runtime_common::tests::setup::parachain::{MsgQueue, Runtime, System};

	let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Runtime> { balances: vec![(alice(), 10 * UNIT)] }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
		MsgQueue::set_para_id(para_id.into());
	});
	ext
}

pub fn relay_ext() -> sp_io::TestExternalities {
	use runtime_common::tests::setup::relay_chain::{Runtime, System};

	let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Runtime> { balances: vec![(alice(), 10 * UNIT)] }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub type RelayChainPalletXcm = pallet_xcm::Pallet<relay_chain::Runtime>;

pub type ParachainPalletXcm = pallet_xcm::Pallet<parachain::Runtime>;
pub type ParachainAssets = pallet_assets::Pallet<parachain::Runtime>;
pub type ParachainBalances = pallet_balances::Pallet<parachain::Runtime>;
