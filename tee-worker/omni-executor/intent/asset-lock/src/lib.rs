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

use executor_primitives::AccountId;
use executor_storage::{AssetLockStorage, AssetLockStorageKey};
use executor_storage::{Storage, StorageDB};
use heima_primitives::ChainAsset;
use parity_scale_codec::Decode;
use parity_scale_codec::Encode;
use ruint::Uint;
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::error;

pub type AssetId = ChainAsset;
pub type AmountType = Uint<256, 4>;

#[allow(dead_code)]
pub mod account_wide;
#[allow(dead_code)]
pub mod always_unlocked;
#[allow(dead_code)]
pub mod precise;

pub struct AccountAssetLocks<AL: AssetsLock> {
	storage: AssetLockStorage,
	phantom: PhantomData<AL>,
}

impl<AL: AssetsLock> AccountAssetLocks<AL> {
	pub fn new(db: Arc<StorageDB>) -> Self {
		Self { storage: AssetLockStorage::new(db), phantom: PhantomData }
	}

	pub fn check_and_insert(
		&self,
		account_id: AccountId,
		asset_id: AssetId,
		amount_to_lock: AmountType,
		available_amount: AmountType,
	) -> Result<(), ()> {
		let key =
			AssetLockStorageKey { account_id: account_id.clone(), asset_id: asset_id.clone() };
		if let Some(account_lock) = self
			.storage
			.get(&key)
			.map_err(|e| error!("Could not get account lock: {:?}", e))?
		{
			let mut account_lock = AL::decode(&mut account_lock.as_slice())
				.map_err(|e| error!("Could not decode account lock: {:?}", e))?;
			account_lock.lock(amount_to_lock, available_amount)?;
			self.storage
				.insert(&key, account_lock.encode())
				.map_err(|e| error!("Could not insert account lock: {:?}", e))?;
			// save
		} else {
			let account_lock = AL::with_lock(amount_to_lock, available_amount)?;
			self.storage
				.insert(&key, account_lock.encode())
				.map_err(|e| error!("Could not insert account lock: {:?}", e))?;
		}
		Ok(())
	}

	pub fn release(
		&self,
		account_id: AccountId,
		asset_id: AssetId,
		amount_to_release: AmountType,
	) -> Result<(), ()> {
		let key =
			AssetLockStorageKey { account_id: account_id.clone(), asset_id: asset_id.clone() };
		if let Some(account_lock) = self
			.storage
			.get(&key)
			.map_err(|e| error!("Could not get account lock: {:?}", e))?
		{
			let mut account_lock = AL::decode(&mut account_lock.as_slice())
				.map_err(|e| error!("Could not decode account lock: {:?}", e))?;
			account_lock
				.release(amount_to_release)
				.map_err(|e| error!("Could not release locked amount: {:?}", e))?;
			self.storage
				.insert(&key, account_lock.encode())
				.map_err(|e| error!("Could not insert account lock: {:?}", e))?;
			Ok(())
		} else {
			Err(())
		}
	}

	pub fn get_locked_amount(
		&self,
		account_id: &AccountId,
		asset_id: AssetId,
	) -> Result<AmountType, ()> {
		let key =
			AssetLockStorageKey { account_id: account_id.clone(), asset_id: asset_id.clone() };
		if let Some(account_lock) = self
			.storage
			.get(&key)
			.map_err(|e| error!("Could not get account lock: {:?}", e))?
		{
			let account_lock = AL::decode(&mut account_lock.as_slice())
				.map_err(|e| error!("Could not decode account lock: {:?}", e))?;
			Ok(account_lock.get())
		} else {
			Ok(AmountType::from(0))
		}
	}
}

pub trait AssetsLock: Encode + Decode {
	fn with_lock(amount_to_lock: AmountType, available_amount: AmountType) -> Result<Self, ()>
	where
		Self: Sized;
	fn lock(&mut self, amount_to_lock: AmountType, available_amount: AmountType) -> Result<(), ()>;

	fn release(&mut self, amount_to_release: AmountType) -> Result<(), ()>;

	fn get(&self) -> AmountType;
}
