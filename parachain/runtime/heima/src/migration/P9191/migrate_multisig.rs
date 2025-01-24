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
	pallet_prelude::*,
	traits::{Get, OnRuntimeUpgrade},
};
use sp_std::{marker::PhantomData, vec::Vec};

#[cfg(feature = "try-runtime")]
use parity_scale_codec::Encode;

use pallet_multisig::Multisigs;

// Replace Parachain Staking Storage for Decimal Change from 12 to 18
pub struct ReplacePalletMultisigStorage<T>(PhantomData<T>);

impl<T> ReplacePalletMultisigStorage<T>
where
	T: pallet_multisig::Config,
{
	// pallet_multisig
	pub fn check_multisig_multisigs_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplacePalletMultisigStorage",
			"Running checking to Multisig - Multisigs"
		);

		assert!(Multisigs::<T>::iter().next().is_none());

		let weight = T::DbWeight::get();
		frame_support::weights::Weight::from_parts(0, weight.read)
	}
}

#[cfg(feature = "try-runtime")]
impl<T> ReplacePalletMultisigStorage<T>
where
	T: pallet_multisig::Config,
{
	pub fn pre_upgrade_multisig_multisigs_storage() -> Result<Vec<u8>, &'static str> {
		assert!(Multisigs::<T>::iter().next().is_none());
		Ok(Vec::<u8>::new())
	}

	pub fn post_upgrade_multisig_multisigs_storage(_state: Vec<u8>) -> Result<(), &'static str> {
		assert!(Multisigs::<T>::iter().next().is_none());
		Ok(())
	}
}

impl<T> OnRuntimeUpgrade for ReplacePalletMultisigStorage<T>
where
	T: frame_system::Config + pallet_multisig::Config,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		// pallet_multisig
		let multisigs_vec = Self::pre_upgrade_multisig_multisigs_storage()?;

		Ok((multisigs_vec,).encode())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		frame_support::weights::Weight::zero()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		let pre_vec: (Vec<u8>,) =
			Decode::decode(&mut &state[..]).map_err(|_| "Failed to decode Tuple")?;

		// pallet_multisig
		Self::post_upgrade_multisig_multisigs_storage(pre_vec.0)?;
		Ok(())
	}
}
