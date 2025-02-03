use crate::{storage_key, Storage};
use executor_primitives::Hash;
use parity_scale_codec::{Decode, Encode};
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
	fn get(&self, identity_hash: &Hash) -> Option<String> {
		match self.db.get(storage_key(STORAGE_NAME, &identity_hash.encode())) {
			Ok(Some(value)) => String::decode(&mut &value[..]).ok(),
			_ => {
				log::error!("Error getting verification_code from storage");
				None
			},
		}
	}

	fn insert(&self, identity_hash: Hash, code: String) -> Result<(), ()> {
		self.db
			.put(storage_key(STORAGE_NAME, &identity_hash.encode()), code.encode())
			.map_err(|e| {
				log::error!("Error inserting verification_code into storage: {:?}", e);
			})
	}

	fn remove(&self, identity_hash: &Hash) -> Result<(), ()> {
		self.db.delete(storage_key(STORAGE_NAME, &identity_hash.encode())).map_err(|e| {
			log::error!("Error removing verification_code from storage: {:?}", e);
		})
	}

	fn contains_key(&self, identity_hash: &Hash) -> bool {
		self.db.key_may_exist(storage_key(STORAGE_NAME, &identity_hash.encode()))
	}
}
