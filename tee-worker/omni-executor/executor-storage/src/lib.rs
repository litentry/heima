use executor_crypto::hashing::{blake2_128, twox_128};
use executor_primitives::{AccountId, MemberAccount};
use frame_support::sp_runtime::traits::BlakeTwo256;
use frame_support::storage::storage_prefix;
use parentchain_api_interface::omni_account::storage::types::account_store::AccountStore;
use parentchain_rpc_client::{
	CustomConfig, SubstrateRpcClient, SubstrateRpcClientFactory, SubxtClient, SubxtClientFactory,
	ToPrimitiveType,
};
use parity_scale_codec::{Codec, Decode, Encode};
use rocksdb::{WriteOptions, DB};
use sp_state_machine::{read_proof_check, StorageProof};
use std::{sync::Arc, vec::Vec};

mod member_omni_account;
pub use member_omni_account::MemberOmniAccountStorage;
mod verification_code;
pub use verification_code::VerificationCodeStorage;
mod account_store;
pub use account_store::AccountStoreStorage;
mod oauth2_state_verifier;
pub use oauth2_state_verifier::OAuth2StateVerifierStorage;
mod pumpx_jwt;
pub use pumpx_jwt::PumpxJwtStorage;
mod intent_id;
pub use intent_id::IntentIdStorage;
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

pub fn get_storage_version(db: &StorageDB) -> u32 {
	match db.get(STORAGE_VERSION_KEY) {
		Ok(Some(v)) => u32::decode(&mut &v[..]).unwrap_or(0),
		Ok(None) => 0,
		Err(e) => {
			error!("Error getting storage version: {:?}", e);
			0
		},
	}
}

pub fn set_storage_version(db: &StorageDB, version: u32) -> Result<(), ()> {
	let mut opts = WriteOptions::default();
	opts.set_sync(true);
	db.put_opt(STORAGE_VERSION_KEY, version.encode(), &opts).map_err(|e| {
		error!("Error setting storage version: {:?}", e);
	})
}

pub async fn init_storage(ws_rpc_endpoint: &str) -> Result<Arc<StorageDB>, ()> {
	let db = Arc::new(StorageDB::open_default(STORAGE_DB_PATH).map_err(|e| {
		error!("Could not open db: {:?}", e);
	})?);

	// Migration example: if version == 0, do migration, then set to 1
	let current_version = get_storage_version(&db);
	if current_version == 0 {
		// ... perform migration logic here ...
		// set_storage_version(&db, 1)?;
	}

	let client_factory: SubxtClientFactory<CustomConfig> = SubxtClientFactory::new(ws_rpc_endpoint);
	let mut client = client_factory.new_client().await.map_err(|e| {
		error!("Could not create client: {:?}", e);
	})?;

	init_omni_account_storages(&mut client, db.clone()).await?;

	Ok(db)
}

const ACCOUNT_STORE_KEYS_PAGE_SIZE: u32 = 300;

async fn init_omni_account_storages(
	client: &mut SubxtClient<CustomConfig>,
	storage_db: Arc<StorageDB>,
) -> Result<(), ()> {
	let account_store_storage = AccountStoreStorage::new(storage_db.clone());
	let member_omni_account_storage = MemberOmniAccountStorage::new(storage_db.clone());
	let account_store_key_prefix = storage_prefix(b"OmniAccount", b"AccountStore");
	let mut start_key: Option<Vec<u8>> = None;

	loop {
		let storage_keys_paged = client
			.get_storage_keys_paged(
				account_store_key_prefix.into(),
				ACCOUNT_STORE_KEYS_PAGE_SIZE,
				start_key.clone(),
			)
			.await
			.map_err(|e| {
				error!("Could not get storage keys paged: {:?}", e);
			})?;
		if storage_keys_paged.is_empty() || storage_keys_paged.last().cloned() == start_key {
			break;
		}
		start_key = storage_keys_paged.last().cloned();
		let proof =
			client
				.get_storage_proof_by_keys(storage_keys_paged.clone())
				.await
				.map_err(|e| {
					error!("Could not get storage proof by keys: {:?}", e);
				})?;
		let header = match client.get_last_finalized_header().await {
			Ok(header) => header,
			_ => {
				error!("Could not get last finalized header");
				return Err(());
			},
		};
		let storage_proof = StorageProof::new(proof);
		let storage_map = read_proof_check::<BlakeTwo256, _>(
			header.state_root,
			storage_proof,
			&storage_keys_paged,
		)
		.map_err(|e| {
			error!("Could not read proof check: {:?}", e);
		})?;

		for key in storage_keys_paged.iter() {
			match storage_map.get(key) {
				Some(Some(value)) => {
					let omni_account: AccountId = extract_account_id_from_storage_key(key)?;
					let maybe_storage_value = client
						.storage()
						.at_latest()
						.await
						.map_err(|e| {
							error!("Could not get storage at latest block: {:?}", e);
						})?
						.fetch_raw(key.clone())
						.await
						.map_err(|e| {
							error!("Could not fetch storage value: {:?}", e);
						})?;
					let Some(storage_value) = maybe_storage_value else {
						error!("Storage value not found for account_id: {:?}", omni_account);
						return Err(());
					};
					if storage_value != *value {
						error!("Storage value mismatch for account_id: {:?}", omni_account);
						return Err(());
					}
					let account_store: AccountStore =
						Decode::decode(&mut &value[..]).map_err(|e| {
							error!("Error decoding account store: {:?}", e);
						})?;
					let mut member_accounts: Vec<MemberAccount> = Vec::new();
					for member in account_store.0.iter() {
						let member_account: MemberAccount = member.to_primitive_type();
						member_omni_account_storage
							.insert(&member_account.hash(), omni_account.clone())
							.map_err(|e| {
								error!("Error inserting member account hash: {:?}", e);
							})?;
						member_accounts.push(member_account);
					}
					account_store_storage.insert(&omni_account, member_accounts).map_err(|e| {
						error!("Error inserting account store: {:?}", e);
					})?;
				},
				_ => {
					error!("No value found for key: {:?}", key);
				},
			}
		}
	}

	Ok(())
}

fn extract_account_id_from_storage_key<K: Decode>(raw_storage_key: &[u8]) -> Result<K, ()> {
	if raw_storage_key.len() < 32 {
		return Err(());
	}
	let mut raw_key = &raw_storage_key[raw_storage_key.len() - 32..];
	K::decode(&mut raw_key).map_err(|e| {
		error!("Error decoding account id: {:?}", e);
	})
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
