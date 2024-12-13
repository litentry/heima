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

//! A pallet to serve as the interface for F/E for verifiable credentials (VC)
//! management. Similar to IMP pallet, the actual processing will be done within TEE.

// TODO: benchmark and weights: we need worst-case scenarios

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

mod schema;
pub use schema::*;

pub type VCIndex = H256;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use core_primitives::{
		Assertion, ErrorDetail, Identity, SchemaIndex, ShardIdentifier, VCMPError,
		SCHEMA_CONTENT_LEN, SCHEMA_ID_LEN,
	};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;
		// some extrinsics should only be called by origins from TEE
		type TEECallOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		// origin who can set the admin account
		type SetAdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		// origin to manage authorized delegatee list
		type DelegateeAdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		// origin that is allowed to call extrinsics
		type ExtrinsicWhitelistOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
	}

	// the admin account
	#[pallet::storage]
	#[pallet::getter(fn admin)]
	pub type Admin<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	// delegatees who can request (and receive) VCs on users' behalf,
	// some VCs can only be requested by delegatee accounts (e.g. A13)
	// delegatees and admins are different:
	// - admins are meant to manage the pallet state manually, e.g. schema
	// - delegatees can request VCs for users, similar to `proxied account`
	#[pallet::storage]
	#[pallet::getter(fn delegatee)]
	pub type Delegatee<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn schema_index)]
	pub type SchemaRegistryIndex<T: Config> = StorageValue<_, SchemaIndex, ValueQuery>;

	// the VC Schema storage
	#[pallet::storage]
	#[pallet::getter(fn schema_registry)]
	pub type SchemaRegistry<T: Config> = StorageMap<_, Blake2_128Concat, SchemaIndex, VCSchema<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		DelegateeAdded {
			account: T::AccountId,
		},
		DelegateeRemoved {
			account: T::AccountId,
		},
		// a VC is requested
		VCRequested {
			account: T::AccountId,
			shard: ShardIdentifier,
			assertion: Assertion,
		},
		// event that should be triggered by TEECallOrigin
		// a VC is just issued
		// we have `id_graph_hash` field since vc request could create the IDGraph
		VCIssued {
			identity: Identity,
			assertion: Assertion,
			id_graph_hash: H256,
			req_ext_hash: H256,
		},
		// Admin account was changed
		AdminChanged {
			old_admin: Option<T::AccountId>,
			new_admin: Option<T::AccountId>,
		},
		// a Schema is issued
		SchemaIssued {
			account: T::AccountId,
			shard: ShardIdentifier,
			index: SchemaIndex,
		},
		// a Schema is disabled
		SchemaDisabled {
			account: T::AccountId,
			shard: ShardIdentifier,
			index: SchemaIndex,
		},
		// a Schema is activated
		SchemaActivated {
			account: T::AccountId,
			shard: ShardIdentifier,
			index: SchemaIndex,
		},
		// a Schema is revoked
		SchemaRevoked {
			account: T::AccountId,
			shard: ShardIdentifier,
			index: SchemaIndex,
		},
		// event errors caused by processing in TEE
		// copied from core_primitives::VCMPError, we use events instead of pallet::errors,
		// see https://github.com/litentry/litentry-parachain/issues/1275
		RequestVCFailed {
			identity: Option<Identity>,
			assertion: Assertion,
			detail: ErrorDetail,
			req_ext_hash: H256,
		},
		UnclassifiedError {
			identity: Option<Identity>,
			detail: ErrorDetail,
			req_ext_hash: H256,
		},
		VCIssuedNew {
			identity: Identity,
			assertion: Assertion,
			omni_account: T::AccountId,
			req_ext_hash: H256,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// a delegatee doesn't exist
		DelegateeNotExist,
		/// a `request_vc` request from unauthorized user
		UnauthorizedUser,
		/// the VC already exists
		VCAlreadyExists,
		/// the ID doesn't exist
		VCNotExist,
		/// The requester doesn't have the permission (because of subject mismatch)
		VCSubjectMismatch,
		/// The VC is already disabled
		VCAlreadyDisabled,
		/// Error when the caller account is not the admin
		RequireAdmin,
		/// Schema not exists
		SchemaNotExists,
		/// Schema is already disabled
		SchemaAlreadyDisabled,
		/// Schema is active
		SchemaAlreadyActivated,
		SchemaIndexOverFlow,
		LengthMismatch,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub admin: Option<T::AccountId>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { admin: None }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			if let Some(ref admin) = self.admin {
				Admin::<T>::put(admin);
			}
		}
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

		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::request_vc())]
		pub fn request_vc(
			origin: OriginFor<T>,
			shard: ShardIdentifier,
			assertion: Assertion,
		) -> DispatchResultWithPostInfo {
			let who = T::ExtrinsicWhitelistOrigin::ensure_origin(origin)?;
			// special handling for A13, where the origin is required to be one of the delegatees
			if let Assertion::A13(_owner) = assertion.clone() {
				ensure!(Delegatee::<T>::contains_key(&who), Error::<T>::UnauthorizedUser);
			}
			Self::deposit_event(Event::VCRequested { account: who, shard, assertion });
			Ok(().into())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::set_admin())]
		pub fn set_admin(origin: OriginFor<T>, new: T::AccountId) -> DispatchResultWithPostInfo {
			T::SetAdminOrigin::ensure_origin(origin)?;
			Self::deposit_event(Event::AdminChanged {
				old_admin: Self::admin(),
				new_admin: Some(new.clone()),
			});
			<Admin<T>>::put(new);
			Ok(Pays::No.into())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(<T as Config>::WeightInfo::add_schema())]
		pub fn add_schema(
			origin: OriginFor<T>,
			shard: ShardIdentifier,
			id: Vec<u8>,
			content: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			ensure!(Some(sender.clone()) == Self::admin(), Error::<T>::RequireAdmin);
			ensure!((id.len() as u32) <= SCHEMA_ID_LEN, Error::<T>::LengthMismatch);
			ensure!((content.len() as u32) <= SCHEMA_CONTENT_LEN, Error::<T>::LengthMismatch);

			let index = Self::schema_index();
			<SchemaRegistryIndex<T>>::put(
				index.checked_add(1u64).ok_or(Error::<T>::SchemaIndexOverFlow)?,
			);

			SchemaRegistry::<T>::insert(
				index,
				VCSchema::<T>::new(id.clone(), sender.clone(), content.clone()),
			);
			Self::deposit_event(Event::SchemaIssued { account: sender, shard, index });
			Ok(().into())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(<T as Config>::WeightInfo::disable_schema())]
		pub fn disable_schema(
			origin: OriginFor<T>,
			shard: ShardIdentifier,
			index: SchemaIndex,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			ensure!(Some(sender.clone()) == Self::admin(), Error::<T>::RequireAdmin);
			SchemaRegistry::<T>::try_mutate(index, |context| {
				let mut c = context.take().ok_or(Error::<T>::SchemaNotExists)?;
				ensure!(c.status == Status::Active, Error::<T>::SchemaAlreadyDisabled);
				c.status = Status::Disabled;
				*context = Some(c);
				Self::deposit_event(Event::SchemaDisabled { account: sender, shard, index });
				Ok(().into())
			})
		}

		#[pallet::call_index(8)]
		#[pallet::weight(<T as Config>::WeightInfo::activate_schema())]
		pub fn activate_schema(
			origin: OriginFor<T>,
			shard: ShardIdentifier,
			index: SchemaIndex,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			ensure!(Some(sender.clone()) == Self::admin(), Error::<T>::RequireAdmin);
			SchemaRegistry::<T>::try_mutate(index, |context| {
				let mut c = context.take().ok_or(Error::<T>::SchemaNotExists)?;
				ensure!(c.status == Status::Disabled, Error::<T>::SchemaAlreadyActivated);
				c.status = Status::Active;
				*context = Some(c);
				Self::deposit_event(Event::SchemaActivated { account: sender, shard, index });
				Ok(().into())
			})
		}

		#[pallet::call_index(9)]
		#[pallet::weight(<T as Config>::WeightInfo::revoke_schema())]
		pub fn revoke_schema(
			origin: OriginFor<T>,
			shard: ShardIdentifier,
			index: SchemaIndex,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			ensure!(Some(sender.clone()) == Self::admin(), Error::<T>::RequireAdmin);
			let _ = SchemaRegistry::<T>::get(index).ok_or(Error::<T>::SchemaNotExists)?;
			SchemaRegistry::<T>::remove(index);
			Self::deposit_event(Event::SchemaRevoked { account: sender, shard, index });
			Ok(().into())
		}

		/// ---------------------------------------------------
		/// The following extrinsics are supposed to be called by TEE only
		/// ---------------------------------------------------
		#[pallet::call_index(30)]
		#[pallet::weight(<T as Config>::WeightInfo::vc_issued())]
		pub fn vc_issued(
			origin: OriginFor<T>,
			identity: Identity,
			assertion: Assertion,
			id_graph_hash: H256,
			req_ext_hash: H256,
		) -> DispatchResultWithPostInfo {
			let _ = T::TEECallOrigin::ensure_origin(origin)?;
			Self::deposit_event(Event::VCIssued {
				identity,
				assertion,
				id_graph_hash,
				req_ext_hash,
			});
			Ok(Pays::No.into())
		}

		#[pallet::call_index(31)]
		#[pallet::weight(<T as Config>::WeightInfo::some_error())]
		pub fn some_error(
			origin: OriginFor<T>,
			identity: Option<Identity>,
			error: VCMPError,
			req_ext_hash: H256,
		) -> DispatchResultWithPostInfo {
			let _ = T::TEECallOrigin::ensure_origin(origin)?;
			match error {
				VCMPError::RequestVCFailed(assertion, detail) => {
					Self::deposit_event(Event::RequestVCFailed {
						identity,
						assertion,
						detail,
						req_ext_hash,
					})
				},
				VCMPError::UnclassifiedError(detail) => {
					Self::deposit_event(Event::UnclassifiedError { identity, detail, req_ext_hash })
				},
			}
			Ok(Pays::No.into())
		}

		// TODO: update the weight info to use on_vc_issued
		#[pallet::call_index(32)]
		#[pallet::weight(<T as Config>::WeightInfo::vc_issued())]
		pub fn on_vc_issued(
			origin: OriginFor<T>,
			identity: Identity,
			assertion: Assertion,
			omni_account: T::AccountId,
			req_ext_hash: H256,
		) -> DispatchResultWithPostInfo {
			let _ = T::TEECallOrigin::ensure_origin(origin)?;
			Self::deposit_event(Event::VCIssuedNew {
				identity,
				assertion,
				omni_account,
				req_ext_hash,
			});
			Ok(Pays::No.into())
		}
	}
}
