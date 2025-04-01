use crate::{storage_key, Storage};
use executor_primitives::AccountId;
use parity_scale_codec::{Decode, Encode};
use rocksdb::DB;
use std::sync::Arc;

const STORAGE_NAME: &str = "pumpx_auth_token_id_storage";

pub struct PumpxAuthTokenIdStorage {
	db: Arc<DB>,
}

impl PumpxAuthTokenIdStorage {
	pub fn new(db: Arc<DB>) -> Self {
		Self { db }
	}
}

impl Storage<AccountId, String> for PumpxAuthTokenIdStorage {
	fn get(&self, omni_account: &AccountId) -> Option<String> {
		match self.db.get(storage_key(STORAGE_NAME, &omni_account.encode())) {
			Ok(Some(value)) => String::decode(&mut &value[..]).ok(),
			_ => {
				log::error!("Error getting pumpx_auth_token from storage");
				None
			},
		}
	}

	fn insert(&self, omni_account: AccountId, token: String) -> Result<(), ()> {
		self.db
			.put(storage_key(STORAGE_NAME, &omni_account.encode()), token.encode())
			.map_err(|e| {
				log::error!("Error inserting pumpx_auth_token into storage: {:?}", e);
			})
	}

	fn remove(&self, omni_account: &AccountId) -> Result<(), ()> {
		self.db.delete(storage_key(STORAGE_NAME, &omni_account.encode())).map_err(|e| {
			log::error!("Error removing pumpx_auth_token from storage: {:?}", e);
		})
	}

	fn contains_key(&self, omni_account: &AccountId) -> bool {
		self.db.key_may_exist(storage_key(STORAGE_NAME, &omni_account.encode()))
	}
}
