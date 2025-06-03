use crate::Storage;
use executor_primitives::AccountId;
use rocksdb::DB;
use std::sync::Arc;

const STORAGE_NAME: &str = "heima_jwt_storage";

pub type HeimaJwtStorageKey = (AccountId, &'static str);

pub struct HeimaJwtStorage {
	db: Arc<DB>,
}

impl HeimaJwtStorage {
	pub fn new(db: Arc<DB>) -> Self {
		Self { db }
	}
}

impl Storage<HeimaJwtStorageKey, String> for HeimaJwtStorage {
	fn db(&self) -> Arc<crate::StorageDB> {
		self.db.clone()
	}

	fn name(&self) -> &'static str {
		STORAGE_NAME
	}
}
