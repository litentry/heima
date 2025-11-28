use crate::Storage;
use oe_primitives::{AccountId, IntentId};
use rocksdb::DB;
use std::sync::Arc;

const STORAGE_NAME: &str = "intent_id_storage";

pub struct IntentIdStorage {
	db: Arc<DB>,
}

impl IntentIdStorage {
	pub fn new(db: Arc<DB>) -> Self {
		Self { db }
	}
}

impl Storage<AccountId, IntentId> for IntentIdStorage {
	fn db(&self) -> Arc<crate::StorageDB> {
		self.db.clone()
	}

	fn name(&self) -> &'static str {
		STORAGE_NAME
	}
}
