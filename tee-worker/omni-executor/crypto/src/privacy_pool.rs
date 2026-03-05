use ark_bn254::Bn254;
use ark_circom::{CircomBuilder, CircomConfig};
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

	let circom = builder.build().map_err(|e| format!("witness: {e}"))?;

	let zkey_file = std::fs::File::open(zkey_path).map_err(|e| format!("open zkey: {e}"))?;
	let (pk, _matrices) = ark_circom::read_zkey(&mut std::io::BufReader::new(zkey_file))
		.map_err(|e| format!("read zkey: {e}"))?;

	let pvk = prepare_verifying_key(&pk.vk);
	let mut rng = thread_rng();

	let proof = Groth16::<Bn254>::create_random_proof_with_reduction(circom, &pk, &mut rng)
		.map_err(|e| format!("prove: {e}"))?;

	// Sanity check locally before returning
	let pub_inputs: Vec<ark_bn254::Fr> = [
		biguint_from_bytes32(root),
		biguint_from_bytes32(&nullifier),
		biguint_from_bytes32(&commitment),
	]
	.into_iter()
	.map(fr_from_biguint)
	.collect();

	let valid = Groth16::<Bn254>::verify_proof(&pvk, &proof, &pub_inputs)
		.map_err(|e| format!("local verify: {e}"))?;
	if !valid {
		return Err("Proof failed local verification".into());
	}

	let proof_bytes = encode_proof_abi(&proof)?;
	Ok(WithdrawalProof { proof_bytes, pub_signals: [*root, nullifier, commitment] })
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
