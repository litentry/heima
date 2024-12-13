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
	all_evm_web3networks, get_eligible_identities, mock::*, Error, IDGraph, Identity,
	IdentityContext, IdentityStatus, Web3Network,
};
use frame_support::{assert_err, assert_noop, assert_ok, traits::Get};
use litentry_primitives::all_substrate_web3networks;
use sp_runtime::AccountId32;
pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
pub const BOB: AccountId32 = AccountId32::new([2u8; 32]);
pub const CHARLIE: AccountId32 = AccountId32::new([3u8; 32]);

#[test]
fn get_eligible_identities_works() {
	let mut id_graph = IDGraph::<Test>::default();
	id_graph.push((alice_substrate_identity(), IdentityContext::new(1u64, vec![])));
	id_graph.push((alice_evm_identity(), IdentityContext::new(1u64, vec![])));
	id_graph.push((alice_twitter_identity(1), IdentityContext::new(2u64, vec![])));

	// only `alice_substrate_identity` is left
	let mut desired_web3networks = vec![Web3Network::Litentry, Web3Network::Polkadot];
	let mut identities =
		get_eligible_identities(id_graph.as_ref(), desired_web3networks.clone(), false);
	assert_eq!(identities.len(), 1);
	assert_eq!(identities[0].1, vec![Web3Network::Polkadot, Web3Network::Litentry]);

	// only `alice_evm_identity` is left
	desired_web3networks = vec![Web3Network::Arbitrum];
	identities = get_eligible_identities(id_graph.as_ref(), desired_web3networks, false);
	assert_eq!(identities.len(), 1);
	assert_eq!(identities[0].1, vec![Web3Network::Arbitrum]);

	// only twitter identity is left
	desired_web3networks = vec![];
	identities = get_eligible_identities(id_graph.as_ref(), desired_web3networks, false);
	assert_eq!(identities.len(), 1);
	assert_eq!(identities[0].1, vec![]);
	assert_eq!(identities[0].0, alice_twitter_identity(1));

	desired_web3networks = vec![Web3Network::Arbitrum];

	// no identity is filtered out, note the networks are reset to the default networks
	identities = get_eligible_identities(id_graph.as_ref(), desired_web3networks, true);
	assert_eq!(identities.len(), 3);
	assert_eq!(identities[0].1, all_substrate_web3networks());
	assert_eq!(identities[1].1, all_evm_web3networks());
	assert_eq!(identities[2].1, vec![]);
}

#[test]
fn link_twitter_identity_works() {
	new_test_ext().execute_with(|| {
		let who: Identity = BOB.into();

		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			who.clone(),
			alice_twitter_identity(1),
			vec![],
		));
		assert_eq!(
			IMT::id_graphs(who.clone(), alice_twitter_identity(1)).unwrap(),
			IdentityContext { link_block: 1, web3networks: vec![], status: IdentityStatus::Active }
		);
		assert_eq!(crate::IDGraphLens::<Test>::get(&who), 2);
	});
}

#[test]
fn link_substrate_identity_works() {
	new_test_ext().execute_with(|| {
		let web3networks: Vec<Web3Network> = vec![Web3Network::Litentry];
		let who: Identity = BOB.into();
		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			who.clone(),
			alice_substrate_identity(),
			web3networks.clone(),
		));
		assert_eq!(
			IMT::id_graphs(who.clone(), alice_substrate_identity()).unwrap(),
			IdentityContext { link_block: 1, web3networks, status: IdentityStatus::Active }
		);
		assert_eq!(crate::IDGraphLens::<Test>::get(&who), 2);
	});
}

#[test]
fn link_evm_identity_works() {
	new_test_ext().execute_with(|| {
		let web3networks: Vec<Web3Network> = vec![Web3Network::Ethereum, Web3Network::Bsc];
		let who: Identity = BOB.into();
		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			who.clone(),
			alice_evm_identity(),
			web3networks.clone(),
		));
		assert_eq!(
			IMT::id_graphs(who.clone(), alice_evm_identity()).unwrap(),
			IdentityContext { link_block: 1, web3networks, status: IdentityStatus::Active }
		);
		assert_eq!(crate::IDGraphLens::<Test>::get(&who), 2);
	});
}

#[test]
fn link_identity_with_wrong_network_fails() {
	new_test_ext().execute_with(|| {
		let web3networks: Vec<Web3Network> = vec![Web3Network::Bsc];
		let who: Identity = BOB.into();
		assert_noop!(
			IMT::link_identity(
				RuntimeOrigin::signed(ALICE),
				who,
				alice_substrate_identity(),
				web3networks,
			),
			Error::<Test>::WrongWeb3NetworkTypes
		);
	});
}

#[test]
fn link_identity_fails_for_linked_identity() {
	new_test_ext().execute_with(|| {
		// bob -> alice OK
		let web3networks: Vec<Web3Network> = vec![Web3Network::Litentry];
		let alice: Identity = ALICE.into();
		let bob: Identity = BOB.into();
		let charlie: Identity = CHARLIE.into();
		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			bob.clone(),
			alice.clone(),
			web3networks.clone(),
		));
		assert_eq!(
			IMT::id_graphs(bob.clone(), alice.clone()).unwrap(),
			IdentityContext {
				link_block: 1,
				web3networks: web3networks.clone(),
				status: IdentityStatus::Active
			}
		);
		assert_eq!(crate::IDGraphLens::<Test>::get(&bob), 2);

		// charlie -> alice NOK, as alice is already in bob's IDGraph
		assert_err!(
			IMT::link_identity(
				RuntimeOrigin::signed(ALICE),
				charlie.clone(),
				alice.clone(),
				web3networks.clone(),
			),
			Error::<Test>::IdentityAlreadyLinked
		);

		assert_eq!(crate::IDGraphLens::<Test>::get(&charlie), 0);

		// alice -> charlie NOK, as alice is already in bob's IDGraph
		assert_err!(
			IMT::link_identity(RuntimeOrigin::signed(ALICE), alice.clone(), charlie, web3networks,),
			Error::<Test>::IdentityAlreadyLinked
		);

		assert_eq!(crate::IDGraphLens::<Test>::get(&alice), 0);
	});
}

#[test]
fn cannot_link_identity_again() {
	new_test_ext().execute_with(|| {
		let web3networks: Vec<Web3Network> = vec![Web3Network::Polkadot];
		let who_bob: Identity = BOB.into();
		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			who_bob.clone(),
			alice_substrate_identity(),
			web3networks.clone()
		));
		assert_eq!(
			IMT::id_graphs(who_bob.clone(), alice_substrate_identity()).unwrap(),
			IdentityContext {
				link_block: 1,
				status: IdentityStatus::Active,
				web3networks: web3networks.clone()
			}
		);
		assert_eq!(crate::IDGraphLens::<Test>::get(&who_bob), 2);

		let who_alice: Identity = ALICE.into();

		assert_err!(
			IMT::link_identity(
				RuntimeOrigin::signed(ALICE),
				who_alice.clone(),
				alice_substrate_identity(),
				web3networks
			),
			Error::<Test>::IdentityAlreadyLinked
		);

		assert_eq!(crate::IDGraphLens::<Test>::get(&who_alice), 0);
	});
}

#[test]
fn cannot_create_more_identities_for_account_than_limit() {
	new_test_ext().execute_with(|| {
		let max_id_graph_len = <<Test as crate::Config>::MaxIDGraphLength as Get<u32>>::get();
		let who: Identity = BOB.into();

		for i in 1..max_id_graph_len {
			assert_ok!(IMT::link_identity(
				RuntimeOrigin::signed(ALICE),
				who.clone(),
				alice_twitter_identity(i),
				vec![],
			));
		}
		assert_err!(
			IMT::link_identity(
				RuntimeOrigin::signed(ALICE),
				who,
				alice_twitter_identity(65),
				vec![],
			),
			Error::<Test>::IDGraphLenLimitReached
		);
	});
}

#[test]
fn deactivate_identity_works() {
	new_test_ext().execute_with(|| {
		let who: Identity = BOB.into();

		assert_noop!(
			IMT::deactivate_identity(
				RuntimeOrigin::signed(ALICE),
				who.clone(),
				alice_substrate_identity(),
			),
			Error::<Test>::IdentityNotExist
		);
		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			who.clone(),
			alice_substrate_identity(),
			vec![Web3Network::Litentry].try_into().unwrap(),
		));
		assert_eq!(
			IMT::id_graphs(who.clone(), alice_substrate_identity()).unwrap(),
			IdentityContext {
				link_block: 1,
				web3networks: vec![Web3Network::Litentry].try_into().unwrap(),
				status: IdentityStatus::Active
			}
		);

		let id_graph = IMT::id_graph(&who);
		assert_eq!(id_graph.len(), 2);
		assert_eq!(crate::IDGraphLens::<Test>::get(&who), 2);

		assert_ok!(IMT::deactivate_identity(
			RuntimeOrigin::signed(ALICE),
			who.clone(),
			alice_substrate_identity(),
		));
		assert_eq!(
			IMT::id_graphs(who.clone(), alice_substrate_identity()).unwrap(),
			IdentityContext {
				link_block: 1,
				web3networks: vec![Web3Network::Litentry].try_into().unwrap(),
				status: IdentityStatus::Inactive
			}
		);

		let id_graph = IMT::id_graph(&who)
			.into_iter()
			.filter(|(_, c)| c.is_active())
			.collect::<IDGraph<Test>>();
		// "1": because of the main id is added by default when first calling link_identity.
		assert_eq!(id_graph.len(), 1);
		assert_eq!(IMT::id_graph(&who).len(), 2);
		// identity is only deactivated, so it still exists
		assert_eq!(crate::IDGraphLens::<Test>::get(&who), 2);

		assert_ok!(IMT::deactivate_identity(
			RuntimeOrigin::signed(ALICE),
			who,
			bob_substrate_identity(),
		));
	});
}

#[test]
fn activate_identity_works() {
	new_test_ext().execute_with(|| {
		let who: Identity = BOB.into();

		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			who.clone(),
			alice_substrate_identity(),
			vec![Web3Network::Litentry].try_into().unwrap(),
		));
		assert_eq!(
			IMT::id_graphs(who.clone(), alice_substrate_identity()).unwrap(),
			IdentityContext {
				link_block: 1,
				web3networks: vec![Web3Network::Litentry].try_into().unwrap(),
				status: IdentityStatus::Active
			}
		);
		let id_graph = IMT::id_graph(&who);
		assert_eq!(id_graph.len(), 2);
		assert_eq!(crate::IDGraphLens::<Test>::get(&who), 2);

		assert_ok!(IMT::deactivate_identity(
			RuntimeOrigin::signed(ALICE),
			who.clone(),
			alice_substrate_identity(),
		));
		assert_eq!(
			IMT::id_graphs(who.clone(), alice_substrate_identity()).unwrap(),
			IdentityContext {
				link_block: 1,
				web3networks: vec![Web3Network::Litentry].try_into().unwrap(),
				status: IdentityStatus::Inactive
			}
		);
		let id_graph = IMT::id_graph(&who)
			.into_iter()
			.filter(|(_, c)| c.is_active())
			.collect::<IDGraph<Test>>();
		// "1": because of the main id is added by default when first calling link_identity.
		assert_eq!(id_graph.len(), 1);
		// identity is only deactivated, so it still exists
		assert_eq!(crate::IDGraphLens::<Test>::get(&who), 2);

		assert_ok!(IMT::activate_identity(
			RuntimeOrigin::signed(ALICE),
			who.clone(),
			alice_substrate_identity(),
		));

		let id_graph = IMT::id_graph(&who);
		assert_eq!(id_graph.len(), 2);
		assert_eq!(crate::IDGraphLens::<Test>::get(&who), 2);
	});
}

#[test]
fn set_identity_networks_works() {
	new_test_ext().execute_with(|| {
		let who: Identity = BOB.into();

		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			who.clone(),
			alice_substrate_identity(),
			vec![Web3Network::Litentry].try_into().unwrap(),
		));
		assert_eq!(
			IMT::id_graphs(who.clone(), alice_substrate_identity()).unwrap(),
			IdentityContext {
				link_block: 1,
				web3networks: vec![Web3Network::Litentry].try_into().unwrap(),
				status: IdentityStatus::Active
			}
		);

		let new_networks: _ = vec![Web3Network::Kusama, Web3Network::Khala];
		assert_ok!(IMT::set_identity_networks(
			RuntimeOrigin::signed(ALICE),
			who.clone(),
			alice_substrate_identity(),
			new_networks.clone(),
		));
		assert_eq!(
			IMT::id_graphs(who, alice_substrate_identity()).unwrap().web3networks.to_vec(),
			new_networks
		);
	})
}

#[test]
fn set_identity_networks_with_wrong_network_fails() {
	new_test_ext().execute_with(|| {
		let who: Identity = BOB.into();

		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			who.clone(),
			alice_substrate_identity(),
			vec![Web3Network::Litentry].try_into().unwrap(),
		));
		assert_eq!(
			IMT::id_graphs(who.clone(), alice_substrate_identity()).unwrap(),
			IdentityContext {
				link_block: 1,
				web3networks: vec![Web3Network::Litentry].try_into().unwrap(),
				status: IdentityStatus::Active
			}
		);

		let new_networks: _ = vec![Web3Network::Bsc, Web3Network::Khala];
		assert_noop!(
			IMT::set_identity_networks(
				RuntimeOrigin::signed(ALICE),
				who,
				alice_substrate_identity(),
				new_networks,
			),
			Error::<Test>::WrongWeb3NetworkTypes
		);
	})
}

#[test]
fn get_id_graph_works() {
	new_test_ext().execute_with(|| {
		let who: Identity = BOB.into();

		// fill in 21 identities, starting from 1 to reserve place for prime_id
		// set the block number too as it's used to tell "recent"
		for i in 1..22 {
			System::set_block_number(i + 1);
			assert_ok!(IMT::link_identity(
				RuntimeOrigin::signed(ALICE),
				who.clone(),
				alice_twitter_identity(i.try_into().unwrap()),
				vec![],
			));
		}
		// the full id_graph should have 22 elements, including the prime_id
		let id_graph = IMT::id_graph(&who);
		assert_eq!(id_graph.len(), 22);

		assert_eq!(id_graph.get(0).unwrap().0, who);

		// index 21 has the newest identity
		assert_eq!(id_graph.get(21).unwrap().0, alice_twitter_identity(21));
	});
}

#[test]
fn get_id_graph_identities_within_same_block() {
	new_test_ext().execute_with(|| {
		let who: Identity = ALICE.into();
		System::set_block_number(1);

		let identities = vec![
			(alice_twitter_identity(1), vec![]),
			(alice_substrate_identity(), vec![Web3Network::LitentryRococo]),
			(alice_evm_identity(), vec![Web3Network::Ethereum]),
			(bob_substrate_identity(), vec![Web3Network::Litentry]),
		];

		for (identity, networks) in identities {
			assert_ok!(IMT::link_identity(
				RuntimeOrigin::signed(ALICE),
				who.clone(),
				identity,
				networks,
			));
		}

		let id_graph = IMT::id_graph(&who);
		let sorted_identities = [
			alice_evm_identity(),
			who.clone(),
			bob_substrate_identity(),
			alice_substrate_identity(),
			alice_twitter_identity(1),
		];

		for (i, identity) in sorted_identities.iter().enumerate() {
			assert_eq!(&id_graph.get(i).unwrap().0, identity);
		}

		// clear all identites
		assert_ok!(IMT::remove_identity(RuntimeOrigin::signed(ALICE), who.clone(), vec![],));

		// change order of the identites
		let identities = vec![
			(bob_substrate_identity(), vec![Web3Network::Litentry]),
			(alice_substrate_identity(), vec![Web3Network::LitentryRococo]),
			(alice_twitter_identity(1), vec![]),
			(alice_evm_identity(), vec![Web3Network::Ethereum]),
		];

		for (identity, networks) in identities {
			assert_ok!(IMT::link_identity(
				RuntimeOrigin::signed(ALICE),
				who.clone(),
				identity,
				networks,
			));
		}

		let id_graph = IMT::id_graph(&who);

		for (i, identity) in sorted_identities.iter().enumerate() {
			assert_eq!(&id_graph.get(i).unwrap().0, identity);
		}
	});
}

#[test]
fn id_graph_stats_works() {
	new_test_ext().execute_with(|| {
		let alice: Identity = ALICE.into();
		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			alice.clone(),
			bob_substrate_identity(),
			vec![Web3Network::Litentry].try_into().unwrap(),
		));
		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			alice.clone(),
			alice_twitter_identity(1),
			vec![],
		));

		let stats = IMT::id_graph_stats().unwrap();
		// alice's IDGraph should have 3 entries:
		// alice's identity itself, bob_substrate_identity, alice_twitter_identity
		assert_eq!(stats.len(), 1);
		assert!(stats.contains(&(alice, 3)));
	});
}

#[test]
fn remove_one_identity_works() {
	new_test_ext().execute_with(|| {
		let alice: Identity = ALICE.into();
		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			alice.clone(),
			bob_substrate_identity(),
			vec![Web3Network::Litentry].try_into().unwrap(),
		));
		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			alice.clone(),
			alice_twitter_identity(1),
			vec![],
		));

		// alice's IDGraph should have 3 entries:
		// alice's identity itself, bob_substrate_identity, alice_twitter_identity
		assert_eq!(IMT::id_graph(&alice).len(), 3);

		assert_ok!(IMT::remove_identity(
			RuntimeOrigin::signed(ALICE),
			alice.clone(),
			vec![bob_substrate_identity()],
		));

		assert_eq!(IMT::id_graph(&alice).len(), 2);
	});
}

#[test]
fn remove_whole_identity_graph_works() {
	new_test_ext().execute_with(|| {
		let alice: Identity = ALICE.into();
		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			alice.clone(),
			bob_substrate_identity(),
			vec![Web3Network::Litentry].try_into().unwrap(),
		));
		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			alice.clone(),
			alice_twitter_identity(1),
			vec![],
		));

		// alice's IDGraph should have 3 entries:
		// alice's identity itself, bob_substrate_identity, alice_twitter_identity
		assert_eq!(IMT::id_graph(&alice).len(), 3);

		assert_ok!(IMT::remove_identity(RuntimeOrigin::signed(ALICE), alice.clone(), vec![],));

		assert_eq!(IMT::id_graph(&alice).len(), 0);
	});
}

#[test]
fn remove_identity_graph_of_other_account_fails() {
	new_test_ext().execute_with(|| {
		let alice: Identity = ALICE.into();
		let bob: Identity = BOB.into();

		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			alice.clone(),
			bob_substrate_identity(),
			vec![Web3Network::Litentry].try_into().unwrap(),
		));
		assert_ok!(IMT::link_identity(
			RuntimeOrigin::signed(ALICE),
			alice,
			alice_twitter_identity(1),
			vec![],
		));

		assert_noop!(
			IMT::remove_identity(
				RuntimeOrigin::signed(ALICE),
				bob,
				vec![alice_substrate_identity()],
			),
			Error::<Test>::IdentityNotExist
		);
	});
}
