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

// Manages asset locks per account. Used for precise tracking of currently locked assets.
#[derive(Encode, Decode)]
pub struct PreciseAssetsLock {
	amount: AmountType,
}

impl AssetsLock for PreciseAssetsLock {
	fn with_lock(amount_to_lock: AmountType, available_amount: AmountType) -> Result<Self, ()> {
		if amount_to_lock > available_amount {
			return Err(());
		}
		Ok(Self { amount: amount_to_lock })
	}

	fn lock(&mut self, amount_to_lock: AmountType, available_amount: AmountType) -> Result<(), ()> {
		if self.amount + amount_to_lock > available_amount {
			return Err(());
		}
		self.amount += amount_to_lock;
		Ok(())
	}

	fn release(&mut self, amount_to_release: AmountType) -> Result<(), ()> {
		if self.amount < amount_to_release {
			// this is some kind of inconsistency - we should either set lock to 0 and return Ok or return Err
			return Err(());
		}
		self.amount -= amount_to_release;
		Ok(())
	}

	fn get(&self) -> AmountType {
		self.amount.to_owned()
	}
}

// #[cfg(test)]
// pub mod tests {
// 	use super::AssetsLock;
// 	use super::PreciseAssetsLock;
// 	use crate::precise::AmountType;
// 	use crate::AssetId;

// 	use crate::AccountAssetLocks;
// 	use heima_primitives::AccountId;
// 	use heima_primitives::EthereumToken;

// 	#[test]
// 	pub fn locks_asset_for_not_tracked_account_if_enough_assets() {
// 		let account_id: AccountId = [1; 32].into();
// 		let asset_id = AssetId::Ethereum(0, EthereumToken::Native);
// 		let amount_to_lock = AmountType::from(10);
// 		let available_amount = AmountType::from(20);
// 		let account_assets_locks = AccountAssetLocks::<PreciseAssetsLock>::new();
// 		assert!(account_assets_locks
// 			.check_and_insert(
// 				account_id.clone(),
// 				asset_id.clone(),
// 				amount_to_lock,
// 				available_amount
// 			)
// 			.is_ok());
// 		assert_eq!(
// 			*account_assets_locks
// 				.locks
// 				.read()
// 				.unwrap()
// 				.get(&account_id)
// 				.unwrap()
// 				.locked_assets
// 				.get(&asset_id)
// 				.unwrap(),
// 			AmountType::from(10)
// 		);
// 	}

// 	#[test]
// 	pub fn not_locks_asset_for_not_tracked_account_if_not_enough_assets() {
// 		let account_id: AccountId = [1; 32].into();
// 		let asset_id = AssetId::Ethereum(0, EthereumToken::Native);
// 		let amount_to_lock = AmountType::from(10);
// 		let available_amount = AmountType::from(9);
// 		let account_assets_locks = AccountAssetLocks::<PreciseAssetsLock>::new();
// 		assert!(account_assets_locks
// 			.check_and_insert(account_id.clone(), asset_id, amount_to_lock, available_amount)
// 			.is_err());
// 		assert!(account_assets_locks.locks.read().unwrap().get(&account_id).is_none());
// 	}

// 	#[test]
// 	pub fn locks_asset_for_tracked_account_if_enough_assets() {
// 		let account_id: AccountId = [1; 32].into();
// 		let asset_id = AssetId::Ethereum(0, EthereumToken::Native);
// 		let amount_to_lock = AmountType::from(10);
// 		let available_amount = AmountType::from(20);
// 		let mut account_assets_locks = AccountAssetLocks::<PreciseAssetsLock>::new();
// 		account_assets_locks.locks.get_mut().unwrap().insert(
// 			account_id.clone(),
// 			PreciseAssetsLock::with_lock(
// 				asset_id.clone(),
// 				AmountType::from(10),
// 				AmountType::from(10),
// 			)
// 			.unwrap(),
// 		);

// 		assert!(account_assets_locks
// 			.check_and_insert(
// 				account_id.clone(),
// 				asset_id.clone(),
// 				amount_to_lock,
// 				available_amount
// 			)
// 			.is_ok());
// 		assert_eq!(
// 			*account_assets_locks
// 				.locks
// 				.read()
// 				.unwrap()
// 				.get(&account_id)
// 				.unwrap()
// 				.locked_assets
// 				.get(&asset_id)
// 				.unwrap(),
// 			AmountType::from(20)
// 		);
// 	}

// 	#[test]
// 	pub fn not_locks_asset_for_tracked_account_if_not_enough_assets() {
// 		let account_id: AccountId = [1; 32].into();
// 		let asset_id = AssetId::Ethereum(0, EthereumToken::Native);
// 		let amount_to_lock = AmountType::from(10);
// 		let available_amount = AmountType::from(15);
// 		let mut account_assets_locks = AccountAssetLocks::<PreciseAssetsLock>::new();
// 		account_assets_locks.locks.get_mut().unwrap().insert(
// 			account_id.clone(),
// 			PreciseAssetsLock::with_lock(
// 				asset_id.clone(),
// 				AmountType::from(10),
// 				AmountType::from(10),
// 			)
// 			.unwrap(),
// 		);

// 		assert!(account_assets_locks
// 			.check_and_insert(
// 				account_id.clone(),
// 				asset_id.clone(),
// 				amount_to_lock,
// 				available_amount
// 			)
// 			.is_err());
// 		assert_eq!(
// 			*account_assets_locks
// 				.locks
// 				.read()
// 				.unwrap()
// 				.get(&account_id)
// 				.unwrap()
// 				.locked_assets
// 				.get(&asset_id)
// 				.unwrap(),
// 			AmountType::from(10)
// 		);
// 	}

// 	#[test]
// 	pub fn returns_error_if_locked_amount_is_lower_that_requested_release() {
// 		let account_id: AccountId = [1; 32].into();
// 		let asset_id = AssetId::Ethereum(0, EthereumToken::Native);
// 		let amount_to_release = AmountType::from(15);
// 		let mut account_assets_locks = AccountAssetLocks::<PreciseAssetsLock>::new();
// 		account_assets_locks.locks.get_mut().unwrap().insert(
// 			account_id.clone(),
// 			PreciseAssetsLock::with_lock(
// 				asset_id.clone(),
// 				AmountType::from(10),
// 				AmountType::from(10),
// 			)
// 			.unwrap(),
// 		);

// 		assert!(account_assets_locks.release(account_id, asset_id, amount_to_release).is_err());
// 	}

// 	#[test]
// 	pub fn returns_error_if_called_release_for_not_tracked_account() {
// 		let account_id: AccountId = [1; 32].into();
// 		let asset_id = AssetId::Ethereum(0, EthereumToken::Native);
// 		let amount_to_release = AmountType::from(15);
// 		let account_assets_locks = AccountAssetLocks::<PreciseAssetsLock>::new();

// 		assert!(account_assets_locks.release(account_id, asset_id, amount_to_release).is_err());
// 	}

// 	#[test]
// 	pub fn returns_error_if_called_release_for_not_tracked_asset_for_account() {
// 		let account_id: AccountId = [1; 32].into();
// 		let asset_id = AssetId::Ethereum(0, EthereumToken::Native);
// 		let amount_to_release = AmountType::from(15);
// 		let mut account_assets_locks = AccountAssetLocks::<PreciseAssetsLock>::new();
// 		account_assets_locks.locks.get_mut().unwrap().insert(
// 			account_id.clone(),
// 			PreciseAssetsLock::with_lock(
// 				asset_id.clone(),
// 				AmountType::from(10),
// 				AmountType::from(10),
// 			)
// 			.unwrap(),
// 		);

// 		assert!(account_assets_locks.release(account_id, asset_id, amount_to_release).is_err());
// 	}

// 	#[test]
// 	pub fn releases_assets() {
// 		let account_id: AccountId = [1; 32].into();
// 		let asset_id = AssetId::Ethereum(0, EthereumToken::Native);
// 		let amount_to_release = AmountType::from(5);
// 		let mut account_assets_locks = AccountAssetLocks::<PreciseAssetsLock>::new();
// 		account_assets_locks.locks.get_mut().unwrap().insert(
// 			account_id.clone(),
// 			PreciseAssetsLock::with_lock(
// 				asset_id.clone(),
// 				AmountType::from(10),
// 				AmountType::from(10),
// 			)
// 			.unwrap(),
// 		);

// 		assert!(account_assets_locks
// 			.release(account_id.clone(), asset_id.clone(), amount_to_release)
// 			.is_ok());
// 		assert_eq!(
// 			*account_assets_locks
// 				.locks
// 				.read()
// 				.unwrap()
// 				.get(&account_id)
// 				.unwrap()
// 				.locked_assets
// 				.get(&asset_id)
// 				.unwrap(),
// 			AmountType::from(5)
// 		);
// 	}
// }
