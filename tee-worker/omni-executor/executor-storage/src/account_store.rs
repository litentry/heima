use crate::Storage;
use executor_primitives::{AccountId, MemberAccount};
use rocksdb::DB;
use std::{sync::Arc, vec::Vec};

const STORAGE_NAME: &str = "account_store_storage";

pub struct AccountStoreStorage {
	db: Arc<DB>,
}

impl AccountStoreStorage {
	pub fn new(db: Arc<DB>) -> Self {
		Self { db }
	}
}

type AccountStore = Vec<MemberAccount>;

impl Storage<AccountId, AccountStore> for AccountStoreStorage {
	fn db(&self) -> Arc<crate::StorageDB> {
		self.db.clone()
	}

	fn name(&self) -> &'static str {
		STORAGE_NAME
	}
}
