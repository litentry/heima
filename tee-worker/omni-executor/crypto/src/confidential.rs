/// Confidential invoice encryption utilities
/// Uses AES-256-GCM for encryption and Keccak256 for commitment scheme
use rand::RngCore;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use sha2::{Digest, Sha256};
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum ConfidentialError {
	EncryptionFailed(String),
	DecryptionFailed(String),
	InvalidCiphertext(String),
	InvalidPlaintext(String),
	RandomGenerationFailed(String),
}

impl fmt::Display for ConfidentialError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::EncryptionFailed(s) => write!(f, "Encryption failed: {}", s),
			Self::DecryptionFailed(s) => write!(f, "Decryption failed: {}", s),
			Self::InvalidCiphertext(s) => write!(f, "Invalid ciphertext: {}", s),
			Self::InvalidPlaintext(s) => write!(f, "Invalid plaintext: {}", s),
			Self::RandomGenerationFailed(s) => write!(f, "Random generation failed: {}", s),
		}
	}
}

impl StdError for ConfidentialError {}

/// Encrypt invoice amount using TEE key (AES-256-GCM)
///
/// # Arguments
/// * `amount` - The invoice amount in smallest units (e.g., USDC has 6 decimals)
/// * `tee_key` - 32-byte TEE encryption key
///
/// # Returns
/// * `Vec<u8>` - Encrypted data (nonce + ciphertext)
pub fn encrypt_amount(amount: u128, tee_key: &[u8; 32]) -> Result<Vec<u8>, ConfidentialError> {
	let unbound_key = UnboundKey::new(&AES_256_GCM, tee_key)
		.map_err(|e| ConfidentialError::EncryptionFailed(format!("Key init failed: {}", e)))?;
	let key = LessSafeKey::new(unbound_key);

	// Generate random nonce (12 bytes for GCM)
	let rng = SystemRandom::new();
	let mut nonce_bytes = [0u8; 12];
	rng.fill(&mut nonce_bytes)
		.map_err(|e| ConfidentialError::RandomGenerationFailed(format!("{}", e)))?;
	let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)
		.map_err(|_| ConfidentialError::EncryptionFailed("Nonce creation failed".to_string()))?;

	// Serialize amount as bytes (little-endian)
	let plaintext = amount.to_le_bytes();

	// Encrypt (in-place operation requires mutable data)
	let mut in_out = plaintext.to_vec();
	key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
		.map_err(|e| ConfidentialError::EncryptionFailed(format!("Seal failed: {}", e)))?;

	// Prepend nonce to ciphertext
	let mut result = nonce_bytes.to_vec();
	result.extend_from_slice(&in_out);

	Ok(result)
}

/// Decrypt invoice amount using TEE key (AES-256-GCM)
///
/// # Arguments
/// * `encrypted` - Encrypted data (nonce + ciphertext)
/// * `tee_key` - 32-byte TEE encryption key
///
/// # Returns
/// * `u128` - Decrypted amount
pub fn decrypt_amount(encrypted: &[u8], tee_key: &[u8; 32]) -> Result<u128, ConfidentialError> {
	if encrypted.len() < 12 {
		return Err(ConfidentialError::InvalidCiphertext(
			"Data too short (missing nonce)".to_string(),
		));
	}

	let unbound_key = UnboundKey::new(&AES_256_GCM, tee_key)
		.map_err(|e| ConfidentialError::DecryptionFailed(format!("Key init failed: {}", e)))?;
	let key = LessSafeKey::new(unbound_key);

	// Extract nonce (first 12 bytes)
	let nonce_bytes: [u8; 12] = encrypted[0..12]
		.try_into()
		.map_err(|_| ConfidentialError::InvalidCiphertext("Nonce extraction failed".to_string()))?;
	let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)
		.map_err(|_| ConfidentialError::DecryptionFailed("Nonce creation failed".to_string()))?;

	// Extract ciphertext (remaining bytes)
	let mut in_out = encrypted[12..].to_vec();

	// Decrypt
	let plaintext = key
		.open_in_place(nonce, Aad::empty(), &mut in_out)
		.map_err(|e| ConfidentialError::DecryptionFailed(format!("Open failed: {}", e)))?;

	// Parse amount
	let amount_bytes: [u8; 16] = plaintext.try_into().map_err(|_| {
		ConfidentialError::InvalidPlaintext(format!("Expected 16 bytes, got {}", plaintext.len()))
	})?;

	Ok(u128::from_le_bytes(amount_bytes))
}

/// Generate commitment hash using Keccak256
///
/// commitment = keccak256(invoice_id || amount)
///
/// # Arguments
/// * `invoice_id` - Unique invoice identifier
/// * `amount` - Invoice amount in smallest units
///
/// # Returns
/// * `[u8; 32]` - Commitment hash
pub fn generate_commitment(invoice_id: &str, amount: u128) -> [u8; 32] {
	let mut hasher = Sha256::new();
	hasher.update(invoice_id.as_bytes());
	hasher.update(&amount.to_le_bytes());
	hasher.finalize().into()
}

/// Verify commitment matches amount
///
/// # Arguments
/// * `commitment` - Commitment hash to verify
/// * `invoice_id` - Invoice identifier
/// * `amount` - Amount to verify
///
/// # Returns
/// * `bool` - True if commitment matches
pub fn verify_commitment(commitment: &[u8; 32], invoice_id: &str, amount: u128) -> bool {
	let computed = generate_commitment(invoice_id, amount);
	commitment == &computed
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_encrypt_decrypt_roundtrip() {
		let tee_key = [42u8; 32];
		let amount: u128 = 50_000_000_000; // 50,000 USDC (6 decimals)

		let encrypted = encrypt_amount(amount, &tee_key).expect("Encryption failed");
		assert!(encrypted.len() > 12); // At least nonce + some data

		let decrypted = decrypt_amount(&encrypted, &tee_key).expect("Decryption failed");
		assert_eq!(decrypted, amount);
	}

	#[test]
	fn test_decrypt_with_wrong_key() {
		let tee_key = [42u8; 32];
		let wrong_key = [99u8; 32];
		let amount: u128 = 50_000_000_000;

		let encrypted = encrypt_amount(amount, &tee_key).expect("Encryption failed");
		let result = decrypt_amount(&encrypted, &wrong_key);
		assert!(result.is_err());
	}

	#[test]
	fn test_decrypt_invalid_ciphertext() {
		let tee_key = [42u8; 32];
		let invalid = vec![1, 2, 3]; // Too short

		let result = decrypt_amount(&invalid, &tee_key);
		assert!(result.is_err());
	}

	#[test]
	fn test_commitment_generation() {
		let invoice_id = "inv_12345";
		let amount: u128 = 50_000_000_000;

		let commitment1 = generate_commitment(invoice_id, amount);
		let commitment2 = generate_commitment(invoice_id, amount);

		// Same input should produce same commitment
		assert_eq!(commitment1, commitment2);

		// Different amount should produce different commitment
		let commitment3 = generate_commitment(invoice_id, amount + 1);
		assert_ne!(commitment1, commitment3);
	}

	#[test]
	fn test_verify_commitment() {
		let invoice_id = "inv_12345";
		let amount: u128 = 50_000_000_000;

		let commitment = generate_commitment(invoice_id, amount);

		// Correct verification
		assert!(verify_commitment(&commitment, invoice_id, amount));

		// Wrong amount
		assert!(!verify_commitment(&commitment, invoice_id, amount + 1));

		// Wrong invoice_id
		assert!(!verify_commitment(&commitment, "inv_99999", amount));
	}

	#[test]
	fn test_different_nonces() {
		let tee_key = [42u8; 32];
		let amount: u128 = 50_000_000_000;

		let encrypted1 = encrypt_amount(amount, &tee_key).expect("Encryption failed");
		let encrypted2 = encrypt_amount(amount, &tee_key).expect("Encryption failed");

		// Same amount encrypted twice should produce different ciphertexts (different nonces)
		assert_ne!(encrypted1, encrypted2);

		// But both should decrypt to the same amount
		let decrypted1 = decrypt_amount(&encrypted1, &tee_key).expect("Decryption failed");
		let decrypted2 = decrypt_amount(&encrypted2, &tee_key).expect("Decryption failed");
		assert_eq!(decrypted1, amount);
		assert_eq!(decrypted2, amount);
	}
}
