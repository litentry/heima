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

use std::sync::RwLock;

use crate::AssetsLock;

// Manages whether any asset is locked for particular account or not.
pub struct AccountWideAssetsLock {
	locked: RwLock<bool>,
}

impl AssetsLock for AccountWideAssetsLock {
	fn with_lock(
		_asset_id: crate::AssetId,
		amount_to_lock: u128,
		available_amount: u128,
	) -> Result<Self, ()>
	where
		Self: Sized,
	{
		if amount_to_lock > available_amount {
			return Err(());
		}
		Ok(Self { locked: RwLock::new(true) })
	}

	fn lock(
		&mut self,
		_asset_id: crate::AssetId,
		_amount_to_lock: u128,
		_available_amount: u128,
	) -> Result<(), ()> {
		let mut locked = self.locked.write().unwrap();

		if *locked {
			return Err(());
		}
		*locked = true;
		Ok(())
	}

	fn release(&mut self, _asset_id: crate::AssetId, _amount_to_release: u128) -> Result<(), ()> {
		let mut locked = self.locked.write().unwrap();

		if !*locked {
			return Err(());
		}

		*locked = false;
		Ok(())
	}
}

#[cfg(test)]
pub mod tests {
	use super::AssetsLock;
	use super::AccountWideAssetsLock;

	use crate::AccountAssetLocks;

	#[test]
	pub fn locks_asset_for_not_tracked_account_if_enough_assets() {
		let account_id = [1; 32];
		let asset_id = 1_u8;
		let amount_to_lock = 10;
		let available_amount = 20;
		let account_assets_locks = AccountAssetLocks::<AccountWideAssetsLock>::empty();
		assert!(account_assets_locks
			.check_and_insert(account_id, asset_id, amount_to_lock, available_amount)
			.is_ok());
		assert!(*account_assets_locks
			.locks
			.read()
			.unwrap()
			.get(&account_id)
			.unwrap()
			.locked
			.read()
			.unwrap());
	}

	#[test]
	pub fn not_locks_asset_for_not_tracked_account_if_not_enough_assets() {
		let account_id = [1; 32];
		let asset_id = 1_u8;
		let amount_to_lock = 10;
		let available_amount = 9;
		let account_assets_locks = AccountAssetLocks::<AccountWideAssetsLock>::empty();
		assert!(account_assets_locks
			.check_and_insert(account_id, asset_id, amount_to_lock, available_amount)
			.is_err());
		assert!(account_assets_locks.locks.read().unwrap().get(&account_id).is_none());
	}

	#[test]
	pub fn not_locks_if_already_locked() {
		let account_id = [1; 32];
		let asset_id = 1_u8;
		let amount_to_lock = 10;
		let available_amount = 15;
		let mut account_assets_locks = AccountAssetLocks::<AccountWideAssetsLock>::empty();
		account_assets_locks.locks.get_mut().unwrap().insert(
			account_id.clone(),
			AccountWideAssetsLock::with_lock(asset_id.clone(), 10, 10).unwrap(),
		);

		assert!(account_assets_locks
			.check_and_insert(account_id, asset_id, amount_to_lock, available_amount)
			.is_err());
		assert!(*account_assets_locks
			.locks
			.read()
			.unwrap()
			.get(&account_id)
			.unwrap()
			.locked
			.read()
			.unwrap());
	}

	#[test]
	pub fn returns_error_if_called_release_for_not_locked_account() {
		let account_id = [1; 32];
		let asset_id = 1_u8;
		let amount_to_release = 15;
		let account_assets_locks = AccountAssetLocks::<AccountWideAssetsLock>::empty();

		assert!(account_assets_locks.release(account_id, asset_id, amount_to_release).is_err());
	}

	#[test]
	pub fn releases_assets() {
		let account_id = [1; 32];
		let asset_id = 1_u8;
		let amount_to_release: u128 = 5;
		let mut account_assets_locks = AccountAssetLocks::<AccountWideAssetsLock>::empty();
		account_assets_locks.locks.get_mut().unwrap().insert(
			account_id.clone(),
			AccountWideAssetsLock::with_lock(asset_id.clone(), 10, 10).unwrap(),
		);

		assert!(account_assets_locks.release(account_id, asset_id, amount_to_release).is_ok());
		assert!(!*account_assets_locks
			.locks
			.read()
			.unwrap()
			.get(&account_id)
			.unwrap()
			.locked
			.read()
			.unwrap());
	}
}
