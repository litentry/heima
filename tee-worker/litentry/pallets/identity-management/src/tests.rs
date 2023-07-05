// Copyright 2020-2023 Litentry Technologies GmbH.
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
	mock::*, Error, Identity, IdentityContext, IdentityStatus, UserShieldingKeyType, Web3Network,
};
use frame_support::{assert_err, assert_noop, assert_ok, traits::Get};
use litentry_primitives::{IdentityString, USER_SHIELDING_KEY_LEN};
use sp_runtime::{AccountId32, BoundedVec};

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
pub const BOB: AccountId32 = AccountId32::new([2u8; 32]);

#[test]
fn set_user_shielding_key_works() {
	new_test_ext(false).execute_with(|| {
		let shielding_key: UserShieldingKeyType = [0u8; USER_SHIELDING_KEY_LEN];
		assert_eq!(IMT::user_shielding_keys(BOB), None);

		assert_ok!(IMT::set_user_shielding_key(
			RuntimeOrigin::signed(ALICE),
			BOB,
			shielding_key.clone(),
		));
		assert_eq!(IMT::user_shielding_keys(BOB), Some(shielding_key.clone()));
		System::assert_last_event(RuntimeEvent::IMT(crate::Event::UserShieldingKeySet {
			who: BOB,
			key: shielding_key,
		}));
		assert_eq!(crate::IDGraphLens::<Test>::get(&BOB), 1);
	});
}

#[test]
fn link_identity_works() {
	new_test_ext(true).execute_with(|| {
		let web3networks: BoundedVec<Web3Network, <Test as crate::Config>::MaxWeb3NetworkLength> =
			BoundedVec::truncate_from(vec![Web3Network::Litentry]);
		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			BOB,
			alice_web3_identity(),
			web3networks,
		));
		assert_eq!(
			IMT::id_graphs(BOB, alice_web3_identity()).unwrap(),
			IdentityContext { link_block: 1, web3networks, status: IdentityStatus::Active }
		);
		assert_eq!(crate::IDGraphLens::<Test>::get(&BOB), 2);
	});
}

#[test]
fn cannot_create_more_identities_for_account_than_limit() {
	new_test_ext(true).execute_with(|| {
		let max_id_graph_len = <<Test as crate::Config>::MaxIDGraphLength as Get<u32>>::get();
		for i in 1..max_id_graph_len {
			assert_ok!(IMT::link_identity(
				RuntimeOrigin::signed(ALICE),
				BOB,
				alice_twitter_identity(i),
				vec![],
			));
		}
		assert_err!(
			IMT::link_identity(
				RuntimeOrigin::signed(ALICE),
				BOB,
				alice_twitter_identity(65),
				vec![],
			),
			Error::<Test>::IDGraphLenLimitReached
		);
	});
}

#[test]
fn remove_identity_works() {
	new_test_ext(false).execute_with(|| {
		let shielding_key: UserShieldingKeyType = [0u8; USER_SHIELDING_KEY_LEN];

		assert_ok!(IMT::set_user_shielding_key(
			RuntimeOrigin::signed(ALICE),
			BOB,
			shielding_key.clone(),
		));
		assert_noop!(
			IMT::remove_identity(RuntimeOrigin::signed(ALICE), BOB, alice_web3_identity(),),
			Error::<Test>::IdentityNotExist
		);
		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			BOB,
			alice_web3_identity(),
			vec![],
		));
		assert_eq!(
			IMT::id_graphs(BOB, alice_web3_identity()).unwrap(),
			IdentityContext {
				link_block: 1,
				web3networks: BoundedVec::default(),
				status: IdentityStatus::Active
			}
		);

		let id_graph = IMT::get_id_graph(&BOB, usize::MAX);
		assert_eq!(id_graph.len(), 2);
		assert_eq!(crate::IDGraphLens::<Test>::get(&BOB), 2);

		assert_ok!(IMT::remove_identity(RuntimeOrigin::signed(ALICE), BOB, alice_web3_identity(),));
		assert_eq!(IMT::id_graphs(BOB, alice_web3_identity()), None);

		let id_graph = IMT::get_id_graph(&BOB, usize::MAX);
		// "1": because of the main id is added by default when first calling set_user_shielding_key.
		assert_eq!(id_graph.len(), 1);
		assert_eq!(crate::IDGraphLens::<Test>::get(&BOB), 1);

		assert_noop!(
			IMT::remove_identity(RuntimeOrigin::signed(ALICE), BOB, bob_web3_identity(),),
			Error::<Test>::RemovePrimeIdentityDisallowed
		);
	});
}

#[test]
fn get_id_graph_works() {
	new_test_ext(true).execute_with(|| {
		// fill in 21 identities, starting from 1 to reserve place for prime_id
		// set the block number too as it's used to tell "recent"
		for i in 1..22 {
			System::set_block_number(i + 1);
			assert_ok!(IMT::link_identity(
				RuntimeOrigin::signed(ALICE),
				BOB,
				alice_twitter_identity(i.try_into().unwrap()),
				vec![],
			));
		}
		// the full id_graph should have 22 elements, including the prime_id
		assert_eq!(IMT::get_id_graph(&BOB, usize::MAX).len(), 22);

		// only get the recent 15 identities
		let id_graph = IMT::get_id_graph(&BOB, 15);
		assert_eq!(id_graph.len(), 15);
		// index 0 has the most recent identity
		assert_eq!(
			id_graph.get(0).unwrap().0,
			Identity::Twitter("alice21".as_bytes().to_vec().try_into().unwrap())
		);
		// index 14 has the least recent identity
		assert_eq!(
			id_graph.get(14).unwrap().0,
			Identity::Twitter("alice7".as_bytes().to_vec().try_into().unwrap())
		);

		// try to get more than id_graph length
		let id_graph = IMT::get_id_graph(&BOB, 30);
		assert_eq!(id_graph.len(), 22);
		assert_eq!(
			id_graph.get(0).unwrap().0,
			Identity::Twitter("alice21".as_bytes().to_vec().try_into().unwrap())
		);
		assert_eq!(id_graph.get(21).unwrap().0, Identity::Substrate([2u8; 32].into()));
	});
}

#[test]
fn id_graph_stats_works() {
	new_test_ext(true).execute_with(|| {
		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			ALICE,
			alice_web3_identity(),
			vec![],
		));
		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			ALICE,
			alice_twitter_identity(1),
			vec![],
		));

		let stats = IMT::id_graph_stats().unwrap();
		assert_eq!(stats.len(), 2);
		assert!(stats.contains(&(ALICE, 2)));
		// bob's identity is created when setting shielding key
		assert!(stats.contains(&(BOB, 1)));
	});
}
