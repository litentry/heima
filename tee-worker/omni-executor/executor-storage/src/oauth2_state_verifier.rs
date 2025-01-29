use crate::Storage;
use executor_primitives::Hash;
use parity_scale_codec::{Decode, Encode};
use rocksdb::DB;
use std::sync::Arc;

const STORAGE_NAME: &[u8; 20] = b"oauth2_state_storage";

pub struct OAuth2StateVerifierStorage {
	db: Arc<DB>,
}

impl OAuth2StateVerifierStorage {
	pub fn new(db: Arc<DB>) -> Self {
		Self { db }
	}

	fn storage_key(identity_hash: &Hash) -> Vec<u8> {
		(STORAGE_NAME, identity_hash).encode()
	}
}

impl Storage<Hash, String> for OAuth2StateVerifierStorage {
	fn get(&self, identity_hash: &Hash) -> Option<String> {
		match self.db.get(Self::storage_key(identity_hash)) {
			Ok(Some(value)) => String::decode(&mut &value[..]).ok(),
			_ => {
				log::error!("Error getting oauth2_state from storage");
				None
			},
		}
	}

	fn insert(&self, identity_hash: Hash, state_verifier: String) -> Result<(), ()> {
		self.db
			.put(Self::storage_key(&identity_hash), state_verifier.encode())
			.map_err(|e| {
				log::error!("Error inserting oauth2_state into storage: {:?}", e);
			})
	}

	fn remove(&self, identity_hash: &Hash) -> Result<(), ()> {
		self.db.delete(Self::storage_key(identity_hash)).map_err(|e| {
			log::error!("Error removing oauth2_state from storage: {:?}", e);
		})
	}

	fn contains_key(&self, identity_hash: &Hash) -> bool {
		self.db.key_may_exist(Self::storage_key(identity_hash))
	}
}
