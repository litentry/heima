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

use crate::{AmountType, AssetsLock};

// Manages whether any asset is locked for particular account or not.
pub struct AccountWideAssetsLock {
	locked: bool,
}

impl AssetsLock for AccountWideAssetsLock {
	fn with_lock(
		_asset_id: crate::AssetId,
		amount_to_lock: AmountType,
		available_amount: AmountType,
	) -> Result<Self, ()>
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
		_asset_id: crate::AssetId,
		_amount_to_lock: AmountType,
		_available_amount: AmountType,
	) -> Result<(), ()> {
		if self.locked {
			return Err(());
		}
		self.locked = true;
		Ok(())
	}

	fn release(
		&mut self,
		_asset_id: crate::AssetId,
		_amount_to_release: AmountType,
	) -> Result<(), ()> {
		if !self.locked {
			return Err(());
		}

		self.locked = false;
		Ok(())
	}
}

#[cfg(test)]
pub mod tests {
	use crate::AmountType;
	use crate::AssetId;

	use super::AccountWideAssetsLock;
	use super::AssetsLock;

	use crate::AccountAssetLocks;
	use heima_primitives::AccountId;
	use heima_primitives::EthereumToken;

	#[test]
	pub fn locks_asset_for_not_tracked_account_if_enough_assets() {
		let account_id: AccountId = [1; 32].into();
		let asset_id = AssetId::Ethereum(0, EthereumToken::Native);
		let amount_to_lock = AmountType::from(10);
		let available_amount = AmountType::from(20);
		let account_assets_locks = AccountAssetLocks::<AccountWideAssetsLock>::empty();
		assert!(account_assets_locks
			.check_and_insert(account_id.clone(), asset_id, amount_to_lock, available_amount)
			.is_ok());
		assert!(account_assets_locks.locks.read().unwrap().get(&account_id).unwrap().locked);
	}

	#[test]
	pub fn not_locks_asset_for_not_tracked_account_if_not_enough_assets() {
		let account_id: AccountId = [1; 32].into();
		let asset_id = AssetId::Ethereum(0, EthereumToken::Native);
		let amount_to_lock = AmountType::from(10);
		let available_amount = AmountType::from(9);
		let account_assets_locks = AccountAssetLocks::<AccountWideAssetsLock>::empty();
		assert!(account_assets_locks
			.check_and_insert(account_id.clone(), asset_id, amount_to_lock, available_amount)
			.is_err());
		assert!(account_assets_locks.locks.read().unwrap().get(&account_id).is_none());
	}

	#[test]
	pub fn not_locks_if_already_locked() {
		let account_id: AccountId = [1; 32].into();
		let asset_id = AssetId::Ethereum(0, EthereumToken::Native);
		let amount_to_lock = AmountType::from(10);
		let available_amount = AmountType::from(15);
		let mut account_assets_locks = AccountAssetLocks::<AccountWideAssetsLock>::empty();
		account_assets_locks.locks.get_mut().unwrap().insert(
			account_id.clone(),
			AccountWideAssetsLock::with_lock(
				asset_id.clone(),
				AmountType::from(10),
				AmountType::from(10),
			)
			.unwrap(),
		);

		assert!(account_assets_locks
			.check_and_insert(account_id.clone(), asset_id, amount_to_lock, available_amount)
			.is_err());
		assert!(account_assets_locks.locks.read().unwrap().get(&account_id).unwrap().locked);
	}

	#[test]
	pub fn returns_error_if_called_release_for_not_locked_account() {
		let account_id = [1; 32].into();
		let asset_id = AssetId::Ethereum(0, EthereumToken::Native);
		let amount_to_release = AmountType::from(15);
		let account_assets_locks = AccountAssetLocks::<AccountWideAssetsLock>::empty();

		assert!(account_assets_locks.release(account_id, asset_id, amount_to_release).is_err());
	}

	#[test]
	pub fn releases_assets() {
		let account_id: AccountId = [1; 32].into();
		let asset_id = AssetId::Ethereum(0, EthereumToken::Native);
		let amount_to_release = AmountType::from(5);
		let mut account_assets_locks = AccountAssetLocks::<AccountWideAssetsLock>::empty();
		account_assets_locks.locks.get_mut().unwrap().insert(
			account_id.clone(),
			AccountWideAssetsLock::with_lock(
				asset_id.clone(),
				AmountType::from(10),
				AmountType::from(10),
			)
			.unwrap(),
		);

		assert!(account_assets_locks
			.release(account_id.clone(), asset_id, amount_to_release)
			.is_ok());
		assert!(!account_assets_locks.locks.read().unwrap().get(&account_id).unwrap().locked);
	}
}
