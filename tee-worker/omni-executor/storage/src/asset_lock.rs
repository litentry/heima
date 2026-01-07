use crate::Storage;
use oe_primitives::AccountId;
use oe_primitives::ChainAsset;
use parity_scale_codec::Encode;
use rocksdb::DB;
use std::sync::Arc;

const STORAGE_NAME: &str = "asset_lock_storage";

#[derive(Encode)]
pub struct Key {
	pub account_id: AccountId,
	pub asset_id: ChainAsset,
}

pub struct AssetLockStorage {
	db: Arc<DB>,
}

impl AssetLockStorage {
	pub fn new(db: Arc<DB>) -> Self {
		Self { db }
	}
}

impl Storage<Key, Vec<u8>> for AssetLockStorage {
	fn db(&self) -> Arc<crate::StorageDB> {
		self.db.clone()
	}

	fn name(&self) -> &'static str {
		STORAGE_NAME
	}
}
