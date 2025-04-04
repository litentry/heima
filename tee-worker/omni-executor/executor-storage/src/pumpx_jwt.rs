use crate::{storage_key, Storage};
use executor_primitives::AccountId;
use parity_scale_codec::{Decode, Encode};
use rocksdb::DB;
use std::sync::Arc;

const STORAGE_NAME: &str = "pumpx_jwt_storage";

pub type PumpxJwtStorageKey = (AccountId, &'static str);

pub struct PumpxJwtStorage {
	db: Arc<DB>,
}

impl PumpxJwtStorage {
	pub fn new(db: Arc<DB>) -> Self {
		Self { db }
	}
}

impl Storage<PumpxJwtStorageKey, String> for PumpxJwtStorage {
	fn get(&self, key: &PumpxJwtStorageKey) -> Option<String> {
		match self.db.get(storage_key(STORAGE_NAME, &key.encode())) {
			Ok(Some(value)) => String::decode(&mut &value[..]).ok(),
			_ => {
				log::error!("Error getting pumpx_auth_token from storage");
				None
			},
		}
	}

	fn insert(&self, key: PumpxJwtStorageKey, token: String) -> Result<(), ()> {
		self.db
			.put(storage_key(STORAGE_NAME, &key.encode()), token.encode())
			.map_err(|e| {
				log::error!("Error inserting pumpx_auth_token into storage: {:?}", e);
			})
	}

	fn remove(&self, key: &PumpxJwtStorageKey) -> Result<(), ()> {
		self.db.delete(storage_key(STORAGE_NAME, &key.encode())).map_err(|e| {
			log::error!("Error removing pumpx_auth_token from storage: {:?}", e);
		})
	}

	fn contains_key(&self, key: &PumpxJwtStorageKey) -> bool {
		self.db.key_may_exist(storage_key(STORAGE_NAME, &key.encode()))
	}
}
