use crate::{storage_key, Storage};
use executor_primitives::Hash;
use parity_scale_codec::{Decode, Encode};
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

impl Storage<Hash, String> for OAuth2StateVerifierStorage {
	fn get(&self, identity_hash: &Hash) -> Option<String> {
		match self.db.get(storage_key(STORAGE_NAME, &identity_hash.encode())) {
			Ok(Some(value)) => String::decode(&mut &value[..]).ok(),
			_ => {
				log::error!("Error getting oauth2_state from storage");
				None
			},
		}
	}

	fn insert(&self, identity_hash: Hash, state_verifier: String) -> Result<(), ()> {
		self.db
			.put(storage_key(STORAGE_NAME, &identity_hash.encode()), state_verifier.encode())
			.map_err(|e| {
				log::error!("Error inserting oauth2_state into storage: {:?}", e);
			})
	}

	fn remove(&self, identity_hash: &Hash) -> Result<(), ()> {
		self.db.delete(storage_key(STORAGE_NAME, &identity_hash.encode())).map_err(|e| {
			log::error!("Error removing oauth2_state from storage: {:?}", e);
		})
	}

	fn contains_key(&self, identity_hash: &Hash) -> bool {
		self.db.key_may_exist(storage_key(STORAGE_NAME, &identity_hash.encode()))
	}
}
