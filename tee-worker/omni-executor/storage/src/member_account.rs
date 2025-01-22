use executor_core::storage::Storage;
use parity_scale_codec::{Decode, Encode};
use primitives::{AccountId, Hash};
use rocksdb::DB;
use std::path::Path;

const STORAGE_NAME: &str = "member_account_hash";

pub struct MemberAccountStorage {
	db: DB,
}

impl MemberAccountStorage {
	pub fn new() -> Self {
		Self::default()
	}
}

impl Default for MemberAccountStorage {
	fn default() -> Self {
		let path = Path::new(crate::STORAGE_PATH).join(STORAGE_NAME);
		let db = DB::open_default(path).expect("Failed to open database");
		Self { db }
	}
}

impl Storage<Hash, AccountId> for MemberAccountStorage {
	fn get(&self, member_identity: &Hash) -> Option<AccountId> {
		match self.db.get(member_identity.encode()) {
			Ok(Some(value)) => AccountId::decode(&mut &value[..]).ok(),
			_ => {
				log::error!("Error getting member_account_hash from storage");
				None
			},
		}
	}

	fn insert(&self, member_identity: Hash, omni_account: AccountId) -> Result<(), ()> {
		self.db.put(member_identity.encode(), omni_account.encode()).map_err(|e| {
			log::error!("Error inserting member_account_hash into storage: {:?}", e);
		})
	}

	fn remove(&self, member_identity: &Hash) -> Result<(), ()> {
		self.db.delete(member_identity.encode()).map_err(|e| {
			log::error!("Error removing member_account_hash from storage: {:?}", e);
		})
	}

	fn contains_key(&self, member_identity: &Hash) -> bool {
		self.db.key_may_exist(member_identity.encode())
	}
}
