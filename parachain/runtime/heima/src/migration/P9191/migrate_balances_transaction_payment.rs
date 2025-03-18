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
	traits::{Get, OnRuntimeUpgrade},
	Blake2_128Concat, WeakBoundedVec,
};
use frame_system::{Account, AccountInfo};
use pallet_balances::{
	Account as BAccount, AccountData, BalanceLock, Freezes, Holds, InactiveIssuance, Locks,
	Reserves, TotalIssuance,
};
use sp_std::{marker::PhantomData, vec::Vec};

pub const DECIMAL_CONVERTOR: u128 = 1_000_000u128;

#[cfg(feature = "try-runtime")]
use parity_scale_codec::Encode;
#[cfg(feature = "try-runtime")]
use sp_std::collections::btree_map::BTreeMap;
use storage::migration::get_storage_value;

// Replace Frame System Storage for Decimal Change from 12 to 18
// Replace Balances Storage for Decimal Change from 12 to 18
pub struct ReplaceBalancesRelatedStorage<T>(PhantomData<T>);
impl<T> ReplaceBalancesRelatedStorage<T>
where
	T: frame_system::Config<AccountData = AccountData<u128>>
		+ pallet_balances::Config<Balance = u128>,
{
	pub fn replace_frame_system_account_storage() -> frame_support::weights::Weight {
		// The storage of Account for pallet balances is in frame_system pallet
		log::info!(
			target: "ReplaceBalancesRelatedStorage",
			"running migration to Frame System Account"
		);
		let pallet_prefix: &[u8] = b"System";
		let storage_item_prefix: &[u8] = b"Account";
		let mut weight: Weight = frame_support::weights::Weight::zero();

		for (account, mut account_info) in storage_key_iter::<
			T::AccountId,
			AccountInfo<T::Index, T::AccountData>,
			Blake2_128Concat,
		>(pallet_prefix, storage_item_prefix)
		.drain()
		{
			account_info.data.free = account_info.data.free.saturating_mul(DECIMAL_CONVERTOR);
			account_info.data.reserved =
				account_info.data.reserved.saturating_mul(DECIMAL_CONVERTOR);
			account_info.data.frozen = account_info.data.frozen.saturating_mul(DECIMAL_CONVERTOR);

			<Account<T>>::insert(&account, account_info);
			weight += T::DbWeight::get().reads_writes(1, 1);
		}
		weight
	}
	pub fn repalce_balances_total_issuance_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceBalancesRelatedStorage",
			"running migration to Balances TotalIssuance"
		);
		let pallet_prefix: &[u8] = b"Balances";
		let storage_item_prefix: &[u8] = b"TotalIssuance";
		let stored_data = get_storage_value::<u128>(pallet_prefix, storage_item_prefix, b"")
			.expect("Storage query fails: Balances TotalIssuance");
		<TotalIssuance<T>>::put(stored_data.saturating_mul(DECIMAL_CONVERTOR));
		let weight = T::DbWeight::get();
		frame_support::weights::Weight::from_parts(0, weight.write + weight.read)
	}
	pub fn repalce_balances_inactive_issuance_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceBalancesRelatedStorage",
			"running migration to Balances InactiveIssuance"
		);
		let pallet_prefix: &[u8] = b"Balances";
		let storage_item_prefix: &[u8] = b"InactiveIssuance";
		let stored_data = get_storage_value::<u128>(pallet_prefix, storage_item_prefix, b"")
			.expect("Storage query fails: Balances InactiveIssuance");
		<InactiveIssuance<T>>::put(stored_data.saturating_mul(DECIMAL_CONVERTOR));
		let weight = T::DbWeight::get();
		frame_support::weights::Weight::from_parts(0, weight.write + weight.read)
	}
	pub fn replace_balances_locks_storage() -> frame_support::weights::Weight {
		log::info!(
			target: "ReplaceBalancesRelatedStorage",
			"running checking to Balances Locks"
		);
		let pallet_prefix: &[u8] = b"Balances";
		let storage_item_prefix: &[u8] = b"Locks";
		let mut weight: Weight = frame_support::weights::Weight::zero();

		for (account, locks) in storage_key_iter::<
			T::AccountId,
			WeakBoundedVec<BalanceLock<T::Balance>, T::MaxLocks>,
			Blake2_128Concat,
		>(pallet_prefix, storage_item_prefix)
		.drain()
		{
			let mut locks_vec = locks.into_inner();
			for balance_lock in locks_vec.iter_mut() {
				balance_lock.amount = balance_lock.amount.saturating_mul(DECIMAL_CONVERTOR);
			}
			let updated_locks =
				WeakBoundedVec::<BalanceLock<T::Balance>, T::MaxLocks>::force_from(locks_vec, None);
			Locks::<T>::insert(&account, updated_locks);
			weight += T::DbWeight::get().reads_writes(1, 1);
		}

		weight
	}
}

#[cfg(feature = "try-runtime")]
impl<T> ReplaceBalancesRelatedStorage<T>
where
	T: frame_system::Config<AccountData = AccountData<u128>>
		+ pallet_balances::Config<Balance = u128>,
{
	pub fn pre_upgrade_frame_system_account_storage() -> Result<Vec<u8>, &'static str> {
		let result: BTreeMap<T::AccountId, AccountInfo<T::Index, AccountData<u128>>> =
			<Account<T>>::iter()
				.map(|(account, state)| {
					let mut new_account: AccountInfo<T::Index, AccountData<u128>> = state;
					new_account.data.free = new_account.data.free.saturating_mul(DECIMAL_CONVERTOR);
					new_account.data.reserved =
						new_account.data.reserved.saturating_mul(DECIMAL_CONVERTOR);
					new_account.data.frozen =
						new_account.data.frozen.saturating_mul(DECIMAL_CONVERTOR);

					(account, new_account)
				})
				.collect();
		Ok(result.encode())
	}
	pub fn post_upgrade_frame_system_account_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_state =
			BTreeMap::<T::AccountId, AccountInfo<T::Index, AccountData<u128>>>::decode(
				&mut &state[..],
			)
			.map_err(|_| "Failed to decode AccountInfo")?;
		for (account, actual_result) in <Account<T>>::iter() {
			let expected_result: AccountInfo<T::Index, AccountData<u128>> =
				expected_state.get(&account).ok_or("Not Expected AccountInfo")?.clone();
			assert_eq!(expected_result, actual_result);
		}
		Ok(())
	}
	pub fn pre_upgrade_balances_total_issuance_storage() -> Result<Vec<u8>, &'static str> {
		Ok(<TotalIssuance<T>>::get().saturating_mul(DECIMAL_CONVERTOR).encode())
	}
	pub fn post_upgrade_balances_total_issuance_storage(
		state: Vec<u8>,
	) -> Result<(), &'static str> {
		let expected_state =
			u128::decode(&mut &state[..]).map_err(|_| "Failed to decode Total Balance")?;
		let actual_state = <TotalIssuance<T>>::get();
		assert_eq!(expected_state, actual_state);
		Ok(())
	}
	pub fn pre_upgrade_balances_inactive_issuance_storage() -> Result<Vec<u8>, &'static str> {
		Ok(<InactiveIssuance<T>>::get().saturating_mul(DECIMAL_CONVERTOR).encode())
	}
	pub fn post_upgrade_balances_inactive_issuance_storage(
		state: Vec<u8>,
	) -> Result<(), &'static str> {
		let expected_state =
			u128::decode(&mut &state[..]).map_err(|_| "Failed to decode Total Balance")?;
		let actual_state = <InactiveIssuance<T>>::get();
		assert_eq!(expected_state, actual_state);
		Ok(())
	}
	pub fn pre_upgrade_balances_account_check() -> Result<Vec<u8>, &'static str> {
		assert!(<BAccount<T>>::iter().next().is_none());
		Ok(Vec::<u8>::new())
	}
	pub fn post_upgrade_balances_account_check(_state: Vec<u8>) -> Result<(), &'static str> {
		assert!(<BAccount<T>>::iter().next().is_none());
		Ok(())
	}
	pub fn pre_upgrade_balances_locks_storage() -> Result<Vec<u8>, &'static str> {
		let result: BTreeMap<T::AccountId, Vec<BalanceLock<u128>>> = <Locks<T>>::iter()
			.map(|(account, state)| {
				let mut new_locks: Vec<BalanceLock<u128>> = state.into_inner();
				for balance_lock in new_locks.iter_mut() {
					balance_lock.amount = balance_lock.amount.saturating_mul(DECIMAL_CONVERTOR);
				}
				(account, new_locks)
			})
			.collect();
		Ok(result.encode())
	}
	pub fn post_upgrade_balances_locks_storage(state: Vec<u8>) -> Result<(), &'static str> {
		let expected_state =
			BTreeMap::<T::AccountId, Vec<BalanceLock<u128>>>::decode(&mut &state[..])
				.map_err(|_| "Failed to decode Locks")?;
		for (account, actual_result) in <Locks<T>>::iter() {
			let expected_result: Vec<BalanceLock<u128>> =
				expected_state.get(&account).ok_or("Not Expected Locks")?.clone();
			assert_eq!(expected_result.encode(), actual_result.into_inner().encode());
		}
		Ok(())
	}
	pub fn pre_upgrade_balances_reserves_check() -> Result<Vec<u8>, &'static str> {
		assert!(<Reserves<T>>::iter().next().is_none());
		Ok(Vec::<u8>::new())
	}
	pub fn post_upgrade_balances_reserves_check(_state: Vec<u8>) -> Result<(), &'static str> {
		assert!(<Reserves<T>>::iter().next().is_none());
		Ok(())
	}
	pub fn pre_upgrade_balances_holds_check() -> Result<Vec<u8>, &'static str> {
		assert!(<Holds<T>>::iter().next().is_none());
		Ok(Vec::<u8>::new())
	}
	pub fn post_upgrade_balances_holds_check(_state: Vec<u8>) -> Result<(), &'static str> {
		assert!(<Holds<T>>::iter().next().is_none());
		Ok(())
	}
	pub fn pre_upgrade_balances_freezes_check() -> Result<Vec<u8>, &'static str> {
		assert!(<Freezes<T>>::iter().next().is_none());
		Ok(Vec::<u8>::new())
	}
	pub fn post_upgrade_balances_freezes_check(_state: Vec<u8>) -> Result<(), &'static str> {
		assert!(<Freezes<T>>::iter().next().is_none());
		Ok(())
	}
}

impl<T> OnRuntimeUpgrade for ReplaceBalancesRelatedStorage<T>
where
	T: frame_system::Config<AccountData = AccountData<u128>>
		+ pallet_balances::Config<Balance = u128>,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		let frame_system_account_vec = Self::pre_upgrade_frame_system_account_storage()?;
		let balances_total_issuance_vec = Self::pre_upgrade_balances_total_issuance_storage()?;
		let balances_inactive_issuance_vec =
			Self::pre_upgrade_balances_inactive_issuance_storage()?;

		let _ = Self::pre_upgrade_balances_account_check()?;

		let balances_locks_vec = Self::pre_upgrade_balances_locks_storage()?;

		let _ = Self::pre_upgrade_balances_reserves_check()?;
		let _ = Self::pre_upgrade_balances_holds_check()?;
		let _ = Self::pre_upgrade_balances_freezes_check()?;

		Ok((
			frame_system_account_vec,
			balances_total_issuance_vec,
			balances_inactive_issuance_vec,
			balances_locks_vec,
		)
			.encode())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let mut weight = frame_support::weights::Weight::from_parts(0, 0);
		// The storage of pallet balances is in frame_system pallet
		weight += Self::replace_frame_system_account_storage();
		weight += Self::repalce_balances_total_issuance_storage();
		weight += Self::repalce_balances_inactive_issuance_storage();

		// The storage of Account for pallet balances is in frame_system pallet
		// Should be empty

		weight += Self::replace_balances_locks_storage();

		// The storage of Reserves/Holds/Freezes for pallet balances is in frame_system pallet
		// Should be empty

		weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		let pre_vec: (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) =
			Decode::decode(&mut &state[..]).map_err(|_| "Failed to decode Tuple")?;
		Self::post_upgrade_frame_system_account_storage(pre_vec.0)?;
		Self::post_upgrade_balances_total_issuance_storage(pre_vec.1)?;
		Self::post_upgrade_balances_inactive_issuance_storage(pre_vec.2)?;

		Self::post_upgrade_balances_account_check(Vec::<u8>::new())?;

		Self::post_upgrade_balances_locks_storage(pre_vec.3)?;

		Self::post_upgrade_balances_reserves_check(Vec::<u8>::new())?;
		Self::post_upgrade_balances_holds_check(Vec::<u8>::new())?;
		Self::post_upgrade_balances_freezes_check(Vec::<u8>::new())?;
		Ok(())
	}
}
