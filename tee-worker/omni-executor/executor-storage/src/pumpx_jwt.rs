use crate::Storage;
use executor_primitives::AccountId;
use rocksdb::DB;
use std::sync::Arc;

const STORAGE_NAME: &str = "pumpx_jwt_storage";

pub type PumpxJwtStorageKey = (AccountId, &'static str);

pub struct PumpxJwtStorage {
	db: Arc<DB>,
}

impl PumpxJwtStorage {
	pub fn new(db: Arc<DB>) -> Self {
		Self { db }
	}
}

impl Storage<PumpxJwtStorageKey, String> for PumpxJwtStorage {
	fn db(&self) -> Arc<crate::StorageDB> {
		self.db.clone()
	}

	fn name(&self) -> &'static str {
		STORAGE_NAME
	}
}
