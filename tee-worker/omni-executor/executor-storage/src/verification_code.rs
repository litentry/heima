use crate::Storage;
use executor_primitives::Hash;
use rocksdb::DB;
use std::sync::Arc;

const STORAGE_NAME: &str = "verification_code_storage";

pub struct VerificationCodeStorage {
	db: Arc<DB>,
}

impl VerificationCodeStorage {
	pub fn new(db: Arc<DB>) -> Self {
		Self { db }
	}
}

impl Storage<Hash, String> for VerificationCodeStorage {
	fn db(&self) -> Arc<crate::StorageDB> {
		self.db.clone()
	}

	fn name(&self) -> &'static str {
		STORAGE_NAME
	}
}
