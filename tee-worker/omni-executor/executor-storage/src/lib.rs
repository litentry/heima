use executor_crypto::hashing::{blake2_128, twox_128};
use parity_scale_codec::{Codec, Decode, Encode};
use rocksdb::{WriteOptions, DB};
use std::{sync::Arc, vec::Vec};

mod verification_code;
pub use verification_code::VerificationCodeStorage;
mod oauth2_state_verifier;
pub use oauth2_state_verifier::OAuth2StateVerifierStorage;
mod heima_jwt;
pub use heima_jwt::HeimaJwtStorage;
mod intent_id;
pub use intent_id::IntentIdStorage;
mod asset_lock;
mod passkey;
pub use passkey::{PasskeyError, PasskeyRecord, PasskeyStorage, PasskeyStorageKey};
mod passkey_challenge;
pub use passkey_challenge::{
	PasskeyChallengeError, PasskeyChallengeRecord, PasskeyChallengeStorage,
};
mod pumpx_account_profile;
pub use pumpx_account_profile::PumpxProfileStorage;
mod wildmeta_timestamp;
pub use wildmeta_timestamp::WildmetaTimestampStorage;
pub mod loan_record;
pub use loan_record::{LoanRecord, LoanRecordStorage, LoanState};

pub use asset_lock::AssetLockStorage;
pub use asset_lock::Key as AssetLockStorageKey;
use tracing::error;

const STORAGE_DB_PATH: &str = "storage_db";
const STORAGE_VERSION_KEY: &[u8] = b"__storage_version__";

pub type StorageDB = DB;

pub trait Storage<K: Encode, V: Codec> {
	fn db(&self) -> Arc<StorageDB>;
	fn name(&self) -> &'static str;

	fn get(&self, key: &K) -> Result<Option<V>, ()> {
		match self.db().get(storage_key(self.name(), &key.encode())) {
			Ok(Some(v)) => {
				let decoded_v = V::decode(&mut &v[..]).map_err(|e| {
					error!("Error decoding value from storage: {:?}", e);
				})?;
				Ok(Some(decoded_v))
			},
			Ok(None) => Ok(None),
			Err(e) => {
				error!("Error getting value from storage: {:?}", e);
				Err(())
			},
		}
	}
	fn contains_key(&self, key: &K) -> bool {
		self.db().key_may_exist(storage_key(self.name(), &key.encode()))
	}

	fn insert(&self, key: &K, value: V) -> Result<(), ()> {
		let mut opts = WriteOptions::default();
		opts.set_sync(true);
		self.db()
			.put_opt(storage_key(self.name(), &key.encode()), value.encode(), &opts)
			.map_err(|e| {
				error!("Error inserting value into storage: {:?}", e);
			})
	}

	fn remove(&self, key: &K) -> Result<(), ()> {
		let mut opts = WriteOptions::default();
		opts.set_sync(true);
		self.db()
			.delete_opt(storage_key(self.name(), &key.encode()), &opts)
			.map_err(|e| {
				error!("Error removing value from storage: {:?}", e);
			})
	}
}

pub fn storage_key(storage_name: &str, key: &[u8]) -> Vec<u8> {
	twox_128(storage_name.as_bytes())
		.iter()
		.chain(blake2_128(key).iter().chain(key.iter())) // blake2_128_concat
		.cloned()
		.collect()
}

fn get_storage_version(db: &StorageDB) -> u32 {
	match db.get(STORAGE_VERSION_KEY) {
		Ok(Some(v)) => u32::decode(&mut &v[..]).unwrap_or(0),
		Ok(None) => 0,
		Err(e) => {
			error!("Error getting storage version: {:?}", e);
			0
		},
	}
}

#[allow(dead_code)] // TODO: remove this when adding the first migration
fn set_storage_version(db: &StorageDB, version: u32) -> Result<(), ()> {
	let mut opts = WriteOptions::default();
	opts.set_sync(true);
	db.put_opt(STORAGE_VERSION_KEY, version.encode(), &opts).map_err(|e| {
		error!("Error setting storage version: {:?}", e);
	})
}

pub async fn init_storage() -> Result<Arc<StorageDB>, ()> {
	let db = Arc::new(StorageDB::open_default(STORAGE_DB_PATH).map_err(|e| {
		error!("Could not open db: {:?}", e);
	})?);

	// Migration example: if version == 0, do migration, then set to 1
	let current_version = get_storage_version(&db);
	if current_version == 0 {
		// ... perform migration logic here ...
		// set_storage_version(&db, 1)?;
	}

	// Init storage here:

	Ok(db)
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs;
	use std::path::Path;

	#[test]
	fn test_storage_version_helpers() {
		let db_path = "test_storage_db";
		if Path::new(db_path).exists() {
			fs::remove_dir_all(db_path).unwrap();
		}
		let db = StorageDB::open_default(db_path).unwrap();

		// Initially, version should be 0
		assert_eq!(get_storage_version(&db), 0);

		// Set version to 1
		set_storage_version(&db, 1).unwrap();
		assert_eq!(get_storage_version(&db), 1);

		// Set version to 42
		set_storage_version(&db, 42).unwrap();
		assert_eq!(get_storage_version(&db), 42);

		fs::remove_dir_all(db_path).unwrap();
	}

	#[test]
	fn test_struct_migration_v0_to_v1() {
		use parity_scale_codec::{Decode, Encode};
		use std::fs;
		use std::path::Path;

		#[derive(Debug, PartialEq, Encode, Decode)]
		struct TestDataV0 {
			a: u32,
			b: String,
		}

		#[derive(Debug, PartialEq, Encode, Decode)]
		struct TestDataV1 {
			a: u32,
			c: Option<u64>,
		}

		let db_path = "test_struct_migration_db_scale";
		if Path::new(db_path).exists() {
			fs::remove_dir_all(db_path).unwrap();
		}
		let db = StorageDB::open_default(db_path).unwrap();

		// Simulate old data (version 0)
		let old = TestDataV0 { a: 42, b: "hello".to_string() };
		db.put(b"testkey", old.encode()).unwrap();
		set_storage_version(&db, 0).unwrap();

		// Migration: if version == 0, read old, convert, write new, set version = 1
		if get_storage_version(&db) == 0 {
			let bytes = db.get(b"testkey").unwrap().unwrap();
			let old = TestDataV0::decode(&mut &bytes[..]).unwrap();
			let new = TestDataV1 { a: old.a, c: None };
			db.put(b"testkey", new.encode()).unwrap();
			set_storage_version(&db, 1).unwrap();
		}

		// Check migrated data
		let bytes = db.get(b"testkey").unwrap().unwrap();
		let new = TestDataV1::decode(&mut &bytes[..]).unwrap();
		assert_eq!(new, TestDataV1 { a: 42, c: None });
		assert_eq!(get_storage_version(&db), 1);

		fs::remove_dir_all(db_path).unwrap();
	}
}
