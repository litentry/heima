use ring::rand::{SecureRandom, SystemRandom};
use sha2::{Digest, Sha256};

/// Generate a pool commitment: SHA256(invoice_id || amount_le || secret)
///
/// The commitment is deposited on-chain. It binds the invoice ID and amount
/// to a TEE-held secret, without revealing either value publicly.
pub fn generate_pool_commitment(invoice_id: &str, amount: u128, secret: &[u8; 32]) -> [u8; 32] {
	let mut h = Sha256::new();
	h.update(invoice_id.as_bytes());
	h.update(amount.to_le_bytes());
	h.update(secret);
	h.finalize().into()
}

/// Generate a withdrawal nullifier: SHA256(secret || leaf_index_le)
///
/// The nullifier is submitted on withdrawal to prevent double-spending.
/// Only the TEE holding the secret can compute it.
pub fn generate_nullifier(secret: &[u8; 32], leaf_index: u32) -> [u8; 32] {
	let mut h = Sha256::new();
	h.update(secret);
	h.update(leaf_index.to_le_bytes());
	h.finalize().into()
}

/// Generate a cryptographically random 32-byte secret using the OS RNG.
pub fn generate_secret() -> Result<[u8; 32], String> {
	let rng = SystemRandom::new();
	let mut secret = [0u8; 32];
	rng.fill(&mut secret).map_err(|e| format!("RNG failed: {}", e))?;
	Ok(secret)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_commitment_is_deterministic() {
		let secret = [1u8; 32];
		let c1 = generate_pool_commitment("inv_abc", 1_000_000, &secret);
		let c2 = generate_pool_commitment("inv_abc", 1_000_000, &secret);
		assert_eq!(c1, c2);
	}

	#[test]
	fn test_commitment_differs_on_different_inputs() {
		let secret = [1u8; 32];
		let c1 = generate_pool_commitment("inv_abc", 1_000_000, &secret);
		let c2 = generate_pool_commitment("inv_abc", 2_000_000, &secret);
		let c3 = generate_pool_commitment("inv_xyz", 1_000_000, &secret);
		let c4 = generate_pool_commitment("inv_abc", 1_000_000, &[2u8; 32]);
		assert_ne!(c1, c2);
		assert_ne!(c1, c3);
		assert_ne!(c1, c4);
	}

	#[test]
	fn test_nullifier_is_deterministic() {
		let secret = [42u8; 32];
		let n1 = generate_nullifier(&secret, 5);
		let n2 = generate_nullifier(&secret, 5);
		assert_eq!(n1, n2);
	}

	#[test]
	fn test_nullifier_differs_on_different_leaf_index() {
		let secret = [42u8; 32];
		let n1 = generate_nullifier(&secret, 0);
		let n2 = generate_nullifier(&secret, 1);
		assert_ne!(n1, n2);
	}

	#[test]
	fn test_generate_secret_is_random() {
		let s1 = generate_secret().unwrap();
		let s2 = generate_secret().unwrap();
		// Collision probability negligible; statistically impossible in practice
		assert_ne!(s1, s2);
	}

	#[test]
	fn test_commitment_and_nullifier_are_different() {
		let secret = [1u8; 32];
		let commitment = generate_pool_commitment("inv_abc", 1_000_000, &secret);
		let nullifier = generate_nullifier(&secret, 0);
		assert_ne!(commitment, nullifier);
	}
}
