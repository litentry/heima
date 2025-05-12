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

use parity_scale_codec::{Decode, Encode};

use crate::{AmountType, AssetsLock};

// Manages whether any asset is locked for particular account or not.
#[derive(Encode, Decode)]
pub struct AccountWideAssetsLock {
	locked: bool,
}

impl AssetsLock for AccountWideAssetsLock {
	fn with_lock(amount_to_lock: AmountType, available_amount: AmountType) -> Result<Self, ()>
	where
		Self: Sized,
	{
		if amount_to_lock > available_amount {
			return Err(());
		}
		Ok(Self { locked: true })
	}

	fn lock(
		&mut self,
		_amount_to_lock: AmountType,
		_available_amount: AmountType,
	) -> Result<(), ()> {
		if self.locked {
			return Err(());
		}
		self.locked = true;
		Ok(())
	}

	fn release(&mut self, _amount_to_release: AmountType) -> Result<(), ()> {
		if !self.locked {
			return Err(());
		}

		self.locked = false;
		Ok(())
	}

	fn get(&self) -> AmountType {
		AmountType::from(AmountType::MAX)
	}
}

#[cfg(test)]
pub mod tests {
	use crate::AmountType;
	use crate::AssetId;

	use super::AccountWideAssetsLock;
	use crate::AccountAssetLocks;
	use executor_storage::Key;
	use executor_storage::Storage;
	use executor_storage::StorageDB;
	use heima_primitives::AccountId;
	use heima_primitives::EthereumToken;
	use std::sync::Arc;
	use tempfile::tempdir;

	#[test]
	pub fn locks_asset_for_not_tracked_account_if_enough_assets() {
		let tmp_dir = tempdir().unwrap();
		let storage_db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());
		let account_id: AccountId = [1; 32].into();
		let asset_id = AssetId::Ethereum(0, EthereumToken::Native);
		let amount_to_lock = AmountType::from(10);
		let available_amount = AmountType::from(20);
		let account_assets_locks = AccountAssetLocks::<AccountWideAssetsLock>::new(storage_db);
		assert!(account_assets_locks
			.check_and_insert(
				account_id.clone(),
				asset_id.clone(),
				amount_to_lock,
				available_amount
			)
			.is_ok());

		assert_eq!(
			account_assets_locks.get_locked_amount(&account_id, asset_id).unwrap(),
			AmountType::MAX
		);
	}

	#[test]
	pub fn not_locks_asset_for_not_tracked_account_if_not_enough_assets() {
		let tmp_dir = tempdir().unwrap();
		let storage_db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());
		let account_id: AccountId = [1; 32].into();
		let asset_id = AssetId::Ethereum(0, EthereumToken::Native);
		let amount_to_lock = AmountType::from(10);
		let available_amount = AmountType::from(9);
		let account_assets_locks = AccountAssetLocks::<AccountWideAssetsLock>::new(storage_db);
		assert!(account_assets_locks
			.check_and_insert(
				account_id.clone(),
				asset_id.clone(),
				amount_to_lock,
				available_amount
			)
			.is_err());

		assert!(account_assets_locks
			.storage
			.get(&Key { account_id: account_id.clone(), asset_id })
			.unwrap()
			.is_none());
	}

	#[test]
	pub fn not_locks_if_already_locked() {
		let tmp_dir = tempdir().unwrap();
		let storage_db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());
		let account_id: AccountId = [1; 32].into();
		let asset_id = AssetId::Ethereum(0, EthereumToken::Native);
		let amount_to_lock = AmountType::from(10);
		let available_amount = AmountType::from(15);
		let account_assets_locks = AccountAssetLocks::<AccountWideAssetsLock>::new(storage_db);
		let _ = account_assets_locks.check_and_insert(
			account_id.clone(),
			asset_id.clone(),
			amount_to_lock,
			available_amount,
		);

		assert!(account_assets_locks
			.check_and_insert(account_id.clone(), asset_id, amount_to_lock, available_amount)
			.is_err());
	}

	#[test]
	pub fn returns_error_if_called_release_for_not_locked_account() {
		let tmp_dir = tempdir().unwrap();
		let storage_db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());
		let account_id = [1; 32].into();
		let asset_id = AssetId::Ethereum(0, EthereumToken::Native);
		let amount_to_release = AmountType::from(15);
		let account_assets_locks = AccountAssetLocks::<AccountWideAssetsLock>::new(storage_db);

		assert!(account_assets_locks.release(account_id, asset_id, amount_to_release).is_err());
	}

	#[test]
	pub fn releases_assets() {
		let tmp_dir = tempdir().unwrap();
		let storage_db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());
		let account_id: AccountId = [1; 32].into();
		let asset_id = AssetId::Ethereum(0, EthereumToken::Native);
		let amount_to_release = AmountType::from(5);
		let account_assets_locks = AccountAssetLocks::<AccountWideAssetsLock>::new(storage_db);
		let _ = account_assets_locks.check_and_insert(
			account_id.clone(),
			asset_id.clone(),
			amount_to_release,
			amount_to_release,
		);

		assert!(account_assets_locks
			.release(account_id.clone(), asset_id, amount_to_release)
			.is_ok());
	}
}
