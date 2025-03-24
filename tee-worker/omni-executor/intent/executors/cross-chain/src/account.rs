use executor_core::intent_executor::AccountId;
use std::{collections::HashMap, sync::RwLock};

pub type AssetId = u8;

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
		amount_to_lock: u128,
		available_amount: u128,
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
		amount_to_release: u128,
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
		amount_to_lock: u128,
		available_amount: u128,
	) -> Result<Self, ()>
	where
		Self: Sized;
	fn lock(
		&mut self,
		asset_id: AssetId,
		amount_to_lock: u128,
		available_amount: u128,
	) -> Result<(), ()>;

	fn release(&mut self, asset_id: AssetId, amount_to_release: u128) -> Result<(), ()>;
}

pub struct PreciseAssetsLock {
	locked_assets: HashMap<AssetId, u128>,
}

impl AssetsLock for PreciseAssetsLock {
	fn with_lock(
		asset_id: AssetId,
		amount_to_lock: u128,
		available_amount: u128,
	) -> Result<Self, ()> {
		if amount_to_lock > available_amount {
			return Err(());
		}

		let mut locked_assets = HashMap::new();
		locked_assets.insert(asset_id, amount_to_lock);

		Ok(Self { locked_assets })
	}

	fn lock(
		&mut self,
		asset_id: AssetId,
		amount_to_lock: u128,
		available_amount: u128,
	) -> Result<(), ()> {
		if let Some(lock) = self.locked_assets.get_mut(&asset_id) {
			if *lock + amount_to_lock > available_amount {
				return Err(());
			}
			*lock += amount_to_lock;
		} else {
			if amount_to_lock > available_amount {
				return Err(());
			}
			self.locked_assets.insert(asset_id, amount_to_lock);
		}
		Ok(())
	}

	fn release(&mut self, asset_id: AssetId, amount_to_release: u128) -> Result<(), ()> {
		if let Some(lock) = self.locked_assets.get_mut(&asset_id) {
			if *lock < amount_to_release {
				// this is some kind of inconsistency - we should either set lock to 0 and return Ok or return Err
				return Err(());
			}
			*lock -= amount_to_release;
		} else {
			// this also indicates some kind of inconsistency - why we are trying to release non-existing lock ?
			return Err(());
		}
		Ok(())
	}
}

#[cfg(test)]
pub mod tests {
	use crate::account::AssetsLock;
	use crate::account::PreciseAssetsLock;

	use super::AccountAssetLocks;

	#[test]
	pub fn locks_asset_for_not_tracked_account_if_enough_assets() {
		let account_id = [1; 32];
		let asset_id = 1_u8;
		let amount_to_lock = 10;
		let available_amount = 20;
		let account_assets_locks = AccountAssetLocks::<PreciseAssetsLock>::empty();
		assert!(account_assets_locks
			.check_and_insert(account_id, asset_id, amount_to_lock, available_amount)
			.is_ok());
		assert_eq!(
			*account_assets_locks
				.locks
				.read()
				.unwrap()
				.get(&account_id)
				.unwrap()
				.locked_assets
				.get(&asset_id)
				.unwrap(),
			10
		);
	}

	#[test]
	pub fn not_locks_asset_for_not_tracked_account_if_not_enough_assets() {
		let account_id = [1; 32];
		let asset_id = 1_u8;
		let amount_to_lock = 10;
		let available_amount = 9;
		let account_assets_locks = AccountAssetLocks::<PreciseAssetsLock>::empty();
		assert!(account_assets_locks
			.check_and_insert(account_id, asset_id, amount_to_lock, available_amount)
			.is_err());
		assert!(account_assets_locks.locks.read().unwrap().get(&account_id).is_none());
	}

	#[test]
	pub fn locks_asset_for_tracked_account_if_enough_assets() {
		let account_id = [1; 32];
		let asset_id = 1_u8;
		let amount_to_lock = 10;
		let available_amount = 20;
		let mut account_assets_locks = AccountAssetLocks::<PreciseAssetsLock>::empty();
		account_assets_locks.locks.get_mut().unwrap().insert(
			account_id.clone(),
			PreciseAssetsLock::with_lock(asset_id.clone(), 10, 10).unwrap(),
		);

		assert!(account_assets_locks
			.check_and_insert(account_id, asset_id, amount_to_lock, available_amount)
			.is_ok());
		assert_eq!(
			*account_assets_locks
				.locks
				.read()
				.unwrap()
				.get(&account_id)
				.unwrap()
				.locked_assets
				.get(&asset_id)
				.unwrap(),
			20
		);
	}

	#[test]
	pub fn not_locks_asset_for_tracked_account_if_not_enough_assets() {
		let account_id = [1; 32];
		let asset_id = 1_u8;
		let amount_to_lock = 10;
		let available_amount = 15;
		let mut account_assets_locks = AccountAssetLocks::<PreciseAssetsLock>::empty();
		account_assets_locks.locks.get_mut().unwrap().insert(
			account_id.clone(),
			PreciseAssetsLock::with_lock(asset_id.clone(), 10, 10).unwrap(),
		);

		assert!(account_assets_locks
			.check_and_insert(account_id, asset_id, amount_to_lock, available_amount)
			.is_err());
		assert_eq!(
			*account_assets_locks
				.locks
				.read()
				.unwrap()
				.get(&account_id)
				.unwrap()
				.locked_assets
				.get(&asset_id)
				.unwrap(),
			10
		);
	}

	#[test]
	pub fn returns_error_if_locked_amount_is_lower_that_requested_release() {
		let account_id = [1; 32];
		let asset_id = 1_u8;
		let amount_to_release = 15;
		let mut account_assets_locks = AccountAssetLocks::<PreciseAssetsLock>::empty();
		account_assets_locks.locks.get_mut().unwrap().insert(
			account_id.clone(),
			PreciseAssetsLock::with_lock(asset_id.clone(), 10, 10).unwrap(),
		);

		assert!(account_assets_locks.release(account_id, asset_id, amount_to_release).is_err());
	}

	#[test]
	pub fn returns_error_if_called_release_for_not_tracked_account() {
		let account_id = [1; 32];
		let asset_id = 1_u8;
		let amount_to_release = 15;
		let account_assets_locks = AccountAssetLocks::<PreciseAssetsLock>::empty();

		assert!(account_assets_locks.release(account_id, asset_id, amount_to_release).is_err());
	}

	#[test]
	pub fn returns_error_if_called_release_for_not_tracked_asset_for_account() {
		let account_id = [1; 32];
		let asset_id = 2_u8;
		let amount_to_release = 15;
		let mut account_assets_locks = AccountAssetLocks::<PreciseAssetsLock>::empty();
		account_assets_locks.locks.get_mut().unwrap().insert(
			account_id.clone(),
			PreciseAssetsLock::with_lock(asset_id.clone(), 10, 10).unwrap(),
		);

		assert!(account_assets_locks.release(account_id, asset_id, amount_to_release).is_err());
	}
}
