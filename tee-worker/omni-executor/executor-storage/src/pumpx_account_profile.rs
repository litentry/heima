use crate::Storage;
use executor_primitives::{AccountId, PumpxAccountProfile};
use rocksdb::DB;
use std::sync::Arc;

const STORAGE_NAME: &str = "pumpx_profile_storage";

pub struct PumpxProfileStorage {
	db: Arc<DB>,
}

impl PumpxProfileStorage {
	pub fn new(db: Arc<DB>) -> Self {
		Self { db }
	}
}

impl Storage<AccountId, PumpxAccountProfile> for PumpxProfileStorage {
	fn db(&self) -> Arc<crate::StorageDB> {
		self.db.clone()
	}

	fn name(&self) -> &'static str {
		STORAGE_NAME
	}
}
