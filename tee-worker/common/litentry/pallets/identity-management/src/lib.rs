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

// Identity management pallet run within TEE(enclave) -- aka IMT
// The pallet is integrated in SGX-runtime, the extrinsics are supposed
// to be called only by enclave
//
// TODO:
// - origin management, only allow calls from TEE (= origin is signed with the ECC key), or root?
//   otherwise we'd always require the origin has some fund
// - maybe don't emit events at all, or at least remove sensistive data
// - benchmarking

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod migrations;

pub use pallet::*;
pub mod identity_context;
pub use identity_context::*;

use frame_support::{pallet_prelude::*, sp_runtime::traits::One, traits::StorageVersion};
use frame_system::pallet_prelude::*;

pub use litentry_primitives::{
	all_bitcoin_web3networks, all_evm_web3networks, all_substrate_web3networks, Identity,
	ParentchainBlockNumber, Web3Network,
};
use sp_core::{blake2_256, H256};
use sp_std::{vec, vec::Vec};

pub type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
pub type IDGraph<T> = Vec<(Identity, IdentityContext<T>)>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use log::debug;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// the event
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// the manager origin for extrinsics
		type ManageOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// maximum number of identities an account can have, if you change this value to lower some accounts may exceed this limit
		#[pallet::constant]
		type MaxIDGraphLength: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// an identity was linked
		IdentityLinked { who: Identity, identity: Identity },
		/// an identity was deactivated
		IdentityDeactivated { who: Identity, identity: Identity },
		/// an identity was activated
		IdentityActivated { who: Identity, identity: Identity },
		/// an identity was removed
		IdentityRemoved { who: Identity, identity: Identity },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// the identity is already linked
		IdentityAlreadyLinked,
		/// the pair (Identity, Identity) doesn't exist
		IdentityNotExist,
		/// creating the prime identity manually is disallowed
		LinkPrimeIdentityDisallowed,
		/// IDGraph len limit reached
		IDGraphLenLimitReached,
		/// identity doesn't match the network types
		WrongWeb3NetworkTypes,
		/// identity cannot be used to build prime identity
		NotSupportedIdentity,
	}

	#[pallet::storage]
	pub type LinkedIdentities<T: Config> =
		StorageMap<_, Blake2_128Concat, Identity, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn id_graphs)]
	pub type IDGraphs<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Identity,
		Blake2_128Concat,
		Identity,
		IdentityContext<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn id_graph_lens)]
	pub type IDGraphLens<T: Config> = StorageMap<_, Blake2_128Concat, Identity, u32, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(1)]
		#[pallet::weight({15_000_000})]
		pub fn link_identity(
			origin: OriginFor<T>,
			who: Identity,
			identity: Identity,
			web3networks: Vec<Web3Network>,
		) -> DispatchResult {
			T::ManageOrigin::ensure_origin(origin)?;

			ensure!(
				!LinkedIdentities::<T>::contains_key(&identity),
				Error::<T>::IdentityAlreadyLinked
			);
			ensure!(identity != who, Error::<T>::LinkPrimeIdentityDisallowed);

			ensure!(
				identity.matches_web3networks(web3networks.as_ref()),
				Error::<T>::WrongWeb3NetworkTypes
			);

			Self::maybe_create_id_graph(&who)?;

			let context =
				<IdentityContext<T>>::new(<frame_system::Pallet<T>>::block_number(), web3networks);
			Self::insert_identity_with_limit(&who, &identity, context)?;
			Self::deposit_event(Event::IdentityLinked { who, identity });
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight({15_000_000})]
		pub fn deactivate_identity(
			origin: OriginFor<T>,
			who: Identity,
			identity: Identity,
		) -> DispatchResult {
			T::ManageOrigin::ensure_origin(origin)?;
			Self::maybe_create_id_graph(&who)?;
			ensure!(IDGraphs::<T>::contains_key(&who, &identity), Error::<T>::IdentityNotExist);

			IDGraphs::<T>::try_mutate(&who, &identity, |context| {
				let mut c = context.take().ok_or(Error::<T>::IdentityNotExist)?;
				c.deactivate();
				*context = Some(c);
				Result::<(), Error<T>>::Ok(())
			})?;
			Self::deposit_event(Event::IdentityDeactivated { who, identity });
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight({15_000_000})]
		pub fn activate_identity(
			origin: OriginFor<T>,
			who: Identity,
			identity: Identity,
		) -> DispatchResult {
			T::ManageOrigin::ensure_origin(origin)?;
			Self::maybe_create_id_graph(&who)?;
			ensure!(IDGraphs::<T>::contains_key(&who, &identity), Error::<T>::IdentityNotExist);

			IDGraphs::<T>::try_mutate(&who, &identity, |context| {
				let mut c = context.take().ok_or(Error::<T>::IdentityNotExist)?;
				c.activate();
				*context = Some(c);
				Result::<(), Error<T>>::Ok(())
			})?;
			Self::deposit_event(Event::IdentityActivated { who, identity });
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight({15_000_000})]
		pub fn set_identity_networks(
			origin: OriginFor<T>,
			who: Identity,
			identity: Identity,
			web3networks: Vec<Web3Network>,
		) -> DispatchResult {
			T::ManageOrigin::ensure_origin(origin)?;
			Self::maybe_create_id_graph(&who)?;
			ensure!(IDGraphs::<T>::contains_key(&who, &identity), Error::<T>::IdentityNotExist);

			IDGraphs::<T>::try_mutate(&who, &identity, |context| {
				let mut c = context.take().ok_or(Error::<T>::IdentityNotExist)?;
				ensure!(
					identity.matches_web3networks(web3networks.as_ref()),
					Error::<T>::WrongWeb3NetworkTypes
				);
				c.set_web3networks(web3networks);
				*context = Some(c);
				Ok(())
			})
		}

		#[pallet::call_index(5)]
		#[pallet::weight({15_000_000})]
		pub fn remove_identity(
			origin: OriginFor<T>,
			who: Identity,
			identities: Vec<Identity>,
		) -> DispatchResult {
			T::ManageOrigin::ensure_origin(origin)?;

			if !identities.is_empty() {
				for identity in identities.iter() {
					ensure!(
						LinkedIdentities::<T>::contains_key(identity),
						Error::<T>::IdentityNotExist
					);
					ensure!(
						IDGraphs::<T>::contains_key(&who, identity),
						Error::<T>::IdentityNotExist
					);

					IDGraphs::<T>::remove(&who, identity);
					LinkedIdentities::<T>::remove(identity);
				}
			} else {
				// removing prime identity with all linked identities
				IDGraphs::iter_prefix(&who)
					.collect::<IDGraph<T>>()
					.iter()
					.for_each(|(identity, _context)| LinkedIdentities::<T>::remove(identity));
				let _ = IDGraphs::<T>::clear_prefix(&who, u32::MAX, None);
			}

			Ok(())
		}

		// Clean all id_graph related storage
		#[pallet::call_index(6)]
		#[pallet::weight({15_000_000})]
		pub fn clean_id_graphs(origin: OriginFor<T>) -> DispatchResult {
			T::ManageOrigin::ensure_origin(origin)?;

			Self::clear_id_graphs();
			Self::clear_id_graph_lens();
			Self::clear_linked_identities();

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// try to create an IDGraph if there's none - `who` will be the prime identity
		// please note the web3networks for the prime identity will be all avaiable networks
		pub fn maybe_create_id_graph(who: &Identity) -> Result<(), DispatchError> {
			if IDGraphs::<T>::get(who, who).is_none() {
				ensure!(
					!LinkedIdentities::<T>::contains_key(who),
					Error::<T>::IdentityAlreadyLinked
				);

				let context = <IdentityContext<T>>::new(
					<T as frame_system::Config>::BlockNumber::one(),
					who.default_web3networks(),
				);

				Self::insert_identity_with_limit(who, who, context)?;
			}
			Ok(())
		}

		fn insert_identity_with_limit(
			owner: &Identity,
			identity: &Identity,
			context: IdentityContext<T>,
		) -> Result<(), DispatchError> {
			IDGraphLens::<T>::try_mutate(owner, |len| {
				let new_len = len.checked_add(1).ok_or(Error::<T>::IDGraphLenLimitReached)?;
				if new_len > T::MaxIDGraphLength::get() {
					return Err(Error::<T>::IDGraphLenLimitReached.into())
				}
				*len = new_len;
				Result::<(), DispatchError>::Ok(())
			})?;
			LinkedIdentities::<T>::insert(identity, ());
			IDGraphs::<T>::insert(owner, identity, context);
			Ok(())
		}

		// get the whole IDGraph, sorted by `link_block` (earliest -> latest)
		//
		// TODO: shall we change the return type to Option<IDGraph<T>> and return
		//       `None` if the IDGraph doesn't exist?
		pub fn id_graph(who: &Identity) -> IDGraph<T> {
			let mut id_graph = IDGraphs::iter_prefix(who).collect::<IDGraph<T>>();

			// Initial sort to ensure a deterministic order
			sort_id_graph::<T>(&mut id_graph);

			id_graph
		}

		// get the IDGraph hash of the given `who`
		pub fn id_graph_hash(who: &Identity) -> Option<H256> {
			let id_graph = Self::id_graph(who);
			if id_graph.is_empty() {
				None
			} else {
				Some(H256::from(blake2_256(&id_graph.encode())))
			}
		}

		// get count of all keys account + identity in the IDGraphs
		pub fn id_graph_stats() -> Option<Vec<(Identity, u32)>> {
			let stats = IDGraphLens::<T>::iter().collect();
			debug!("IDGraph stats: {:?}", stats);
			Some(stats)
		}

		fn clear_id_graphs() {
			// Retrieve all the outer and inner keys from the storage by collecting tuples of (outer_key, inner_key)
			let keys: Vec<(Identity, Identity)> = IDGraphs::<T>::iter()
				.map(|(outer_key, inner_key, _)| (outer_key, inner_key))
				.collect();

			// Iterate through all the key pairs (outer_key, inner_key) and remove the corresponding entries from storage
			for (outer_key, inner_key) in keys {
				IDGraphs::<T>::remove(outer_key, inner_key);
			}
		}

		fn clear_id_graph_lens() {
			// Retrieve all the keys from the storage
			let keys: Vec<Identity> = IDGraphLens::<T>::iter_keys().collect();

			// Iterate through each key and remove the entry
			for key in keys {
				IDGraphLens::<T>::remove(key);
			}
		}

		fn clear_linked_identities() {
			// Retrieve all the keys from the storage
			let keys: Vec<Identity> = LinkedIdentities::<T>::iter_keys().collect();

			// Iterate through each key and remove the entry
			for key in keys {
				LinkedIdentities::<T>::remove(key);
			}
		}
	}
}
