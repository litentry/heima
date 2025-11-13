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
use executor_crypto::hashing::blake2_256;
use executor_primitives::AccountId;
use parity_scale_codec::{Decode, Encode};
use std::sync::Arc;

const STORAGE_NAME: &str = "passkey_storage";
const INDEX_STORAGE_NAME: &str = "passkey_account_index";

/// Storage key type: (AccountId, Hash(credential_id))
pub type PasskeyStorageKey = (AccountId, [u8; 32]);

/// Index storage: Maps AccountId -> Vec<String> (credential_ids)
/// This enables O(1) lookup of all credential IDs for a given account
pub type PasskeyAccountIndexKey = AccountId;
pub type PasskeyAccountIndexValue = Vec<String>;

/// Passkey data structure
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct PasskeyRecord {
	pub credential_id: String,
	pub pubkey: Vec<u8>, // Store SEC1 bytes directly
	pub created_at: u64,
	pub omni_account: AccountId,
	pub alias_name: String,
	pub last_used: u64,
}

/// Errors that can occur during passkey operations
#[derive(Debug, PartialEq, Eq)]
pub enum PasskeyError {
	StorageError,
	DuplicatePasskey,
	ValidationError,
}

/// Index storage that maps AccountId to list of credential IDs
pub struct PasskeyAccountIndex {
	db: Arc<StorageDB>,
}

impl PasskeyAccountIndex {
	pub fn new(db: Arc<StorageDB>) -> Self {
		Self { db }
	}

	fn add_credential(&self, omni_account: &AccountId, credential_id: &str) -> Result<(), ()> {
		let mut credentials = self.get(omni_account)?.unwrap_or_default();
		if !credentials.contains(&credential_id.to_string()) {
			credentials.push(credential_id.to_string());
			self.insert(omni_account, credentials)?;
		}
		Ok(())
	}

	fn remove_credential(&self, omni_account: &AccountId, credential_id: &str) -> Result<(), ()> {
		if let Some(mut credentials) = self.get(omni_account)? {
			credentials.retain(|cid| cid != credential_id);
			if credentials.is_empty() {
				self.remove(omni_account)?;
			} else {
				self.insert(omni_account, credentials)?;
			}
		}
		Ok(())
	}

	fn get_credentials(&self, omni_account: &AccountId) -> Result<Vec<String>, ()> {
		Ok(self.get(omni_account)?.unwrap_or_default())
	}
}

impl Storage<PasskeyAccountIndexKey, PasskeyAccountIndexValue> for PasskeyAccountIndex {
	fn db(&self) -> Arc<StorageDB> {
		self.db.clone()
	}

	fn name(&self) -> &'static str {
		INDEX_STORAGE_NAME
	}
}

/// Passkey storage using composite key of (omni_account, Hash(credential_id))
pub struct PasskeyStorage {
	db: Arc<StorageDB>,
	index: PasskeyAccountIndex,
}

impl PasskeyStorage {
	pub fn new(db: Arc<StorageDB>) -> Self {
		let index = PasskeyAccountIndex::new(db.clone());
		Self { db, index }
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

		self.remove(&key).map_err(|_| PasskeyError::StorageError)?;

		self.index
			.remove_credential(omni_account, credential_id)
			.map_err(|_| PasskeyError::StorageError)?;

		Ok(())
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
		alias_name: Option<String>,
	) -> Result<(), PasskeyError> {
		let current_time = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_secs();

		let alias = alias_name.unwrap_or_else(|| credential_id.to_string());

		let record = PasskeyRecord {
			credential_id: credential_id.to_string(),
			pubkey: pubkey.to_vec(),
			created_at: current_time,
			omni_account: omni_account.clone(),
			alias_name: alias,
			last_used: current_time,
		};

		let key = Self::make_key(omni_account, credential_id);
		if self.contains_key(&key) {
			return Err(PasskeyError::DuplicatePasskey);
		}

		self.insert(&key, record).map_err(|_| PasskeyError::StorageError)?;

		self.index
			.add_credential(omni_account, credential_id)
			.map_err(|_| PasskeyError::StorageError)?;

		Ok(())
	}

	/// List all passkeys for a given omni_account
	/// Returns a vector of tuples (credential_id, alias_name, created_at, last_used)
	pub fn list_passkeys(
		&self,
		omni_account: &AccountId,
	) -> Result<Vec<(String, String, u64, u64)>, PasskeyError> {
		let mut passkeys = Vec::new();

		let credential_ids = self
			.index
			.get_credentials(omni_account)
			.map_err(|_| PasskeyError::StorageError)?;

		for credential_id in credential_ids {
			match self.get_passkey(omni_account, &credential_id) {
				Ok(Some(record)) => {
					passkeys.push((
						record.credential_id.clone(),
						record.alias_name.clone(),
						record.created_at,
						record.last_used,
					));
				},
				Ok(None) => {
					tracing::warn!(
						"Passkey index references non-existent credential: {} for account {:?}",
						credential_id,
						omni_account
					);
					// Continue despite missing record
				},
				Err(_) => {
					tracing::error!(
						"Error retrieving passkey record for credential: {}",
						credential_id
					);
					return Err(PasskeyError::StorageError);
				},
			}
		}

		Ok(passkeys)
	}

	/// Rename the alias name of a passkey
	pub fn rename_passkey_alias(
		&self,
		omni_account: &AccountId,
		credential_id: &str,
		new_alias_name: String,
	) -> Result<(), PasskeyError> {
		let key = Self::make_key(omni_account, credential_id);

		// Get the existing record
		let mut record = self
			.get(&key)
			.map_err(|_| PasskeyError::StorageError)?
			.ok_or(PasskeyError::ValidationError)?;

		// Update the alias name
		record.alias_name = new_alias_name;

		// Save the updated record
		self.insert(&key, record).map_err(|_| PasskeyError::StorageError)
	}

	/// Update the last_used timestamp for a passkey
	pub fn update_last_used(
		&self,
		omni_account: &AccountId,
		credential_id: &str,
	) -> Result<(), PasskeyError> {
		let key = Self::make_key(omni_account, credential_id);

		// Get the existing record
		let mut record = self
			.get(&key)
			.map_err(|_| PasskeyError::StorageError)?
			.ok_or(PasskeyError::ValidationError)?;

		// Update the last_used timestamp
		let current_time = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_secs();
		record.last_used = current_time;

		// Save the updated record
		self.insert(&key, record).map_err(|_| PasskeyError::StorageError)
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

	fn create_test_storage() -> (PasskeyStorage, String) {
		use std::sync::atomic::{AtomicUsize, Ordering};
		static COUNTER: AtomicUsize = AtomicUsize::new(0);
		let db_path = format!("test_passkey_storage_{}", COUNTER.fetch_add(1, Ordering::SeqCst));
		if Path::new(&db_path).exists() {
			fs::remove_dir_all(&db_path).unwrap();
		}
		let db = Arc::new(StorageDB::open_default(&db_path).unwrap());
		(PasskeyStorage::new(db), db_path)
	}

	fn remove_test_storage(db_path: &str) {
		if Path::new(db_path).exists() {
			fs::remove_dir_all(db_path).unwrap();
		}
	}

	#[test]
	fn test_passkey_add_and_get() {
		let (storage, db_path) = create_test_storage();
		let omni_account = AccountId::from([1u8; 32]);

		// Add a passkey
		let test_pubkey = b"test_pubkey_123";
		storage.add_passkey(&omni_account, "cred123", test_pubkey, None).unwrap();

		// Test retrieval
		let retrieved = storage.get_passkey(&omni_account, "cred123").unwrap().unwrap();
		assert_eq!(retrieved.credential_id, "cred123");
		assert_eq!(retrieved.pubkey, test_pubkey.to_vec());

		// Test existence check
		assert!(storage.exists_passkey(&omni_account, "cred123"));
		assert!(!storage.exists_passkey(&omni_account, "nonexistent"));

		// Cleanup
		remove_test_storage(&db_path);
	}

	#[test]
	fn test_passkey_removal() {
		let (storage, db_path) = create_test_storage();
		let omni_account = AccountId::from([2u8; 32]);

		// Add a passkey
		let test_pubkey = b"test_pubkey_456";
		storage.add_passkey(&omni_account, "cred456", test_pubkey, None).unwrap();

		// Verify it exists
		assert!(storage.exists_passkey(&omni_account, "cred456"));

		// Remove it
		storage.remove_passkey(&omni_account, "cred456").unwrap();

		// Verify it's gone
		assert!(!storage.exists_passkey(&omni_account, "cred456"));
		assert!(storage.get_passkey(&omni_account, "cred456").unwrap().is_none());

		// Cleanup
		remove_test_storage(&db_path);
	}

	#[test]
	fn test_duplicate_detection() {
		let (storage, db_path) = create_test_storage();
		let omni_account = AccountId::from([3u8; 32]);

		// Add first passkey
		let test_pubkey1 = b"test_pubkey_789";
		storage.add_passkey(&omni_account, "cred789", test_pubkey1, None).unwrap();

		// Try to add with same omni_account + credential_id
		let test_pubkey2 = b"test_pubkey_789_new";
		let result = storage.add_passkey(&omni_account, "cred789", test_pubkey2, None);
		assert_eq!(result, Err(PasskeyError::DuplicatePasskey));

		// Cleanup
		remove_test_storage(&db_path);
	}

	#[test]
	fn test_multiple_passkeys_per_omni_account() {
		let (storage, db_path) = create_test_storage();
		let omni_account = AccountId::from([4u8; 32]);

		// Add multiple passkeys for the same omni account
		let test_pubkey1 = b"test_pubkey_1";
		let test_pubkey2 = b"test_pubkey_2";
		storage.add_passkey(&omni_account, "cred1", test_pubkey1, None).unwrap();
		storage.add_passkey(&omni_account, "cred2", test_pubkey2, None).unwrap();

		// Both should exist independently
		assert!(storage.exists_passkey(&omni_account, "cred1"));
		assert!(storage.exists_passkey(&omni_account, "cred2"));

		let record1 = storage.get_passkey(&omni_account, "cred1").unwrap().unwrap();
		let record2 = storage.get_passkey(&omni_account, "cred2").unwrap().unwrap();

		assert_eq!(record1.pubkey, test_pubkey1.to_vec());
		assert_eq!(record2.pubkey, test_pubkey2.to_vec());

		// Cleanup
		remove_test_storage(&db_path);
	}

	#[test]
	fn test_list_passkeys() {
		let (storage, db_path) = create_test_storage();
		let omni_account1 = AccountId::from([5u8; 32]);
		let omni_account2 = AccountId::from([6u8; 32]);

		// Add multiple passkeys for account1
		let test_pubkey1 = b"test_pubkey_1";
		let test_pubkey2 = b"test_pubkey_2";
		let test_pubkey3 = b"test_pubkey_3";
		storage.add_passkey(&omni_account1, "cred1", test_pubkey1, None).unwrap();
		storage.add_passkey(&omni_account1, "cred2", test_pubkey2, None).unwrap();
		storage
			.add_passkey(&omni_account1, "cred3", test_pubkey3, Some("My Passkey".to_string()))
			.unwrap();

		// Add a passkey for account2
		let test_pubkey4 = b"test_pubkey_4";
		storage.add_passkey(&omni_account2, "cred4", test_pubkey4, None).unwrap();

		// List passkeys for account1
		let passkeys1 = storage.list_passkeys(&omni_account1).unwrap();
		assert_eq!(passkeys1.len(), 3);

		// Verify all three passkeys are present (now returns credential_id, alias_name, created_at, last_used)
		let credential_ids: Vec<String> =
			passkeys1.iter().map(|(cred_id, _, _, _)| cred_id.clone()).collect();
		assert!(credential_ids.contains(&"cred1".to_string()));
		assert!(credential_ids.contains(&"cred2".to_string()));
		assert!(credential_ids.contains(&"cred3".to_string()));

		let alias_names: Vec<String> =
			passkeys1.iter().map(|(_, alias, _, _)| alias.clone()).collect();
		assert!(alias_names.contains(&"cred1".to_string()));
		assert!(alias_names.contains(&"cred2".to_string()));
		assert!(alias_names.contains(&"My Passkey".to_string()));

		// List passkeys for account2
		let passkeys2 = storage.list_passkeys(&omni_account2).unwrap();
		assert_eq!(passkeys2.len(), 1);
		assert_eq!(passkeys2[0].0, "cred4");
		assert_eq!(passkeys2[0].1, "cred4");

		// List passkeys for non-existent account
		let omni_account3 = AccountId::from([7u8; 32]);
		let passkeys3 = storage.list_passkeys(&omni_account3).unwrap();
		assert_eq!(passkeys3.len(), 0);

		// Cleanup
		remove_test_storage(&db_path);
	}
}
