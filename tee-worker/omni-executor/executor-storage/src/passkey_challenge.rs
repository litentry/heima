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
use executor_primitives::AccountId;
use parity_scale_codec::{Decode, Encode};
use std::sync::Arc;

const STORAGE_NAME: &str = "passkey_challenge_storage";

/// Passkey challenge record with expiration
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct PasskeyChallengeRecord {
	pub omni_account: AccountId,
	pub created_at: u64,
	pub expires_at: u64,
}

/// Errors that can occur during challenge operations
#[derive(Debug, PartialEq, Eq)]
pub enum PasskeyChallengeError {
	StorageError,
	ChallengeExpired,
	ChallengeNotFound,
	InvalidChallenge,
}

/// PassKey challenge storage with expiration management
#[derive(Clone)]
pub struct PasskeyChallengeStorage {
	db: Arc<StorageDB>,
}

impl PasskeyChallengeStorage {
	pub fn new(db: Arc<StorageDB>) -> Self {
		Self { db }
	}

	pub fn store_challenge(
		&self,
		omni_account: &AccountId,
		challenge: &str,
		timeout_seconds: u64,
	) -> Result<(), PasskeyChallengeError> {
		let current_time = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_secs();

		let record = PasskeyChallengeRecord {
			omni_account: omni_account.clone(),
			created_at: current_time,
			expires_at: current_time + timeout_seconds,
		};

		self.insert(&challenge, record).map_err(|_| PasskeyChallengeError::StorageError)
	}

	pub fn verify_and_consume_challenge(
		&self,
		challenge: &str,
		omni_account: &AccountId,
	) -> Result<(), PasskeyChallengeError> {
		let record = self
			.get(&challenge)
			.map_err(|_| PasskeyChallengeError::StorageError)?
			.ok_or(PasskeyChallengeError::ChallengeNotFound)?;

		if record.omni_account != *omni_account {
			return Err(PasskeyChallengeError::InvalidChallenge);
		}

		let current_time = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_secs();

		if current_time > record.expires_at {
			self.remove(&challenge).map_err(|_| PasskeyChallengeError::StorageError)?;
			return Err(PasskeyChallengeError::ChallengeExpired);
		}

		self.remove(&challenge).map_err(|_| PasskeyChallengeError::StorageError)?;

		let cleanup_storage = self.clone();

		let _ = std::thread::spawn(move || {
			if let Err(e) = cleanup_storage.cleanup_expired24h_challenges() {
				tracing::error!(
					"Failed to cleanup expired passkey challenges after verification: {:?}",
					e
				);
			}
		});

		Ok(())
	}

	/// Clean up challenges that expired more than 24 hours ago
	///
	/// This method only removes challenges that expired at least 24 hours ago.
	/// This grace period ensures users get accurate error messages (ChallengeExpired
	/// vs ChallengeNotFound) for recently expired challenges.
	///
	/// It's recommended to call this periodically (e.g., every hour) via a background job.
	pub fn cleanup_expired24h_challenges(&self) -> Result<u32, PasskeyChallengeError> {
		let current_time = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_secs();

		// Only cleanup challenges that expired 24 hours ago or more
		const CLEANUP_GRACE_PERIOD_SECONDS: u64 = 24 * 60 * 60; // 24 hours
		let cleanup_threshold = current_time.saturating_sub(CLEANUP_GRACE_PERIOD_SECONDS);

		let mut cleaned_count = 0u32;
		let mut expired_keys = Vec::new();

		// Calculate the prefix for our storage namespace
		// storage_key = twox_128(storage_name) + blake2_128_concat(key)
		// For prefix iteration, we only need the storage name hash
		use executor_crypto::hashing::twox_128;
		let storage_prefix = twox_128(STORAGE_NAME.as_bytes());

		let db = self.db();
		let iter = db.prefix_iterator(storage_prefix);

		// Iterate through all records in our namespace
		for item in iter {
			match item {
				Ok((key, value)) => {
					// Verify the key belongs to our storage (starts with our prefix)
					if !key.starts_with(&storage_prefix) {
						// We've moved past our namespace, stop iteration
						break;
					}

					// Decode the challenge record
					match PasskeyChallengeRecord::decode(&mut &value[..]) {
						Ok(record) => {
							// Only cleanup if expired AND past the 24-hour grace period
							if record.expires_at < cleanup_threshold {
								// Mark for deletion (we can't delete while iterating)
								expired_keys.push(key.to_vec());
								cleaned_count += 1;
							}
						},
						Err(e) => {
							// Log decode error but continue cleanup
							tracing::warn!(
								"Failed to decode challenge record during cleanup: {:?}",
								e
							);
							// Optionally delete corrupted records
							expired_keys.push(key.to_vec());
						},
					}
				},
				Err(e) => {
					tracing::error!("Error iterating through challenges during cleanup: {:?}", e);
					return Err(PasskeyChallengeError::StorageError);
				},
			}
		}

		// Delete all expired challenges
		for key in expired_keys {
			if let Err(e) = db.delete(&key) {
				tracing::error!("Failed to delete expired challenge: {:?}", e);
				// Continue cleanup even if some deletions fail
			}
		}

		if cleaned_count > 0 {
			tracing::info!("Cleaned up {} passkey challenges that expired >24h ago", cleaned_count);
		}

		Ok(cleaned_count)
	}

	/// Check if a challenge exists and is valid (without consuming it)
	pub fn is_challenge_valid(
		&self,
		challenge: &str,
		omni_account: &AccountId,
	) -> Result<bool, PasskeyChallengeError> {
		let record = self.get(&challenge).map_err(|_| PasskeyChallengeError::StorageError)?;

		match record {
			Some(record) => {
				// Check if challenge matches the account
				if record.omni_account != *omni_account {
					return Ok(false);
				}

				// Check expiration
				let current_time = std::time::SystemTime::now()
					.duration_since(std::time::UNIX_EPOCH)
					.unwrap()
					.as_secs();

				Ok(current_time <= record.expires_at)
			},
			None => Ok(false),
		}
	}
}

impl Storage<&str, PasskeyChallengeRecord> for PasskeyChallengeStorage {
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

	fn create_test_storage() -> (PasskeyChallengeStorage, String) {
		use std::sync::atomic::{AtomicUsize, Ordering};
		static COUNTER: AtomicUsize = AtomicUsize::new(0);
		let db_path =
			format!("test_passkey_challenge_storage_{}", COUNTER.fetch_add(1, Ordering::SeqCst));
		if Path::new(&db_path).exists() {
			fs::remove_dir_all(&db_path).unwrap();
		}
		let db = Arc::new(StorageDB::open_default(&db_path).unwrap());
		(PasskeyChallengeStorage::new(db), db_path)
	}

	fn remove_test_storage(db_path: &str) {
		if Path::new(db_path).exists() {
			fs::remove_dir_all(db_path).unwrap();
		}
	}

	#[test]
	fn test_store_and_verify_challenge() {
		let (storage, db_path) = create_test_storage();
		let omni_account = AccountId::from([1u8; 32]);
		let challenge = "test_challenge_123";

		// Store challenge with 60 second timeout
		storage.store_challenge(&omni_account, challenge, 60).unwrap();

		// Verify challenge is valid
		assert!(storage.is_challenge_valid(challenge, &omni_account).unwrap());

		// Verify and consume challenge
		storage.verify_and_consume_challenge(challenge, &omni_account).unwrap();

		// Challenge should be consumed (no longer valid)
		assert!(!storage.is_challenge_valid(challenge, &omni_account).unwrap());

		// Cleanup
		remove_test_storage(&db_path);
	}

	#[test]
	fn test_challenge_expiration() {
		let (storage, db_path) = create_test_storage();
		let omni_account = AccountId::from([2u8; 32]);
		let challenge = "expired_challenge";

		// Store challenge with 0 second timeout (immediately expired)
		storage.store_challenge(&omni_account, challenge, 0).unwrap();

		// Wait at least 1 second to ensure expiration (time precision is in seconds)
		std::thread::sleep(std::time::Duration::from_secs(1));

		// Should return ChallengeExpired (not ChallengeNotFound) because cleanup
		// only removes challenges expired >24h ago
		let result = storage.verify_and_consume_challenge(challenge, &omni_account);
		assert_eq!(result, Err(PasskeyChallengeError::ChallengeExpired));

		// Cleanup
		remove_test_storage(&db_path);
	}

	#[test]
	fn test_invalid_account() {
		let (storage, db_path) = create_test_storage();
		let omni_account1 = AccountId::from([3u8; 32]);
		let omni_account2 = AccountId::from([4u8; 32]);
		let challenge = "account_mismatch_challenge";

		// Store challenge for account1
		storage.store_challenge(&omni_account1, challenge, 60).unwrap();

		// Try to verify with account2 (should fail)
		let result = storage.verify_and_consume_challenge(challenge, &omni_account2);
		assert_eq!(result, Err(PasskeyChallengeError::InvalidChallenge));

		// Original challenge should still exist for account1
		assert!(storage.is_challenge_valid(challenge, &omni_account1).unwrap());

		// Cleanup
		remove_test_storage(&db_path);
	}

	#[test]
	fn test_challenge_not_found() {
		let (storage, db_path) = create_test_storage();
		let omni_account = AccountId::from([5u8; 32]);

		// Try to verify non-existent challenge
		let result = storage.verify_and_consume_challenge("nonexistent", &omni_account);
		assert_eq!(result, Err(PasskeyChallengeError::ChallengeNotFound));

		// Cleanup
		remove_test_storage(&db_path);
	}

	#[test]
	fn test_cleanup_no_expired_challenges() {
		let (storage, db_path) = create_test_storage();
		let omni_account = AccountId::from([6u8; 32]);

		// Store some valid challenges (not expired)
		storage.store_challenge(&omni_account, "challenge1", 60).unwrap();
		storage.store_challenge(&omni_account, "challenge2", 60).unwrap();
		storage.store_challenge(&omni_account, "challenge3", 60).unwrap();

		// Run cleanup - should clean 0 challenges (none expired >24h)
		let cleaned = storage.cleanup_expired24h_challenges().unwrap();
		assert_eq!(cleaned, 0);

		// All challenges should still be valid
		assert!(storage.is_challenge_valid("challenge1", &omni_account).unwrap());
		assert!(storage.is_challenge_valid("challenge2", &omni_account).unwrap());
		assert!(storage.is_challenge_valid("challenge3", &omni_account).unwrap());

		// Cleanup
		remove_test_storage(&db_path);
	}

	#[test]
	fn test_cleanup_all_expired_challenges() {
		let (storage, db_path) = create_test_storage();
		let omni_account = AccountId::from([7u8; 32]);

		// Store challenges with 0 second timeout (immediately expired)
		storage.store_challenge(&omni_account, "expired1", 0).unwrap();
		storage.store_challenge(&omni_account, "expired2", 0).unwrap();
		storage.store_challenge(&omni_account, "expired3", 0).unwrap();

		// Wait at least 1 second to ensure expiration
		std::thread::sleep(std::time::Duration::from_secs(1));

		// Run cleanup - should clean 0 (challenges expired <24h ago, still in grace period)
		let cleaned = storage.cleanup_expired24h_challenges().unwrap();
		assert_eq!(cleaned, 0);

		// Challenges are expired but still exist (within 24h grace period)
		// They will return ChallengeExpired when verified, not ChallengeNotFound
		let result1 = storage.verify_and_consume_challenge("expired1", &omni_account);
		assert_eq!(result1, Err(PasskeyChallengeError::ChallengeExpired));

		// Cleanup
		remove_test_storage(&db_path);
	}

	#[test]
	fn test_cleanup_mixed_expired_and_valid() {
		let (storage, db_path) = create_test_storage();
		let omni_account = AccountId::from([8u8; 32]);

		// Store mix of expired and valid challenges
		storage.store_challenge(&omni_account, "expired1", 0).unwrap();
		storage.store_challenge(&omni_account, "valid1", 60).unwrap();
		storage.store_challenge(&omni_account, "expired2", 0).unwrap();
		storage.store_challenge(&omni_account, "valid2", 60).unwrap();

		// Wait at least 1 second to ensure expiration
		std::thread::sleep(std::time::Duration::from_secs(1));

		// Run cleanup - should clean 0 (expired challenges still in 24h grace period)
		let cleaned = storage.cleanup_expired24h_challenges().unwrap();
		assert_eq!(cleaned, 0);

		// Valid challenges should still exist
		assert!(storage.is_challenge_valid("valid1", &omni_account).unwrap());
		assert!(storage.is_challenge_valid("valid2", &omni_account).unwrap());

		// Expired challenges still exist (in grace period) but return ChallengeExpired
		let result1 = storage.verify_and_consume_challenge("expired1", &omni_account);
		assert_eq!(result1, Err(PasskeyChallengeError::ChallengeExpired));

		// Cleanup
		remove_test_storage(&db_path);
	}

	#[test]
	fn test_cleanup_multiple_accounts() {
		let (storage, db_path) = create_test_storage();
		let account1 = AccountId::from([9u8; 32]);
		let account2 = AccountId::from([10u8; 32]);

		// Store expired challenges for different accounts
		storage.store_challenge(&account1, "account1_expired", 0).unwrap();
		storage.store_challenge(&account2, "account2_expired", 0).unwrap();
		storage.store_challenge(&account1, "account1_valid", 60).unwrap();

		// Wait at least 1 second to ensure expiration
		std::thread::sleep(std::time::Duration::from_secs(1));

		// Cleanup should clean 0 (expired challenges still in 24h grace period)
		let cleaned = storage.cleanup_expired24h_challenges().unwrap();
		assert_eq!(cleaned, 0);

		// Valid challenge should still exist
		assert!(storage.is_challenge_valid("account1_valid", &account1).unwrap());

		// Expired challenges return ChallengeExpired (not removed yet)
		let result1 = storage.verify_and_consume_challenge("account1_expired", &account1);
		assert_eq!(result1, Err(PasskeyChallengeError::ChallengeExpired));

		// Cleanup
		remove_test_storage(&db_path);
	}

	#[test]
	fn test_cleanup_idempotent() {
		let (storage, db_path) = create_test_storage();
		let omni_account = AccountId::from([11u8; 32]);

		// Store expired challenge
		storage.store_challenge(&omni_account, "expired", 0).unwrap();
		// Wait at least 1 second to ensure expiration
		std::thread::sleep(std::time::Duration::from_secs(1));

		// Cleanup won't remove challenges expired <24h ago
		let cleaned1 = storage.cleanup_expired24h_challenges().unwrap();
		assert_eq!(cleaned1, 0);

		// Second cleanup should also find nothing
		let cleaned2 = storage.cleanup_expired24h_challenges().unwrap();
		assert_eq!(cleaned2, 0);

		// Idempotent - still nothing
		let cleaned3 = storage.cleanup_expired24h_challenges().unwrap();
		assert_eq!(cleaned3, 0);

		// Cleanup
		remove_test_storage(&db_path);
	}

	#[test]
	fn test_cleanup_empty_storage() {
		let (storage, db_path) = create_test_storage();

		// Cleanup on empty storage should work fine
		let cleaned = storage.cleanup_expired24h_challenges().unwrap();
		assert_eq!(cleaned, 0);

		// Cleanup
		remove_test_storage(&db_path);
	}
}
