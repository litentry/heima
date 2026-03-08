use ark_bn254::Bn254;
use ark_circom::{CircomBuilder, CircomConfig, CircomReduction};
use ark_ff::{BigInteger, PrimeField};
use ark_groth16::{prepare_verifying_key, Groth16};
use ark_std::rand::thread_rng;
use ethers::abi::{encode, Token};
use light_poseidon::{Poseidon, PoseidonHasher};
use num_bigint::BigUint;
use ring::rand::{SecureRandom, SystemRandom};

// ─── Field helpers ────────────────────────────────────────────────────────────

const BN254_Q: &str =
	"21888242871839275222246405745257275088548364400416034343698204186575808495617";

fn bn254_q() -> BigUint {
	BN254_Q.parse().expect("static")
}

fn fr_from_biguint(n: BigUint) -> ark_bn254::Fr {
	ark_bn254::Fr::from(n % bn254_q())
}

fn fr_to_bytes32(f: &ark_bn254::Fr) -> [u8; 32] {
	let bytes = f.into_bigint().to_bytes_be();
	let mut out = [0u8; 32];
	let len = bytes.len().min(32);
	out[32 - len..].copy_from_slice(&bytes[bytes.len() - len..]);
	out
}

fn biguint_from_bytes32(b: &[u8; 32]) -> BigUint {
	BigUint::from_bytes_be(b)
}

/// Combine the 32-byte secret into a single BN254 field element the same way
/// the circom circuit does: `secret = secret_lo + secret_hi * 2^128`
/// where secret_lo = bytes[16..32] as u128 BE, secret_hi = bytes[0..16] as u128 BE.
fn combine_secret(secret: &[u8; 32]) -> BigUint {
	let hi = BigUint::from(u128::from_be_bytes(secret[0..16].try_into().unwrap()));
	let lo = BigUint::from(u128::from_be_bytes(secret[16..32].try_into().unwrap()));
	(lo + hi * (BigUint::from(1u128) << 128)) % bn254_q()
}

// ─── Core crypto functions ────────────────────────────────────────────────────

/// Generate a cryptographically random 32-byte secret using the OS RNG.
pub fn generate_secret() -> Result<[u8; 32], String> {
	let rng = SystemRandom::new();
	let mut secret = [0u8; 32];
	rng.fill(&mut secret).map_err(|e| format!("RNG failed: {}", e))?;
	Ok(secret)
}

/// Compute `Poseidon([secret_combined, amount])`, matching the circom circuit.
/// Returns a 32-byte big-endian field element.
pub fn generate_pool_commitment(secret: &[u8; 32], amount: u128) -> Result<[u8; 32], String> {
	let s = fr_from_biguint(combine_secret(secret));
	let a = fr_from_biguint(BigUint::from(amount));

	let mut poseidon =
		Poseidon::<ark_bn254::Fr>::new_circom(2).map_err(|e| format!("Poseidon init: {e}"))?;
	let hash = poseidon.hash(&[s, a]).map_err(|e| format!("Poseidon hash: {e}"))?;
	Ok(fr_to_bytes32(&hash))
}

/// Compute `Poseidon([secret_combined, leaf_index])`, matching the circom circuit.
/// Returns a 32-byte big-endian field element.
pub fn generate_nullifier(secret: &[u8; 32], leaf_index: u32) -> Result<[u8; 32], String> {
	let s = fr_from_biguint(combine_secret(secret));
	let li = fr_from_biguint(BigUint::from(leaf_index));

	let mut poseidon =
		Poseidon::<ark_bn254::Fr>::new_circom(2).map_err(|e| format!("Poseidon init: {e}"))?;
	let hash = poseidon.hash(&[s, li]).map_err(|e| format!("Poseidon hash: {e}"))?;
	Ok(fr_to_bytes32(&hash))
}

// ─── ZK proof generation ──────────────────────────────────────────────────────

/// Groth16 withdrawal proof ready to submit on-chain.
pub struct WithdrawalProof {
	/// ABI-encoded (uint256[2] pA, uint256[2][2] pB, uint256[2] pC) — 320 bytes
	pub proof_bytes: Vec<u8>,
	/// [root, nullifier, commitment] as 32-byte big-endian field elements
	pub pub_signals: [[u8; 32]; 3],
}

/// Generate a Groth16 ZK proof for a pool withdrawal.
///
/// # Arguments
/// * `secret` — 32-byte TEE-generated secret
/// * `leaf_index` — leaf position in the Merkle tree
/// * `amount` — token amount in raw units (same value used at deposit)
/// * `path_elements` — 20 Merkle sibling hashes (big-endian field elements from on-chain)
/// * `root` — current tree root from on-chain (big-endian field element)
/// * `wasm_path` — path to `withdraw_js/withdraw.wasm`
/// * `zkey_path` — path to `withdraw_final.zkey`; `.r1cs` file must be alongside it
pub fn generate_withdrawal_proof(
	secret: &[u8; 32],
	leaf_index: u32,
	amount: u128,
	path_elements: &[[u8; 32]; 20],
	root: &[u8; 32],
	wasm_path: &str,
	zkey_path: &str,
) -> Result<WithdrawalProof, String> {
	let commitment = generate_pool_commitment(secret, amount)?;
	let nullifier = generate_nullifier(secret, leaf_index)?;

	// r1cs file sits next to the wasm (circuits/build/withdraw.r1cs)
	let r1cs_path = wasm_path
		.replace("withdraw_js/withdraw.wasm", "withdraw.r1cs")
		.replace("withdraw.wasm", "../withdraw.r1cs");

	let cfg = CircomConfig::<ark_bn254::Fr>::new(wasm_path, &r1cs_path)
		.map_err(|e| format!("CircomConfig: {e}"))?;

	tracing::debug!("CircomConfig loaded, wasm={} r1cs={}", wasm_path, r1cs_path);

	let mut builder = CircomBuilder::new(cfg);

	// Secret halves (lo = bytes[16..32], hi = bytes[0..16])
	let secret_hi = u128::from_be_bytes(secret[0..16].try_into().unwrap());
	let secret_lo = u128::from_be_bytes(secret[16..32].try_into().unwrap());
	builder.push_input("secret_lo", BigUint::from(secret_lo));
	builder.push_input("secret_hi", BigUint::from(secret_hi));
	builder.push_input("leaf_index", BigUint::from(leaf_index));
	builder.push_input("amount", BigUint::from(amount));

	for elem in path_elements.iter() {
		builder.push_input("path_elements", biguint_from_bytes32(elem));
	}

	// Public inputs
	builder.push_input("root", biguint_from_bytes32(root));
	builder.push_input("nullifier", biguint_from_bytes32(&nullifier));
	builder.push_input("commitment", biguint_from_bytes32(&commitment));

	tracing::debug!(
		"Building witness: secret_lo={}, secret_hi={}, leaf_index={}, amount={}, path_elements[0]={}",
		u128::from_be_bytes(secret[16..32].try_into().unwrap()),
		u128::from_be_bytes(secret[0..16].try_into().unwrap()),
		leaf_index,
		amount,
		biguint_from_bytes32(&path_elements[0]),
	);

	let circom = builder.build().map_err(|e| format!("witness: {e}"))?;

	// Extract public inputs from the witness BEFORE consuming the circuit.
	// Using get_public_inputs() guarantees the values match what the prover uses,
	// avoiding any mismatch between manually-recomputed values and the witness.
	let pub_inputs = circom.get_public_inputs().ok_or("No witness in circom circuit")?;

	let zkey_file = std::fs::File::open(zkey_path).map_err(|e| format!("open zkey: {e}"))?;
	let (pk, _matrices) = ark_circom::read_zkey(&mut std::io::BufReader::new(zkey_file))
		.map_err(|e| format!("read zkey: {e}"))?;

	let pvk = prepare_verifying_key(&pk.vk);
	let mut rng = thread_rng();

	tracing::debug!(
		"Public inputs from witness: root={:?}, nullifier={:?}, commitment={:?}",
		pub_inputs.get(0).map(|f| fr_to_bytes32(f)),
		pub_inputs.get(1).map(|f| fr_to_bytes32(f)),
		pub_inputs.get(2).map(|f| fr_to_bytes32(f)),
	);

	let proof = Groth16::<Bn254, CircomReduction>::create_random_proof_with_reduction(
		circom, &pk, &mut rng,
	)
	.map_err(|e| format!("prove: {e}"))?;

	// Sanity check: verify the proof against the public inputs from the witness.
	let valid = Groth16::<Bn254, CircomReduction>::verify_proof(&pvk, &proof, &pub_inputs)
		.map_err(|e| format!("local verify: {e}"))?;
	if !valid {
		return Err("Proof failed local verification".into());
	}

	// Convert public inputs back to bytes for pub_signals.
	// Order matches R1CS wire ordering: [root, nullifier, commitment].
	let fr_to_bytes = |f: &ark_bn254::Fr| -> [u8; 32] { fr_to_bytes32(f) };
	let pub_root = fr_to_bytes(&pub_inputs[0]);
	let pub_nullifier = fr_to_bytes(&pub_inputs[1]);
	let pub_commitment = fr_to_bytes(&pub_inputs[2]);

	let proof_bytes = encode_proof_abi(&proof)?;
	Ok(WithdrawalProof { proof_bytes, pub_signals: [pub_root, pub_nullifier, pub_commitment] })
}

fn encode_proof_abi(proof: &ark_groth16::Proof<Bn254>) -> Result<Vec<u8>, String> {
	fn g1(p: &ark_bn254::G1Affine) -> Vec<Token> {
		let x = p.x.into_bigint().to_bytes_be();
		let y = p.y.into_bigint().to_bytes_be();
		vec![
			Token::Uint(ethers::types::U256::from_big_endian(&x)),
			Token::Uint(ethers::types::U256::from_big_endian(&y)),
		]
	}

	fn g2(p: &ark_bn254::G2Affine) -> Vec<Token> {
		// snarkjs Groth16 verifier expects [x.c1, x.c0] and [y.c1, y.c0]
		let xc0 = p.x.c0.into_bigint().to_bytes_be();
		let xc1 = p.x.c1.into_bigint().to_bytes_be();
		let yc0 = p.y.c0.into_bigint().to_bytes_be();
		let yc1 = p.y.c1.into_bigint().to_bytes_be();
		vec![
			Token::FixedArray(vec![
				Token::Uint(ethers::types::U256::from_big_endian(&xc1)),
				Token::Uint(ethers::types::U256::from_big_endian(&xc0)),
			]),
			Token::FixedArray(vec![
				Token::Uint(ethers::types::U256::from_big_endian(&yc1)),
				Token::Uint(ethers::types::U256::from_big_endian(&yc0)),
			]),
		]
	}

	Ok(encode(&[
		Token::FixedArray(g1(&proof.a)),
		Token::FixedArray(g2(&proof.b)),
		Token::FixedArray(g1(&proof.c)),
	]))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_commitment_is_deterministic() {
		let secret = [1u8; 32];
		let c1 = generate_pool_commitment(&secret, 1_000_000).unwrap();
		let c2 = generate_pool_commitment(&secret, 1_000_000).unwrap();
		assert_eq!(c1, c2);
	}

	#[test]
	fn test_commitment_differs_on_different_inputs() {
		let secret = [1u8; 32];
		let c1 = generate_pool_commitment(&secret, 1_000_000).unwrap();
		let c2 = generate_pool_commitment(&secret, 2_000_000).unwrap();
		let c4 = generate_pool_commitment(&[2u8; 32], 1_000_000).unwrap();
		assert_ne!(c1, c2);
		assert_ne!(c1, c4);
	}

	#[test]
	fn test_nullifier_is_deterministic() {
		let secret = [42u8; 32];
		let n1 = generate_nullifier(&secret, 5).unwrap();
		let n2 = generate_nullifier(&secret, 5).unwrap();
		assert_eq!(n1, n2);
	}

	#[test]
	fn test_nullifier_differs_on_different_leaf_index() {
		let secret = [42u8; 32];
		let n1 = generate_nullifier(&secret, 0).unwrap();
		let n2 = generate_nullifier(&secret, 1).unwrap();
		assert_ne!(n1, n2);
	}

	#[test]
	fn test_generate_secret_is_random() {
		let s1 = generate_secret().unwrap();
		let s2 = generate_secret().unwrap();
		assert_ne!(s1, s2);
	}

	#[test]
	fn test_commitment_and_nullifier_are_different() {
		let secret = [1u8; 32];
		let commitment = generate_pool_commitment(&secret, 1_000_000).unwrap();
		let nullifier = generate_nullifier(&secret, 0).unwrap();
		assert_ne!(commitment, nullifier);
	}
}

#[cfg(test)]
mod poseidon_compat_tests {
	use super::*;
	use ark_ff::{BigInteger, PrimeField};
	use light_poseidon::{Poseidon, PoseidonHasher};
	use num_bigint::BigUint;

	#[test]
	fn test_poseidon_zero_zero_matches_contract() {
		// PoseidonT3.hash([0,0]) on Arb Sepolia = 14744269619966411208579211824598458697587494354926760081771325075741142829156
		let expected: BigUint =
			"14744269619966411208579211824598458697587494354926760081771325075741142829156"
				.parse()
				.unwrap();
		let zero = ark_bn254::Fr::from(0u64);
		let mut pos = Poseidon::<ark_bn254::Fr>::new_circom(2).unwrap();
		let result = pos.hash(&[zero, zero]).unwrap();
		let computed = BigUint::from_bytes_be(&result.into_bigint().to_bytes_be());
		assert_eq!(computed, expected, "light_poseidon(0,0) mismatch with PoseidonT3");
	}
}

#[cfg(test)]
mod proof_tests {
	use super::*;

	/// Integration test: generate a real Groth16 proof for a known deposit.
	/// Requires the circuit artifacts to be present.
	#[test]
	#[ignore] // run with: cargo test -p oe-crypto -- proof_tests::test_real_proof --ignored --nocapture
	fn test_real_proof() {
		let wasm = std::env::var("OE_CIRCUIT_WASM_PATH").unwrap_or_else(|_| {
			"contracts/privacy-pool/circuits/build/withdraw_js/withdraw.wasm".to_string()
		});
		let zkey = std::env::var("OE_CIRCUIT_ZKEY_PATH").unwrap_or_else(|_| {
			"contracts/privacy-pool/circuits/build/withdraw_final.zkey".to_string()
		});

		// Use a fixed secret + known on-chain commitment
		let secret = [
			0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
			0x01, 0x01, // hi
			0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
			0x01, 0x01, // lo
		];
		let amount: u128 = 1_000_000;
		let leaf_index: u32 = 0;

		// Compute expected commitment and nullifier
		let commitment = generate_pool_commitment(&secret, amount).unwrap();
		let nullifier = generate_nullifier(&secret, leaf_index).unwrap();
		println!("commitment: {}", num_bigint::BigUint::from_bytes_be(&commitment));
		println!("nullifier:  {}", num_bigint::BigUint::from_bytes_be(&nullifier));

		// Build a Merkle root for this single deposit (leaf_index=0, all siblings are zero hashes)
		use ark_ff::{BigInteger, PrimeField};
		use light_poseidon::{Poseidon, PoseidonHasher};
		let mut path_elements = [[0u8; 32]; 20];
		let mut current = commitment;
		for i in 0..20u32 {
			let zero = {
				let mut h = [0u8; 32];
				for _ in 0..i {
					let fa = fr_from_biguint(num_bigint::BigUint::from_bytes_be(&h));
					let mut pos = Poseidon::<ark_bn254::Fr>::new_circom(2).unwrap();
					let hash = pos.hash(&[fa, fa]).unwrap();
					let bytes = hash.into_bigint().to_bytes_be();
					let len = bytes.len().min(32);
					h = [0u8; 32];
					h[32 - len..].copy_from_slice(&bytes[bytes.len() - len..]);
				}
				h
			};
			path_elements[i as usize] = zero;
			let fa = fr_from_biguint(num_bigint::BigUint::from_bytes_be(&current));
			let fb = fr_from_biguint(num_bigint::BigUint::from_bytes_be(&zero));
			let mut pos = Poseidon::<ark_bn254::Fr>::new_circom(2).unwrap();
			let hash = pos.hash(&[fa, fb]).unwrap();
			let bytes = hash.into_bigint().to_bytes_be();
			let len = bytes.len().min(32);
			let mut next = [0u8; 32];
			next[32 - len..].copy_from_slice(&bytes[bytes.len() - len..]);
			current = next;
		}
		let root = current;
		println!("root: {}", num_bigint::BigUint::from_bytes_be(&root));

		let result = generate_withdrawal_proof(
			&secret,
			leaf_index,
			amount,
			&path_elements,
			&root,
			&wasm,
			&zkey,
		);
		match &result {
			Err(e) => println!("PROOF ERROR: {}", e),
			Ok(p) => println!("Proof generated! {} bytes", p.proof_bytes.len()),
		}
		assert!(result.is_ok(), "proof generation failed: {:?}", result.err());
	}
}

#[cfg(test)]
mod merkle_tests {
	use super::*;
	use ark_ff::{BigInteger, PrimeField};
	use light_poseidon::{Poseidon, PoseidonHasher};
	use num_bigint::BigUint;

	fn poseidon2(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
		let fa = fr_from_biguint(BigUint::from_bytes_be(a));
		let fb = fr_from_biguint(BigUint::from_bytes_be(b));
		let mut pos = Poseidon::<ark_bn254::Fr>::new_circom(2).unwrap();
		let h = pos.hash(&[fa, fb]).unwrap();
		let bytes = h.into_bigint().to_bytes_be();
		let mut out = [0u8; 32];
		let len = bytes.len().min(32);
		out[32 - len..].copy_from_slice(&bytes[bytes.len() - len..]);
		out
	}

	fn zero_hash(level: u32) -> [u8; 32] {
		let mut h = [0u8; 32];
		for _ in 0..level {
			h = poseidon2(&h, &h);
		}
		h
	}

	#[test]
	fn test_merkle_root_for_single_deposit() {
		// On-chain commitment at leaf_index=0:
		// 469781679476332366041019967678925910311708526703837323809754038603352212159
		// On-chain root after deposit:
		// 6434775000210258688043227879212772558467458562877782451826202305888857397764

		let commitment_big: BigUint =
			"469781679476332366041019967678925910311708526703837323809754038603352212159"
				.parse()
				.unwrap();
		let expected_root: BigUint =
			"6434775000210258688043227879212772558467458562877782451826202305888857397764"
				.parse()
				.unwrap();

		let mut leaf_bytes = commitment_big.to_bytes_be();
		// pad to 32 bytes
		let mut commitment = [0u8; 32];
		commitment[32 - leaf_bytes.len()..].copy_from_slice(&leaf_bytes);

		// For leaf_index=0, all bits=0, path = zeros
		let mut current = commitment;
		for i in 0..20u32 {
			let zero = zero_hash(i);
			current = poseidon2(&current, &zero);
		}

		let computed_root = BigUint::from_bytes_be(&current);
		assert_eq!(computed_root, expected_root, "Computed root doesn't match on-chain root");
	}
}
