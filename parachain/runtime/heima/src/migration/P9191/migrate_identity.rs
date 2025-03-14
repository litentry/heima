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
#![allow(clippy::type_complexity)]

use frame_support::{
	migration::storage_key_iter,
	pallet_prelude::*,
	traits::{Currency, Get, OnRuntimeUpgrade},
	Twox64Concat,
};
use sp_std::{marker::PhantomData, vec::Vec};

#[cfg(feature = "try-runtime")]
use parity_scale_codec::{Decode, Encode};

use pallet_identity::{RegistrarInfo, Registration};
use storage::migration::get_storage_value;
type BalanceOf<T> = <<T as pallet_identity::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

// Replace Parachain Staking Storage for Decimal Change from 12 to 18
pub struct ReplacePalletIdentityStorage<T>(PhantomData<T>);

impl<T> ReplacePalletIdentityStorage<T>
where
	T: pallet_identity::Config,
{
	// pallet_identity
	pub fn check_identityof_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplacePalletIdentityStorage",
			"Running check to ParachainIdentity IdentityOf"
		);
		let pallet_prefix: &[u8] = b"ParachainIdentity";
		let storage_item_prefix: &[u8] = b"IdentityOf";

		assert!(storage_key_iter::<
			T::AccountId,
			Registration<BalanceOf<T>, T::MaxRegistrars, T::MaxAdditionalFields>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.next()
		.is_none());
		let weight = T::DbWeight::get();
		frame_support::weights::Weight::from_parts(0, weight.read)
	}

	pub fn check_subsof_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplacePalletIdentityStorage",
			"Running check to ParachainIdentity SubsOf"
		);
		let pallet_prefix: &[u8] = b"ParachainIdentity";
		let storage_item_prefix: &[u8] = b"SubsOf";

		assert!(storage_key_iter::<
			T::AccountId,
			Registration<BalanceOf<T>, T::MaxRegistrars, T::MaxAdditionalFields>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.next()
		.is_none());
		let weight = T::DbWeight::get();
		frame_support::weights::Weight::from_parts(0, weight.read)
	}

	pub fn check_registrars_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplacePalletIdentityStorage",
			"Running check to ParachainIdentity Registrars"
		);
		let pallet_prefix: &[u8] = b"ParachainIdentity";
		let storage_item_prefix: &[u8] = b"Registrars";

		assert!(get_storage_value::<
			BoundedVec<Option<RegistrarInfo<BalanceOf<T>, T::AccountId>>, T::MaxRegistrars>,
		>(pallet_prefix, storage_item_prefix, b"")
		.is_none());

		let weight = T::DbWeight::get();
		frame_support::weights::Weight::from_parts(0, weight.read)
	}
}

#[cfg(feature = "try-runtime")]
impl<T> ReplacePalletIdentityStorage<T>
where
	T: pallet_identity::Config,
{
	// pallet_identity
	pub fn pre_upgrade_identityof_storage() -> Result<Vec<u8>, &'static str> {
		let pallet_prefix: &[u8] = b"ParachainIdentity";
		let storage_item_prefix: &[u8] = b"IdentityOf";

		assert!(storage_key_iter::<
			T::AccountId,
			Registration<BalanceOf<T>, T::MaxRegistrars, T::MaxAdditionalFields>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.next()
		.is_none());
		Ok(Vec::<u8>::new())
	}
	pub fn post_upgrade_identityof_storage(_state: Vec<u8>) -> Result<(), &'static str> {
		let pallet_prefix: &[u8] = b"ParachainIdentity";
		let storage_item_prefix: &[u8] = b"IdentityOf";

		assert!(storage_key_iter::<
			T::AccountId,
			Registration<BalanceOf<T>, T::MaxRegistrars, T::MaxAdditionalFields>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.next()
		.is_none());
		Ok(())
	}
	pub fn pre_upgrade_subsof_storage() -> Result<Vec<u8>, &'static str> {
		let pallet_prefix: &[u8] = b"ParachainIdentity";
		let storage_item_prefix: &[u8] = b"SubsOf";

		assert!(storage_key_iter::<
			T::AccountId,
			Registration<BalanceOf<T>, T::MaxRegistrars, T::MaxAdditionalFields>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.next()
		.is_none());
		Ok(Vec::<u8>::new())
	}
	pub fn post_upgrade_subsof_storage(_state: Vec<u8>) -> Result<(), &'static str> {
		let pallet_prefix: &[u8] = b"ParachainIdentity";
		let storage_item_prefix: &[u8] = b"SubsOf";

		assert!(storage_key_iter::<
			T::AccountId,
			Registration<BalanceOf<T>, T::MaxRegistrars, T::MaxAdditionalFields>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.next()
		.is_none());

		Ok(())
	}
	pub fn pre_upgrade_registrars_storage() -> Result<Vec<u8>, &'static str> {
		let pallet_prefix: &[u8] = b"ParachainIdentity";
		let storage_item_prefix: &[u8] = b"Registrars";

		assert!(get_storage_value::<
			BoundedVec<Option<RegistrarInfo<BalanceOf<T>, T::AccountId>>, T::MaxRegistrars>,
		>(pallet_prefix, storage_item_prefix, b"")
		.is_none());
		Ok(Vec::<u8>::new())
	}
	pub fn post_upgrade_registrars_storage(_state: Vec<u8>) -> Result<(), &'static str> {
		let pallet_prefix: &[u8] = b"ParachainIdentity";
		let storage_item_prefix: &[u8] = b"Registrars";

		assert!(get_storage_value::<
			BoundedVec<Option<RegistrarInfo<BalanceOf<T>, T::AccountId>>, T::MaxRegistrars>,
		>(pallet_prefix, storage_item_prefix, b"")
		.is_none());
		Ok(())
	}
}

impl<T> OnRuntimeUpgrade for ReplacePalletIdentityStorage<T>
where
	T: frame_system::Config + pallet_identity::Config,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		// pallet_identity
		let identityof_vec = Self::pre_upgrade_identityof_storage()?;
		let subsof_vec = Self::pre_upgrade_subsof_storage()?;
		let registrars_vec = Self::pre_upgrade_registrars_storage()?;

		Ok((identityof_vec, subsof_vec, registrars_vec).encode())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		frame_support::weights::Weight::zero()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		let pre_vec: (Vec<u8>, Vec<u8>, Vec<u8>) =
			Decode::decode(&mut &state[..]).map_err(|_| "Failed to decode Tuple")?;
		// pallet_identity
		Self::post_upgrade_identityof_storage(pre_vec.0)?;
		Self::post_upgrade_subsof_storage(pre_vec.1)?;
		Self::post_upgrade_registrars_storage(pre_vec.2)?;

		Ok(())
	}
}
