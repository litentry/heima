use crate::{storage_key, Storage};
use executor_primitives::AccountId;
use parentchain_api_interface::omni_account::storage::types::account_store::AccountStore;
use parity_scale_codec::{Decode, Encode};
use rocksdb::DB;
use std::sync::Arc;

const STORAGE_NAME: &str = "account_store_storage";

pub struct AccountStoreStorage {
	db: Arc<DB>,
}

impl AccountStoreStorage {
	pub fn new(db: Arc<DB>) -> Self {
		Self { db }
	}
}

impl Storage<AccountId, AccountStore> for AccountStoreStorage {
	fn get(&self, account_id: &AccountId) -> Option<AccountStore> {
		match self.db.get(storage_key(STORAGE_NAME, &account_id.encode())) {
			Ok(Some(value)) => AccountStore::decode(&mut &value[..])
				.map_err(|e| {
					log::error!("Error decoding value from storage: {:?}", e);
				})
				.ok(),
			Ok(None) => None,
			Err(e) => {
				log::error!("Error getting value from storage: {:?}", e);
				None
			},
		}
	}

	fn insert(&self, account_id: AccountId, account_store: AccountStore) -> Result<(), ()> {
		self.db
			.put(storage_key(STORAGE_NAME, &account_id.encode()), account_store.encode())
			.map_err(|e| {
				log::error!("Error inserting value into storage: {:?}", e);
			})
	}

	fn remove(&self, account_id: &AccountId) -> Result<(), ()> {
		self.db.delete(storage_key(STORAGE_NAME, &account_id.encode())).map_err(|e| {
			log::error!("Error removing value from storage: {:?}", e);
		})
	}

	fn contains_key(&self, account_id: &AccountId) -> bool {
		self.db.key_may_exist(storage_key(STORAGE_NAME, &account_id.encode()))
	}
}
