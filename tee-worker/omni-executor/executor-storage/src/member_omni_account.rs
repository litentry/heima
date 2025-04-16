use crate::Storage;
use executor_primitives::{AccountId, Hash};
use rocksdb::DB;
use std::sync::Arc;

const STORAGE_NAME: &str = "member_omni_account";

pub struct MemberOmniAccountStorage {
	db: Arc<DB>,
}

impl MemberOmniAccountStorage {
	pub fn new(db: Arc<DB>) -> Self {
		Self { db }
	}
}

impl Storage<Hash, AccountId> for MemberOmniAccountStorage {
	fn db(&self) -> Arc<crate::StorageDB> {
		self.db.clone()
	}

	fn name(&self) -> &'static str {
		STORAGE_NAME
	}
}
