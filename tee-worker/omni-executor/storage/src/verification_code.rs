use executor_core::storage::Storage;
use parity_scale_codec::{Decode, Encode};
use primitives::Hash;
use rocksdb::DB;
use std::sync::Arc;

const STORAGE_NAME: &[u8; 25] = b"verification_code_storage";

pub struct VerificationCodeStorage {
	db: Arc<DB>,
}

impl VerificationCodeStorage {
	pub fn new(db: Arc<DB>) -> Self {
		Self { db }
	}

	fn storage_key(identity_hash: &Hash) -> Vec<u8> {
		(STORAGE_NAME, identity_hash).encode()
	}
}

impl Storage<Hash, String> for VerificationCodeStorage {
	fn get(&self, identity_hash: &Hash) -> Option<String> {
		match self.db.get(Self::storage_key(identity_hash)) {
			Ok(Some(value)) => String::decode(&mut &value[..]).ok(),
			_ => {
				log::error!("Error getting verification_code from storage");
				None
			},
		}
	}

	fn insert(&self, identity_hash: Hash, code: String) -> Result<(), ()> {
		self.db.put(Self::storage_key(&identity_hash), code.encode()).map_err(|e| {
			log::error!("Error inserting verification_code into storage: {:?}", e);
		})
	}

	fn remove(&self, identity_hash: &Hash) -> Result<(), ()> {
		self.db.delete(Self::storage_key(identity_hash)).map_err(|e| {
			log::error!("Error removing verification_code from storage: {:?}", e);
		})
	}

	fn contains_key(&self, identity_hash: &Hash) -> bool {
		self.db.key_may_exist(Self::storage_key(identity_hash))
	}
}
