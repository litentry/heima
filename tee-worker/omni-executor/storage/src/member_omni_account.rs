use crate::Storage;
use parity_scale_codec::{Decode, Encode};
use primitives::{AccountId, Hash};
use rocksdb::DB;
use std::sync::Arc;

const STORAGE_NAME: &[u8; 19] = b"member_omni_account";

pub struct MemberOmniAccountStorage {
	db: Arc<DB>,
}

impl MemberOmniAccountStorage {
	pub fn new(db: Arc<DB>) -> Self {
		Self { db }
	}

	fn storage_key(member_identity: &Hash) -> Vec<u8> {
		(STORAGE_NAME, member_identity).encode()
	}
}

impl Storage<Hash, AccountId> for MemberOmniAccountStorage {
	fn get(&self, member_identity: &Hash) -> Option<AccountId> {
		match self.db.get(Self::storage_key(member_identity)) {
			Ok(Some(value)) => AccountId::decode(&mut &value[..]).ok(),
			_ => {
				log::error!("Error getting member_account_hash from storage");
				None
			},
		}
	}

	fn insert(&self, member_identity: Hash, omni_account: AccountId) -> Result<(), ()> {
		self.db
			.put(Self::storage_key(&member_identity), omni_account.encode())
			.map_err(|e| {
				log::error!("Error inserting member_account_hash into storage: {:?}", e);
			})
	}

	fn remove(&self, member_identity: &Hash) -> Result<(), ()> {
		self.db.delete(Self::storage_key(member_identity)).map_err(|e| {
			log::error!("Error removing member_account_hash from storage: {:?}", e);
		})
	}

	fn contains_key(&self, member_identity: &Hash) -> bool {
		self.db.key_may_exist(Self::storage_key(member_identity))
	}
}
