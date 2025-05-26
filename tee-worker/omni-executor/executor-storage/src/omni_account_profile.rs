use crate::Storage;
use executor_primitives::{AccountId, OmniAccountProfile};
use rocksdb::DB;
use std::sync::Arc;

const STORAGE_NAME: &str = "omni_account_profile_storage";

pub struct OmniAccountProfileStorage {
	db: Arc<DB>,
}

impl OmniAccountProfileStorage {
	pub fn new(db: Arc<DB>) -> Self {
		Self { db }
	}
}

impl Storage<AccountId, OmniAccountProfile> for OmniAccountProfileStorage {
	fn db(&self) -> Arc<crate::StorageDB> {
		self.db.clone()
	}

	fn name(&self) -> &'static str {
		STORAGE_NAME
	}
}
