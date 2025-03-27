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

use executor_core::intent_executor::AccountId;
use heima_primitives::ChainAsset;
use ruint::Uint;
use std::{collections::HashMap, sync::RwLock};

pub type AssetId = ChainAsset;
pub type AmountType = Uint<256, 4>;

#[allow(dead_code)]
pub mod account_wide;
#[allow(dead_code)]
pub mod precise;

pub struct AccountAssetLocks<AL: AssetsLock> {
	locks: RwLock<HashMap<AccountId, AL>>,
}

impl<AL: AssetsLock> AccountAssetLocks<AL> {
	pub fn empty() -> Self {
		Self { locks: RwLock::new(HashMap::new()) }
	}

	pub fn check_and_insert(
		&self,
		account_id: AccountId,
		asset_id: AssetId,
		amount_to_lock: AmountType,
		available_amount: AmountType,
	) -> Result<(), ()> {
		let mut account_lock = self.locks.write().unwrap();

		if let Some(account) = account_lock.get_mut(&account_id) {
			account.lock(asset_id, amount_to_lock, available_amount)?;
		} else {
			let account = AL::with_lock(asset_id, amount_to_lock, available_amount)?;
			account_lock.insert(account_id, account);
		}

		Ok(())
	}

	pub fn release(
		&self,
		account_id: AccountId,
		asset_id: AssetId,
		amount_to_release: AmountType,
	) -> Result<(), ()> {
		let mut account_lock = self.locks.write().unwrap();
		if let Some(account) = account_lock.get_mut(&account_id) {
			account.release(asset_id, amount_to_release)
		} else {
			Err(())
		}
	}
}

pub trait AssetsLock {
	fn with_lock(
		asset_id: AssetId,
		amount_to_lock: AmountType,
		available_amount: AmountType,
	) -> Result<Self, ()>
	where
		Self: Sized;
	fn lock(
		&mut self,
		asset_id: AssetId,
		amount_to_lock: AmountType,
		available_amount: AmountType,
	) -> Result<(), ()>;

	fn release(&mut self, asset_id: AssetId, amount_to_release: AmountType) -> Result<(), ()>;
}
