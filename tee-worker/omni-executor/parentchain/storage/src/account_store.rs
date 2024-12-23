use executor_core::storage::Storage;
use parentchain_api_interface::omni_account::storage::types::account_store::AccountStore;
use parity_scale_codec::{Decode, Encode};
use rocksdb::DB;
use std::path::Path;
use subxt_core::utils::AccountId32 as AccountId;

const STORAGE_NAME: &str = "account_store_storage";

pub struct AccountStoreStorage {
	db: DB,
}

impl AccountStoreStorage {
	pub fn new() -> Self {
		Self::default()
	}
}

impl Default for AccountStoreStorage {
	fn default() -> Self {
		let path = Path::new(crate::STORAGE_PATH).join(STORAGE_NAME);
		let db = DB::open_default(path).expect("Failed to open database");
		Self { db }
	}
}

impl Storage<AccountId, AccountStore> for AccountStoreStorage {
	fn get(&self, account_id: &AccountId) -> Option<AccountStore> {
		match self.db.get(account_id) {
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
		self.db.put(account_id, account_store.encode()).map_err(|e| {
			log::error!("Error inserting value into storage: {:?}", e);
		})
	}

	fn remove(&self, account_id: &AccountId) -> Result<(), ()> {
		self.db.delete(account_id).map_err(|e| {
			log::error!("Error removing value from storage: {:?}", e);
		})
	}

	fn contains_key(&self, account_id: &AccountId) -> bool {
		self.db.key_may_exist(account_id)
	}
}
