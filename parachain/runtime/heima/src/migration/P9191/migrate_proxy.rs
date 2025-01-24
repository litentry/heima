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
use sp_runtime::{traits::Hash, Saturating};
use sp_std::{marker::PhantomData, vec::Vec};

pub const DECIMAL_CONVERTOR: u32 = 1_000_000;

#[cfg(feature = "try-runtime")]
use parity_scale_codec::Encode;
#[cfg(feature = "try-runtime")]
use sp_std::collections::btree_map::BTreeMap;

use pallet_proxy::{Announcement, Announcements, Proxies, ProxyDefinition};
type BalanceOf<T> = <<T as pallet_proxy::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

type CallHashOf<T> = <<T as pallet_proxy::Config>::CallHasher as Hash>::Output;

pub struct ReplacePalletProxyStorage<T>(PhantomData<T>);

impl<T> ReplacePalletProxyStorage<T>
where
	T: pallet_proxy::Config,
{
	// pallet_proxy
	pub fn replace_proxy_proxies_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplacePalletProxyStorage",
			"Running migration to Proxy - Proxies"
		);

		let mut weight = frame_support::weights::Weight::zero();

		let pallet_prefix: &[u8] = b"Proxy";
		let storage_item_prefix: &[u8] = b"Proxies";

		for (account, (proxies, amount)) in storage_key_iter::<
			T::AccountId,
			(
				BoundedVec<
					ProxyDefinition<T::AccountId, T::ProxyType, T::BlockNumber>,
					T::MaxProxies,
				>,
				BalanceOf<T>,
			),
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.drain()
		{
			let new_amount = amount.saturating_mul(DECIMAL_CONVERTOR.into());
			<Proxies<T>>::insert(account, (proxies, new_amount));

			weight += T::DbWeight::get().reads_writes(1, 1);
		}

		weight
	}

	pub fn replace_proxy_announcements_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplacePalletProxyStorage",
			"Running migration to Proxy - Announcements"
		);

		let mut weight = frame_support::weights::Weight::zero();

		let pallet_prefix: &[u8] = b"Proxy";
		let storage_item_prefix: &[u8] = b"Announcements";

		for (account, (announcements, amount)) in storage_key_iter::<
			T::AccountId,
			(
				BoundedVec<
					Announcement<T::AccountId, CallHashOf<T>, T::BlockNumber>,
					T::MaxPending,
				>,
				BalanceOf<T>,
			),
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.drain()
		{
			let new_amount = amount.saturating_mul(DECIMAL_CONVERTOR.into());
			<Announcements<T>>::insert(account, (announcements, new_amount));

			weight += T::DbWeight::get().reads_writes(1, 1);
		}

		weight
	}
}

#[cfg(feature = "try-runtime")]
impl<T> ReplacePalletProxyStorage<T>
where
	T: pallet_proxy::Config,
{
	// pallet_proxy
	pub fn pre_upgrade_proxy_proxies_storage() -> Result<Vec<u8>, &'static str> {
		let result: BTreeMap<
			T::AccountId,
			(
				BoundedVec<
					ProxyDefinition<T::AccountId, T::ProxyType, T::BlockNumber>,
					T::MaxProxies,
				>,
				BalanceOf<T>,
			),
		> = <Proxies<T>>::iter()
			.map(|(account, (proxies, amount))| {
				let new_amount = amount.saturating_mul(DECIMAL_CONVERTOR.into());
				(account, (proxies, new_amount))
			})
			.collect();
		Ok(result.encode())
	}

	pub fn post_upgrade_proxy_proxies_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_state = BTreeMap::<
			T::AccountId,
			(
				BoundedVec<
					ProxyDefinition<T::AccountId, T::ProxyType, T::BlockNumber>,
					T::MaxProxies,
				>,
				BalanceOf<T>,
			),
		>::decode(&mut &state[..])
		.map_err(|_| "Failed to decode BoundedVec<ProxyDefinition>")?;
		for (account, actual_result) in <Proxies<T>>::iter() {
			let expected_result: (
				BoundedVec<
					ProxyDefinition<T::AccountId, T::ProxyType, T::BlockNumber>,
					T::MaxProxies,
				>,
				BalanceOf<T>,
			) = expected_state
				.get(&account)
				.ok_or("Not Expected BoundedVec<ProxyDefinition>")?
				.clone();
			assert_eq!(expected_result.encode(), actual_result.encode());
		}
		Ok(())
	}

	pub fn pre_upgrade_proxy_announcements_storage() -> Result<Vec<u8>, &'static str> {
		let result: BTreeMap<
			T::AccountId,
			(
				BoundedVec<
					Announcement<T::AccountId, CallHashOf<T>, T::BlockNumber>,
					T::MaxPending,
				>,
				BalanceOf<T>,
			),
		> = <Announcements<T>>::iter()
			.map(|(account, (announcements, amount))| {
				let new_amount = amount.saturating_mul(DECIMAL_CONVERTOR.into());
				(account, (announcements, new_amount))
			})
			.collect();
		Ok(result.encode())
	}

	pub fn post_upgrade_proxy_announcements_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_state = BTreeMap::<
			T::AccountId,
			(
				BoundedVec<
					Announcement<T::AccountId, CallHashOf<T>, T::BlockNumber>,
					T::MaxPending,
				>,
				BalanceOf<T>,
			),
		>::decode(&mut &state[..])
		.map_err(|_| "Failed to decode BoundedVec<Announcement>")?;
		for (account, actual_result) in <Announcements<T>>::iter() {
			let expected_result: (
				BoundedVec<
					Announcement<T::AccountId, CallHashOf<T>, T::BlockNumber>,
					T::MaxPending,
				>,
				BalanceOf<T>,
			) = expected_state
				.get(&account)
				.ok_or("Not Expected BoundedVec<Announcement>")?
				.clone();
			assert_eq!(expected_result.encode(), actual_result.encode());
		}
		Ok(())
	}
}

impl<T> OnRuntimeUpgrade for ReplacePalletProxyStorage<T>
where
	T: pallet_proxy::Config,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		// pallet_proxy
		let proxies_vec = Self::pre_upgrade_proxy_proxies_storage()?;
		let announcements_vec = Self::pre_upgrade_proxy_announcements_storage()?;

		log::info!(
			target: "ReplacePalletProxyStorage",
			"Finished performing Proxy pre upgrade checks"
		);

		Ok((proxies_vec, announcements_vec).encode())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let mut weight = frame_support::weights::Weight::from_parts(0, 0);

		// pallet_proxy
		weight += Self::replace_proxy_proxies_storage();
		weight += Self::replace_proxy_announcements_storage();

		log::info!(
			target: "ReplacePalletProxyStorage",
			"Finished performing Proxy storage migration"
		);

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		let pre_vec: (Vec<u8>, Vec<u8>) =
			Decode::decode(&mut &state[..]).map_err(|_| "Failed to decode Tuple")?;

		// pallet_proxy
		Self::post_upgrade_proxy_proxies_storage(pre_vec.0)?;
		Self::post_upgrade_proxy_announcements_storage(pre_vec.1)?;

		log::info!(
			target: "ReplacePalletProxyStorage",
			"Finished performing Proxy post upgrade checks"
		);

		Ok(())
	}
}
