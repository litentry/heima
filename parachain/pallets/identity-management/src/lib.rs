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

//! TODO: event/error handling
//! Currently the errors are synchronously emitted from this pallet itself,
//! meanwhile we have the `SomeError` **Event** which is callable from TEE
//! to represent any generic "error".
//! However, there are so many error cases in TEE that I'm not even sure
//! if it's a good idea to have a matching extrinsic for error propagation.
//!
//! The reasons that we don't use pallet_teebag::post_opaque_task directly are:
//! - call pallet_teebag::post_opaque_task inside IMP won't trigger the handler, because it's not
//!   called as extrinsics so won't be scraped
//! - the origin is discarded in call_worker but we need it
//! - to simplify the F/E usage, we only need to encrypt the needed parameters (see e.g.
//!   shield_funds)

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused_variables)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;

pub use crate::weights::WeightInfo;
pub use pallet::*;

use sp_core::H256;
use sp_std::vec::Vec;

const MAX_REDIRECT_URL_LEN: u32 = 256;

#[frame_support::pallet]
pub mod pallet {
	use super::{Vec, WeightInfo, H256, MAX_REDIRECT_URL_LEN};
	use core_primitives::{ErrorDetail, IMPError, Identity, ShardIdentifier};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[derive(
		Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen,
	)]
	#[scale_info(skip_type_params(MaxOIDCClientUris, MaxRedirectUriLen))]
	pub struct OIDCClient<MaxOIDCClientUris: Get<u32>, MaxRedirectUriLen: Get<u32>> {
		redirect_uris: BoundedVec<BoundedVec<u8, MaxRedirectUriLen>, MaxOIDCClientUris>,
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;
		// some extrinsics should only be called by origins from TEE
		type TEECallOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		// origin to manage authorized delegatee list
		type DelegateeAdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		// origin that is allowed to call extrinsics
		type ExtrinsicWhitelistOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
		// maximum number of OIDC client URIs
		#[pallet::constant]
		type MaxOIDCClientRedirectUris: Get<u32>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn integrity_test() {
			assert!(
				<T as Config>::MaxOIDCClientRedirectUris::get() > 0,
				"MaxOIDCClientUris must be greater than zero."
			);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		DelegateeAdded {
			account: T::AccountId,
		},
		DelegateeRemoved {
			account: T::AccountId,
		},
		// TODO: do we need account as event parameter? This needs to be decided by F/E
		LinkIdentityRequested {
			shard: ShardIdentifier,
			account: T::AccountId,
			encrypted_identity: Vec<u8>,
			encrypted_validation_data: Vec<u8>,
			encrypted_web3networks: Vec<u8>,
		},
		DeactivateIdentityRequested {
			shard: ShardIdentifier,
			account: T::AccountId,
			encrypted_identity: Vec<u8>,
		},
		ActivateIdentityRequested {
			shard: ShardIdentifier,
			account: T::AccountId,
			encrypted_identity: Vec<u8>,
		},
		// event that should be triggered by TEECallOrigin
		// we return the request-extrinsic-hash for better tracking
		// TODO: what if the event is triggered by an extrinsic that is included in a batch call?
		//       Can we retrieve that extrinsic hash in F/E?
		IdentityLinked {
			prime_identity: Identity,
			id_graph_hash: H256,
			req_ext_hash: H256,
		},
		IdentityDeactivated {
			prime_identity: Identity,
			id_graph_hash: H256,
			req_ext_hash: H256,
		},
		IdentityActivated {
			prime_identity: Identity,
			id_graph_hash: H256,
			req_ext_hash: H256,
		},
		IdentityNetworksSet {
			prime_identity: Identity,
			id_graph_hash: H256,
			req_ext_hash: H256,
		},
		// event errors caused by processing in TEE
		// copied from core_primitives::IMPError, we use events instead of pallet::errors,
		// see https://github.com/litentry/heima/issues/1275
		//
		// why is the `prime_identity` in the error event an Option?
		// because in some erroneous cases we can't get the extrinsic sender (e.g. decode error)
		LinkIdentityFailed {
			prime_identity: Option<Identity>,
			detail: ErrorDetail,
			req_ext_hash: H256,
		},
		DeactivateIdentityFailed {
			prime_identity: Option<Identity>,
			detail: ErrorDetail,
			req_ext_hash: H256,
		},
		ActivateIdentityFailed {
			prime_identity: Option<Identity>,
			detail: ErrorDetail,
			req_ext_hash: H256,
		},
		UnclassifiedError {
			prime_identity: Option<Identity>,
			detail: ErrorDetail,
			req_ext_hash: H256,
		},
		OIDCClientRegistered {
			client_id: T::AccountId,
		},
		OIDCClientUnregistered {
			client_id: T::AccountId,
		},
	}

	// delegatees who can send extrinsics(currently only `link_identity`) on users' behalf
	#[pallet::storage]
	#[pallet::getter(fn delegatee)]
	pub type Delegatee<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, (), OptionQuery>;

	// OIDC clients who can use the OIDC flow
	#[pallet::storage]
	#[pallet::getter(fn oidc_client)]
	pub type OIDCClients<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		OIDCClient<T::MaxOIDCClientRedirectUris, ConstU32<MAX_REDIRECT_URL_LEN>>,
		OptionQuery,
	>;

	#[pallet::error]
	pub enum Error<T> {
		/// a delegatee doesn't exist
		DelegateeNotExist,
		/// a `link_identity` request from unauthorized user
		UnauthorizedUser,
		/// redirect_uris exceed the maximum length
		TooManyRedirectUris,
		/// redirect_uris is empty
		EmptyRedirectUris,
		/// redirect_uri exceeds the maximum length
		RedirectUriTooLong,
		/// OIDC client already exists
		OIDCClientAlreadyRegistered,
		/// OIDC client does not exists
		OIDCClientDoesNotExist,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// add an account to the delegatees
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::add_delegatee())]
		pub fn add_delegatee(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			let _ = T::DelegateeAdminOrigin::ensure_origin(origin)?;
			// we don't care if `account` already exists
			Delegatee::<T>::insert(account.clone(), ());
			Self::deposit_event(Event::DelegateeAdded { account });
			Ok(())
		}

		/// remove an account from the delegatees
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::remove_delegatee())]
		pub fn remove_delegatee(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			let _ = T::DelegateeAdminOrigin::ensure_origin(origin)?;
			ensure!(Delegatee::<T>::contains_key(&account), Error::<T>::DelegateeNotExist);
			Delegatee::<T>::remove(account.clone());
			Self::deposit_event(Event::DelegateeRemoved { account });
			Ok(())
		}

		/// Link an identity with given network types and validation data.
		/// We do the origin check for this extrinsic, it has to be
		/// - either the caller themselves, i.e. ensure_signed(origin)? == who
		/// - or from a delegatee in the list
		///
		/// `encrypted_web3networks`:
		/// an initial list of web3 networks on which the identity is used:
		/// Vec<Web3Network> encrypted with TEE's shielding key. In fact, it
		/// doesn't have to be encrypted as it's a finite set and you can sort
		/// it out by enumerating all possible combinations. But still, to keep
		/// it consistent with identities and validation data.
		///
		/// The networks must match the identity type, it means:
		/// - for web2 identity, Vec<Web3Network> must be empty;
		/// - for substrate identity, Vec<Web3Network> must have substrate networks only;
		/// - for evm identity, Vec<Web3Network> must have evm networks only.
		/// Otherwise the linking will fail.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::link_identity())]
		pub fn link_identity(
			origin: OriginFor<T>,
			shard: ShardIdentifier,
			user: T::AccountId,
			encrypted_identity: Vec<u8>,
			encrypted_validation_data: Vec<u8>,
			encrypted_web3networks: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = T::ExtrinsicWhitelistOrigin::ensure_origin(origin)?;
			ensure!(
				who == user || Delegatee::<T>::contains_key(&who),
				Error::<T>::UnauthorizedUser
			);
			Self::deposit_event(Event::LinkIdentityRequested {
				shard,
				account: user,
				encrypted_identity,
				encrypted_validation_data,
				encrypted_web3networks,
			});
			Ok(().into())
		}

		/// Deactivate an identity
		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config>::WeightInfo::deactivate_identity())]
		pub fn deactivate_identity(
			origin: OriginFor<T>,
			shard: ShardIdentifier,
			encrypted_identity: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = T::ExtrinsicWhitelistOrigin::ensure_origin(origin)?;
			Self::deposit_event(Event::DeactivateIdentityRequested {
				shard,
				account: who,
				encrypted_identity,
			});
			Ok(().into())
		}

		/// Activate an identity
		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::activate_identity())]
		pub fn activate_identity(
			origin: OriginFor<T>,
			shard: ShardIdentifier,
			encrypted_identity: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = T::ExtrinsicWhitelistOrigin::ensure_origin(origin)?;
			Self::deposit_event(Event::ActivateIdentityRequested {
				shard,
				account: who,
				encrypted_identity,
			});
			Ok(().into())
		}

		/// Register an OIDC client
		// TODO: take a deposit to cover the storage cost and prevent spamming
		#[pallet::call_index(6)]
		#[pallet::weight((195_000_000, DispatchClass::Normal))]
		pub fn register_oidc_client(
			origin: OriginFor<T>,
			redirect_uris: Vec<Vec<u8>>,
		) -> DispatchResult {
			let client_id = ensure_signed(origin)?;
			ensure!(!redirect_uris.is_empty(), Error::<T>::EmptyRedirectUris);
			ensure!(
				!OIDCClients::<T>::contains_key(&client_id),
				Error::<T>::OIDCClientAlreadyRegistered
			);

			let client_redirect_uris = redirect_uris
				.into_iter()
				.map(|uri| {
					BoundedVec::<u8, _>::try_from(uri).map_err(|_| Error::<T>::RedirectUriTooLong)
				})
				.collect::<Result<Vec<_>, _>>()?;

			OIDCClients::<T>::insert(
				&client_id,
				OIDCClient {
					redirect_uris: BoundedVec::try_from(client_redirect_uris)
						.map_err(|_| Error::<T>::TooManyRedirectUris)?,
				},
			);

			Self::deposit_event(Event::OIDCClientRegistered { client_id });

			Ok(())
		}

		/// Unregister an OIDC client
		#[pallet::call_index(7)]
		#[pallet::weight((195_000_000, DispatchClass::Normal))]
		pub fn unregister_oidc_client(origin: OriginFor<T>) -> DispatchResult {
			let client_id = ensure_signed(origin)?;
			ensure!(OIDCClients::<T>::contains_key(&client_id), Error::<T>::OIDCClientDoesNotExist);
			OIDCClients::<T>::remove(&client_id);
			Self::deposit_event(Event::OIDCClientUnregistered { client_id });

			Ok(())
		}

		/// ---------------------------------------------------
		/// The following extrinsics are supposed to be called by TEE only
		/// ---------------------------------------------------
		#[pallet::call_index(31)]
		#[pallet::weight(<T as Config>::WeightInfo::identity_linked())]
		pub fn identity_linked(
			origin: OriginFor<T>,
			prime_identity: Identity,
			id_graph_hash: H256,
			req_ext_hash: H256,
		) -> DispatchResultWithPostInfo {
			let _ = T::TEECallOrigin::ensure_origin(origin)?;
			Self::deposit_event(Event::IdentityLinked {
				prime_identity,
				id_graph_hash,
				req_ext_hash,
			});
			Ok(Pays::No.into())
		}

		#[pallet::call_index(32)]
		#[pallet::weight(<T as Config>::WeightInfo::identity_deactivated())]
		pub fn identity_deactivated(
			origin: OriginFor<T>,
			prime_identity: Identity,
			id_graph_hash: H256,
			req_ext_hash: H256,
		) -> DispatchResultWithPostInfo {
			let _ = T::TEECallOrigin::ensure_origin(origin)?;
			Self::deposit_event(Event::IdentityDeactivated {
				prime_identity,
				id_graph_hash,
				req_ext_hash,
			});
			Ok(Pays::No.into())
		}

		#[pallet::call_index(33)]
		#[pallet::weight(<T as Config>::WeightInfo::identity_activated())]
		pub fn identity_activated(
			origin: OriginFor<T>,
			prime_identity: Identity,
			id_graph_hash: H256,
			req_ext_hash: H256,
		) -> DispatchResultWithPostInfo {
			let _ = T::TEECallOrigin::ensure_origin(origin)?;
			Self::deposit_event(Event::IdentityActivated {
				prime_identity,
				id_graph_hash,
				req_ext_hash,
			});
			Ok(Pays::No.into())
		}

		#[pallet::call_index(34)]
		#[pallet::weight(<T as Config>::WeightInfo::identity_networks_set())]
		pub fn identity_networks_set(
			origin: OriginFor<T>,
			prime_identity: Identity,
			id_graph_hash: H256,
			req_ext_hash: H256,
		) -> DispatchResultWithPostInfo {
			let _ = T::TEECallOrigin::ensure_origin(origin)?;
			Self::deposit_event(Event::IdentityNetworksSet {
				prime_identity,
				id_graph_hash,
				req_ext_hash,
			});
			Ok(Pays::No.into())
		}

		#[pallet::call_index(35)]
		#[pallet::weight(<T as Config>::WeightInfo::some_error())]
		pub fn some_error(
			origin: OriginFor<T>,
			prime_identity: Option<Identity>,
			error: IMPError,
			req_ext_hash: H256,
		) -> DispatchResultWithPostInfo {
			let _ = T::TEECallOrigin::ensure_origin(origin)?;
			match error {
				IMPError::LinkIdentityFailed(detail) => {
					Self::deposit_event(Event::LinkIdentityFailed {
						prime_identity,
						detail,
						req_ext_hash,
					})
				},
				IMPError::DeactivateIdentityFailed(detail) => {
					Self::deposit_event(Event::DeactivateIdentityFailed {
						prime_identity,
						detail,
						req_ext_hash,
					})
				},
				IMPError::ActivateIdentityFailed(detail) => {
					Self::deposit_event(Event::ActivateIdentityFailed {
						prime_identity,
						detail,
						req_ext_hash,
					})
				},
				IMPError::UnclassifiedError(detail) => {
					Self::deposit_event(Event::UnclassifiedError {
						prime_identity,
						detail,
						req_ext_hash,
					})
				},
			}
			Ok(Pays::No.into())
		}
	}
}
