use crate::{storage_key, Storage};
use executor_primitives::{AccountId, Hash};
use parity_scale_codec::{Decode, Encode};
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
	fn get(&self, member_identity: &Hash) -> Option<AccountId> {
		match self.db.get(storage_key(STORAGE_NAME, &member_identity.encode())) {
			Ok(Some(value)) => AccountId::decode(&mut &value[..]).ok(),
			_ => {
				log::error!("Error getting member_account_hash from storage");
				None
			},
		}
	}

	fn insert(&self, member_identity: Hash, omni_account: AccountId) -> Result<(), ()> {
		self.db
			.put(storage_key(STORAGE_NAME, &member_identity.encode()), omni_account.encode())
			.map_err(|e| {
				log::error!("Error inserting member_account_hash into storage: {:?}", e);
			})
	}

	fn remove(&self, member_identity: &Hash) -> Result<(), ()> {
		self.db
			.delete(storage_key(STORAGE_NAME, &member_identity.encode()))
			.map_err(|e| {
				log::error!("Error removing member_account_hash from storage: {:?}", e);
			})
	}

	fn contains_key(&self, member_identity: &Hash) -> bool {
		self.db.key_may_exist(storage_key(STORAGE_NAME, &member_identity.encode()))
	}
}
