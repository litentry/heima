use crate::Storage;
use oe_primitives::{Hash, OAuth2VerificationData};
use rocksdb::DB;
use std::sync::Arc;

const STORAGE_NAME: &str = "oauth2_state_storage";

pub struct OAuth2StateVerifierStorage {
	db: Arc<DB>,
}

impl OAuth2StateVerifierStorage {
	pub fn new(db: Arc<DB>) -> Self {
		Self { db }
	}
}

impl Storage<Hash, OAuth2VerificationData> for OAuth2StateVerifierStorage {
	fn db(&self) -> Arc<crate::StorageDB> {
		self.db.clone()
	}

	fn name(&self) -> &'static str {
		STORAGE_NAME
	}
}
