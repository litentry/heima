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

/// Passkey data structure
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct PasskeyRecord {
	pub omni_account: AccountId,
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

/// Simplified passkey storage using composite key of omni_account + credential_id
pub struct PasskeyStorage {
	db: Arc<StorageDB>,
}

impl PasskeyStorage {
	pub fn new(db: Arc<StorageDB>) -> Self {
		Self { db }
	}

	fn generate_key(omni_account: &AccountId, credential_id: &str) -> [u8; 32] {
		let mut data = Vec::new();
		data.extend_from_slice(omni_account.as_ref());
		data.extend_from_slice(credential_id.as_bytes());
		blake2_256(&data)
	}

	pub fn get_passkey(
		&self,
		omni_account: &AccountId,
		credential_id: &str,
	) -> Result<Option<PasskeyRecord>, ()> {
		let key = Self::generate_key(omni_account, credential_id);
		self.get(&key)
	}

	pub fn remove_passkey(
		&self,
		omni_account: &AccountId,
		credential_id: &str,
	) -> Result<(), PasskeyError> {
		let key = Self::generate_key(omni_account, credential_id);
		self.remove(&key).map_err(|_| PasskeyError::StorageError)
	}

	pub fn exists_passkey(&self, omni_account: &AccountId, credential_id: &str) -> bool {
		let key = Self::generate_key(omni_account, credential_id);
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
			omni_account: omni_account.clone(),
			credential_id: credential_id.to_string(),
			pubkey: pubkey.to_vec(),
			created_at: current_time,
		};

		let key = Self::generate_key(&record.omni_account, &record.credential_id);
		if self.contains_key(&key) {
			return Err(PasskeyError::DuplicatePasskey);
		}
		self.insert(&key, record).map_err(|_| PasskeyError::StorageError)
	}
}

impl Storage<[u8; 32], PasskeyRecord> for PasskeyStorage {
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
		assert_eq!(retrieved.omni_account, omni_account);
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
}
