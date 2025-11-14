//! Integration tests for complete passkey flows
//!
//! These tests verify the end-to-end functionality of passkey operations
//! including registration, authentication, challenge management, and cleanup.

use oe_primitives::AccountId;
use oe_storage::{
	PasskeyChallengeError, PasskeyChallengeStorage, PasskeyError, PasskeyStorage, StorageDB,
};
use std::sync::Arc;

/// Helper to create a test storage database
fn create_test_storage() -> (Arc<StorageDB>, String) {
	use std::sync::atomic::{AtomicUsize, Ordering};
	use std::time::SystemTime;
	static COUNTER: AtomicUsize = AtomicUsize::new(0);
	// Use timestamp + counter for unique database paths
	let timestamp = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros();
	let count = COUNTER.fetch_add(1, Ordering::SeqCst);
	let db_path = format!("test_passkey_integration_{}_{}", timestamp, count);
	(Arc::new(StorageDB::open_default(&db_path).unwrap()), db_path)
}

/// Helper to remove test storage database
fn remove_test_storage(db_path: &str) {
	use std::fs;
	use std::path::Path;
	if Path::new(db_path).exists() {
		fs::remove_dir_all(db_path).unwrap();
	}
}

/// Helper to create a test omni account
fn create_test_account(id: u8) -> AccountId {
	AccountId::from([id; 32])
}

#[test]
fn test_complete_passkey_registration_flow() {
	// This test simulates the complete passkey registration workflow:
	// 1. Request challenge
	// 2. Store passkey
	// 3. Verify passkey is stored correctly

	let (storage_db, db_path) = create_test_storage();
	let challenge_storage = PasskeyChallengeStorage::new(storage_db.clone());
	let passkey_storage = PasskeyStorage::new(storage_db.clone());

	let omni_account = create_test_account(1);

	// Step 1: Generate and store challenge (simulates omni_requestPasskeyChallenge)
	let challenge = "test_challenge_reg";
	let timeout = 300; // 5 minutes
	challenge_storage
		.store_challenge(&omni_account, challenge, timeout)
		.expect("Failed to store challenge");

	// Step 2: Store the passkey (simulates omni_attachPasskey after attestation verification)
	let credential_id = "test_credential_123";
	let pubkey = b"test_pubkey_bytes_for_registration";

	passkey_storage
		.add_passkey(&omni_account, credential_id, pubkey, None)
		.expect("Failed to add passkey");

	// Step 3: Verify the passkey was stored correctly
	let stored_passkey = passkey_storage
		.get_passkey(&omni_account, credential_id)
		.expect("Failed to get passkey")
		.expect("Passkey not found");

	assert_eq!(stored_passkey.credential_id, credential_id);
	assert_eq!(stored_passkey.pubkey, pubkey.to_vec());

	// Cleanup
	remove_test_storage(&db_path);
}

#[test]
fn test_complete_passkey_authentication_flow() {
	// This test simulates the complete passkey authentication workflow:
	// 1. Register a passkey
	// 2. Request challenge for authentication
	// 3. Verify and consume challenge

	let (storage_db, db_path) = create_test_storage();
	let challenge_storage = PasskeyChallengeStorage::new(storage_db.clone());
	let passkey_storage = PasskeyStorage::new(storage_db.clone());

	let omni_account = create_test_account(2);

	// Step 1: Register a passkey
	let credential_id = "test_credential_auth";
	let pubkey = b"test_pubkey_bytes_for_auth";

	passkey_storage
		.add_passkey(&omni_account, credential_id, pubkey, None)
		.expect("Failed to add passkey");

	// Step 2: Generate authentication challenge
	let challenge = "test_challenge_auth";
	challenge_storage
		.store_challenge(&omni_account, challenge, 300)
		.expect("Failed to store challenge");

	// Step 3: Verify and consume challenge (simulates authentication)
	challenge_storage
		.verify_and_consume_challenge(challenge, &omni_account)
		.expect("Failed to verify challenge");

	// Verify challenge was consumed (should fail on second attempt)
	let result = challenge_storage.verify_and_consume_challenge(challenge, &omni_account);
	assert!(matches!(result, Err(PasskeyChallengeError::ChallengeNotFound)));

	// Cleanup
	remove_test_storage(&db_path);
}

#[test]
fn test_passkey_challenge_expired() {
	// This test verifies that expired challenges return ChallengeExpired error

	let (storage_db, db_path) = create_test_storage();
	let challenge_storage = PasskeyChallengeStorage::new(storage_db.clone());

	let omni_account = create_test_account(5);

	// Create challenge with immediate expiration
	challenge_storage
		.store_challenge(&omni_account, "expired", 0)
		.expect("Failed to store challenge");

	// Wait for expiration
	std::thread::sleep(std::time::Duration::from_secs(1));

	// Verify we get ChallengeExpired error (not ChallengeNotFound)
	let result = challenge_storage.verify_and_consume_challenge("expired", &omni_account);
	assert!(matches!(result, Err(PasskeyChallengeError::ChallengeExpired)));

	// Cleanup
	remove_test_storage(&db_path);
}

#[test]
fn test_multiple_passkeys_per_account() {
	// This test verifies handling of multiple passkeys for the same account

	let (storage_db, db_path) = create_test_storage();
	let passkey_storage = PasskeyStorage::new(storage_db.clone());

	let omni_account = create_test_account(6);

	// Register multiple passkeys (e.g., phone and laptop)
	let phone_credential = "phone_credential";
	let phone_pubkey = b"phone_pubkey_bytes";

	let laptop_credential = "laptop_credential";
	let laptop_pubkey = b"laptop_pubkey_bytes";

	passkey_storage
		.add_passkey(&omni_account, phone_credential, phone_pubkey, None)
		.expect("Failed to add phone passkey");

	passkey_storage
		.add_passkey(&omni_account, laptop_credential, laptop_pubkey, None)
		.expect("Failed to add laptop passkey");

	// Verify both passkeys exist independently
	let phone = passkey_storage
		.get_passkey(&omni_account, phone_credential)
		.expect("Failed to get phone passkey")
		.expect("Phone passkey not found");

	let laptop = passkey_storage
		.get_passkey(&omni_account, laptop_credential)
		.expect("Failed to get laptop passkey")
		.expect("Laptop passkey not found");

	assert_eq!(phone.pubkey, phone_pubkey.to_vec());
	assert_eq!(laptop.pubkey, laptop_pubkey.to_vec());

	// Cleanup
	remove_test_storage(&db_path);
}

#[test]
fn test_passkey_removal_flow() {
	// This test verifies the complete passkey removal workflow

	let (storage_db, db_path) = create_test_storage();
	let passkey_storage = PasskeyStorage::new(storage_db.clone());

	let omni_account = create_test_account(7);

	// Register a passkey
	let credential_id = "test_credential_remove";
	let pubkey = b"test_pubkey_remove";

	passkey_storage
		.add_passkey(&omni_account, credential_id, pubkey, None)
		.expect("Failed to add passkey");

	// Verify passkey exists
	assert!(passkey_storage.exists_passkey(&omni_account, credential_id));

	// Remove the passkey
	passkey_storage
		.remove_passkey(&omni_account, credential_id)
		.expect("Failed to remove passkey");

	// Verify passkey no longer exists
	assert!(!passkey_storage.exists_passkey(&omni_account, credential_id));

	// Verify get_passkey returns None
	let result = passkey_storage
		.get_passkey(&omni_account, credential_id)
		.expect("Failed to query passkey");

	assert!(result.is_none());

	// Cleanup
	remove_test_storage(&db_path);
}

#[test]
fn test_challenge_account_mismatch() {
	// This test verifies that challenges are properly scoped to specific accounts

	let (storage_db, db_path) = create_test_storage();
	let challenge_storage = PasskeyChallengeStorage::new(storage_db.clone());

	let account1 = create_test_account(8);
	let account2 = create_test_account(9);

	// Store challenge for account1
	challenge_storage
		.store_challenge(&account1, "test_challenge", 300)
		.expect("Failed to store challenge");

	// Try to verify with account2 (should fail)
	let result = challenge_storage.verify_and_consume_challenge("test_challenge", &account2);

	assert!(matches!(result, Err(PasskeyChallengeError::InvalidChallenge)));

	// Verify challenge still exists for account1
	challenge_storage
		.verify_and_consume_challenge("test_challenge", &account1)
		.expect("Challenge should still be valid for account1");

	// Cleanup
	remove_test_storage(&db_path);
}

#[test]
fn test_passkey_duplicate_prevention() {
	// This test verifies that duplicate passkeys are properly prevented

	let (storage_db, db_path) = create_test_storage();
	let passkey_storage = PasskeyStorage::new(storage_db.clone());

	let omni_account = create_test_account(10);

	let credential_id = "test_credential_dup";
	let pubkey = b"test_pubkey_dup";

	// Add passkey first time (should succeed)
	passkey_storage
		.add_passkey(&omni_account, credential_id, pubkey, None)
		.expect("Failed to add passkey");

	// Try to add the same passkey again (should fail)
	let duplicate_result = passkey_storage.add_passkey(&omni_account, credential_id, pubkey, None);

	assert!(matches!(duplicate_result, Err(PasskeyError::DuplicatePasskey)));

	// Cleanup
	remove_test_storage(&db_path);
}

#[test]
fn test_concurrent_authentication_sessions() {
	// This test verifies that multiple authentication sessions can be managed simultaneously

	let (storage_db, db_path) = create_test_storage();
	let challenge_storage = PasskeyChallengeStorage::new(storage_db.clone());
	let passkey_storage = PasskeyStorage::new(storage_db.clone());

	let account1 = create_test_account(12);
	let account2 = create_test_account(13);

	// Register passkeys for both accounts
	passkey_storage
		.add_passkey(&account1, "cred1", b"pubkey1", None)
		.expect("Failed to add passkey for account1");

	passkey_storage
		.add_passkey(&account2, "cred2", b"pubkey2", None)
		.expect("Failed to add passkey for account2");

	// Create challenges for both accounts
	challenge_storage
		.store_challenge(&account1, "challenge1", 300)
		.expect("Failed to store challenge1");

	challenge_storage
		.store_challenge(&account2, "challenge2", 300)
		.expect("Failed to store challenge2");

	// Verify challenges independently
	challenge_storage
		.verify_and_consume_challenge("challenge1", &account1)
		.expect("Failed to verify challenge1");

	challenge_storage
		.verify_and_consume_challenge("challenge2", &account2)
		.expect("Failed to verify challenge2");

	// Verify passkeys exist
	let pk1 = passkey_storage.get_passkey(&account1, "cred1").unwrap().unwrap();
	let pk2 = passkey_storage.get_passkey(&account2, "cred2").unwrap().unwrap();

	assert_eq!(pk1.pubkey, b"pubkey1");
	assert_eq!(pk2.pubkey, b"pubkey2");

	// Cleanup
	remove_test_storage(&db_path);
}
