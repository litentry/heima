// Copyright 2020-2024 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.

use crate::{Storage, StorageDB};
use executor_crypto::hashing::{blake2_256, twox_128};
use executor_primitives::AccountId;
use parity_scale_codec::{Decode, Encode};
use std::sync::Arc;

const STORAGE_NAME: &str = "passkey_storage";

/// Storage key type: (AccountId, Hash(credential_id))
pub type PasskeyStorageKey = (AccountId, [u8; 32]);

/// Passkey data structure
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct PasskeyRecord {
	pub credential_id: String,
	pub pubkey: Vec<u8>, // Store SEC1 bytes directly
	pub created_at: u64,
}

/// Errors that can occur during passkey operations
#[derive(Debug, PartialEq, Eq)]
pub enum PasskeyError {
	StorageError,
	DuplicatePasskey,
	ValidationError,
}

/// Passkey storage using composite key of (omni_account, Hash(credential_id))
pub struct PasskeyStorage {
	db: Arc<StorageDB>,
}

impl PasskeyStorage {
	pub fn new(db: Arc<StorageDB>) -> Self {
		Self { db }
	}

	fn make_key(omni_account: &AccountId, credential_id: &str) -> PasskeyStorageKey {
		(omni_account.clone(), blake2_256(credential_id.as_bytes()))
	}

	pub fn get_passkey(
		&self,
		omni_account: &AccountId,
		credential_id: &str,
	) -> Result<Option<PasskeyRecord>, ()> {
		let key = Self::make_key(omni_account, credential_id);
		self.get(&key)
	}

	pub fn remove_passkey(
		&self,
		omni_account: &AccountId,
		credential_id: &str,
	) -> Result<(), PasskeyError> {
		let key = Self::make_key(omni_account, credential_id);
		self.remove(&key).map_err(|_| PasskeyError::StorageError)
	}

	pub fn exists_passkey(&self, omni_account: &AccountId, credential_id: &str) -> bool {
		let key = Self::make_key(omni_account, credential_id);
		self.contains_key(&key)
	}

	pub fn add_passkey(
		&self,
		omni_account: &AccountId,
		credential_id: &str,
		pubkey: &[u8], // Accept SEC1 bytes directly
	) -> Result<(), PasskeyError> {
		let current_time = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_secs();

		let record = PasskeyRecord {
			credential_id: credential_id.to_string(),
			pubkey: pubkey.to_vec(),
			created_at: current_time,
		};

		let key = Self::make_key(omni_account, credential_id);
		if self.contains_key(&key) {
			return Err(PasskeyError::DuplicatePasskey);
		}
		self.insert(&key, record).map_err(|_| PasskeyError::StorageError)
	}

	/// List all passkeys for a given omni_account
	/// Returns a vector of tuples (credential_id, PasskeyRecord)
	pub fn list_passkeys(
		&self,
		omni_account: &AccountId,
	) -> Result<Vec<(String, PasskeyRecord)>, PasskeyError> {
		let mut passkeys = Vec::new();

		// The storage key structure is:
		// storage_key = twox_128(storage_name) + blake2_128_concat(encoded_tuple_key)
		// where blake2_128_concat(x) = blake2_128(x) + x
		//
		// For a tuple key (AccountId, [u8; 32]), the encoded form is:
		// AccountId.encode() + [u8; 32].encode()
		// = AccountId bytes + credential_id_hash bytes
		//
		// We want to match all keys for a given AccountId, so we need to construct
		// a prefix that covers:
		// twox_128(storage_name) + blake2_128(full_key) + AccountId.encode() + ...
		//
		// However, blake2_128 hashes the ENTIRE encoded key, so we can't just match
		// on the AccountId portion. Instead, we iterate through ALL keys in this storage
		// and filter by AccountId.

		let storage_prefix = twox_128(STORAGE_NAME.as_bytes());
		let db = self.db();
		let iter = db.prefix_iterator(storage_prefix);

		for item in iter {
			match item {
				Ok((key, value)) => {
					// Verify the key belongs to our storage namespace
					if !key.starts_with(&storage_prefix) {
						// We've moved past our namespace, stop iteration
						break;
					}

					// The key structure after storage_prefix is:
					// blake2_128(encoded_tuple) + encoded_tuple
					// where encoded_tuple = AccountId + [u8; 32]
					//
					// Skip the blake2_128 hash (16 bytes) to get to the encoded tuple
					let encoded_tuple_start = storage_prefix.len() + 16;
					if key.len() < encoded_tuple_start + 32 {
						// Invalid key structure, skip
						continue;
					}

					// Extract the AccountId from the key (32 bytes after the hash)
					let key_account_bytes = &key[encoded_tuple_start..encoded_tuple_start + 32];

					// Compare with our target omni_account
					if key_account_bytes == <AccountId as AsRef<[u8]>>::as_ref(omni_account) {
						// This key belongs to our account, decode the record
						match PasskeyRecord::decode(&mut &value[..]) {
							Ok(record) => {
								passkeys.push((record.credential_id.clone(), record));
							},
							Err(e) => {
								tracing::warn!(
									"Failed to decode passkey record for account {:?}: {:?}",
									omni_account,
									e
								);
								// Continue iteration despite decode error
							},
						}
					}
				},
				Err(e) => {
					tracing::error!("Error iterating through passkeys: {:?}", e);
					return Err(PasskeyError::StorageError);
				},
			}
		}

		Ok(passkeys)
	}
}

impl Storage<PasskeyStorageKey, PasskeyRecord> for PasskeyStorage {
	fn db(&self) -> Arc<StorageDB> {
		self.db.clone()
	}

	fn name(&self) -> &'static str {
		STORAGE_NAME
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs;
	use std::path::Path;

	fn create_test_storage() -> PasskeyStorage {
		use std::sync::atomic::{AtomicUsize, Ordering};
		static COUNTER: AtomicUsize = AtomicUsize::new(0);
		let db_path = format!("test_passkey_storage_{}", COUNTER.fetch_add(1, Ordering::SeqCst));
		if Path::new(&db_path).exists() {
			fs::remove_dir_all(&db_path).unwrap();
		}
		let db = Arc::new(StorageDB::open_default(&db_path).unwrap());
		PasskeyStorage::new(db)
	}

	#[test]
	fn test_passkey_add_and_get() {
		let storage = create_test_storage();
		let omni_account = AccountId::from([1u8; 32]);

		// Add a passkey
		let test_pubkey = b"test_pubkey_123";
		storage.add_passkey(&omni_account, "cred123", test_pubkey).unwrap();

		// Test retrieval
		let retrieved = storage.get_passkey(&omni_account, "cred123").unwrap().unwrap();
		assert_eq!(retrieved.credential_id, "cred123");
		assert_eq!(retrieved.pubkey, test_pubkey.to_vec());

		// Test existence check
		assert!(storage.exists_passkey(&omni_account, "cred123"));
		assert!(!storage.exists_passkey(&omni_account, "nonexistent"));
	}

	#[test]
	fn test_passkey_removal() {
		let storage = create_test_storage();
		let omni_account = AccountId::from([2u8; 32]);

		// Add a passkey
		let test_pubkey = b"test_pubkey_456";
		storage.add_passkey(&omni_account, "cred456", test_pubkey).unwrap();

		// Verify it exists
		assert!(storage.exists_passkey(&omni_account, "cred456"));

		// Remove it
		storage.remove_passkey(&omni_account, "cred456").unwrap();

		// Verify it's gone
		assert!(!storage.exists_passkey(&omni_account, "cred456"));
		assert!(storage.get_passkey(&omni_account, "cred456").unwrap().is_none());
	}

	#[test]
	fn test_duplicate_detection() {
		let storage = create_test_storage();
		let omni_account = AccountId::from([3u8; 32]);

		// Add first passkey
		let test_pubkey1 = b"test_pubkey_789";
		storage.add_passkey(&omni_account, "cred789", test_pubkey1).unwrap();

		// Try to add with same omni_account + credential_id
		let test_pubkey2 = b"test_pubkey_789_new";
		let result = storage.add_passkey(&omni_account, "cred789", test_pubkey2);
		assert_eq!(result, Err(PasskeyError::DuplicatePasskey));
	}

	#[test]
	fn test_multiple_passkeys_per_omni_account() {
		let storage = create_test_storage();
		let omni_account = AccountId::from([4u8; 32]);

		// Add multiple passkeys for the same omni account
		let test_pubkey1 = b"test_pubkey_1";
		let test_pubkey2 = b"test_pubkey_2";
		storage.add_passkey(&omni_account, "cred1", test_pubkey1).unwrap();
		storage.add_passkey(&omni_account, "cred2", test_pubkey2).unwrap();

		// Both should exist independently
		assert!(storage.exists_passkey(&omni_account, "cred1"));
		assert!(storage.exists_passkey(&omni_account, "cred2"));

		let record1 = storage.get_passkey(&omni_account, "cred1").unwrap().unwrap();
		let record2 = storage.get_passkey(&omni_account, "cred2").unwrap().unwrap();

		assert_eq!(record1.pubkey, test_pubkey1.to_vec());
		assert_eq!(record2.pubkey, test_pubkey2.to_vec());
	}

	#[test]
	fn test_list_passkeys() {
		let storage = create_test_storage();
		let omni_account1 = AccountId::from([5u8; 32]);
		let omni_account2 = AccountId::from([6u8; 32]);

		// Add multiple passkeys for account1
		let test_pubkey1 = b"test_pubkey_1";
		let test_pubkey2 = b"test_pubkey_2";
		let test_pubkey3 = b"test_pubkey_3";
		storage.add_passkey(&omni_account1, "cred1", test_pubkey1).unwrap();
		storage.add_passkey(&omni_account1, "cred2", test_pubkey2).unwrap();
		storage.add_passkey(&omni_account1, "cred3", test_pubkey3).unwrap();

		// Add a passkey for account2
		let test_pubkey4 = b"test_pubkey_4";
		storage.add_passkey(&omni_account2, "cred4", test_pubkey4).unwrap();

		// List passkeys for account1
		let passkeys1 = storage.list_passkeys(&omni_account1).unwrap();
		assert_eq!(passkeys1.len(), 3);

		// Verify all three passkeys are present
		let cred_ids: Vec<String> = passkeys1.iter().map(|(cid, _)| cid.clone()).collect();
		assert!(cred_ids.contains(&"cred1".to_string()));
		assert!(cred_ids.contains(&"cred2".to_string()));
		assert!(cred_ids.contains(&"cred3".to_string()));

		// List passkeys for account2
		let passkeys2 = storage.list_passkeys(&omni_account2).unwrap();
		assert_eq!(passkeys2.len(), 1);
		assert_eq!(passkeys2[0].0, "cred4");
		assert_eq!(passkeys2[0].1.pubkey, test_pubkey4.to_vec());

		// List passkeys for non-existent account
		let omni_account3 = AccountId::from([7u8; 32]);
		let passkeys3 = storage.list_passkeys(&omni_account3).unwrap();
		assert_eq!(passkeys3.len(), 0);
	}
}
