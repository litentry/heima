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

use crate::types::{
	addPasskeySignerCall, addRootSignerCall, getNonceCall, getOwnerCall, removePasskeySignerCall,
	removeRootSignerCall, PasskeyPublicKey,
};
use crate::utils::build_call_transaction;
use alloy::primitives::{Address, FixedBytes, U256};
use alloy::rpc::types::TransactionRequest;
use alloy::sol_types::{SolCall, SolValue};
use ethereum_rpc::RpcProvider;
use std::sync::Arc;
use tracing::error;

pub struct OmniAccountClient<P: RpcProvider<Transaction = TransactionRequest>> {
	address: Address,
	rpc_client: Arc<P>,
}

impl<P: RpcProvider<Transaction = TransactionRequest>> OmniAccountClient<P> {
	pub fn new(address: Address, rpc_client: Arc<P>) -> Self {
		Self { address, rpc_client }
	}

	pub fn rpc_client(&self) -> Arc<P> {
		self.rpc_client.clone()
	}

	pub async fn get_nonce(&self) -> Result<U256, ()> {
		let call_data = getNonceCall {}.abi_encode();
		let tx = build_call_transaction(self.address, call_data);
		let result = self
			.rpc_client
			.call(tx)
			.await
			.map_err(|e| error!("Could not get nonce: {:?}", e))?;
		let nonce = U256::abi_decode(&result).map_err(|_| error!("Could not decode nonce"))?;
		Ok(nonce)
	}

	pub async fn add_root_signer(&self, root: Address) -> Result<(), ()> {
		let call_data = addRootSignerCall { root }.abi_encode();
		let tx = build_call_transaction(self.address, call_data);
		self.rpc_client.send_transaction(tx).await.map_err(|_| ())?;
		Ok(())
	}

	pub async fn remove_root_signer(&self, root: Address) -> Result<(), ()> {
		let call_data = removeRootSignerCall { root }.abi_encode();
		let tx = build_call_transaction(self.address, call_data);
		self.rpc_client.send_transaction(tx).await.map_err(|_| ())?;
		Ok(())
	}

	pub async fn get_owner(&self) -> Result<FixedBytes<32>, ()> {
		let call_data = getOwnerCall {}.abi_encode();
		let tx = build_call_transaction(self.address, call_data);
		let result = self
			.rpc_client
			.call(tx)
			.await
			.map_err(|e| error!("Could not get owner: {:?}", e))?;
		let owner =
			FixedBytes::<32>::abi_decode(&result).map_err(|_| error!("Could not decode owner"))?;
		Ok(owner)
	}

	pub async fn add_passkey_signer(&self, pk: PasskeyPublicKey) -> Result<(), ()> {
		let call_data = addPasskeySignerCall { pk }.abi_encode();
		let tx = build_call_transaction(self.address, call_data);
		self.rpc_client.send_transaction(tx).await.map_err(|_| ())?;
		Ok(())
	}

	pub async fn remove_passkey_signer(&self, pk: PasskeyPublicKey) -> Result<(), ()> {
		let call_data = removePasskeySignerCall { pk }.abi_encode();
		let tx = build_call_transaction(self.address, call_data);
		self.rpc_client.send_transaction(tx).await.map_err(|_| ())?;
		Ok(())
	}

	pub async fn add_passkey_signer_from_string(&self, pubkey_str: &str) -> Result<(), String> {
		let pk = parse_passkey_public_key(pubkey_str)?;
		self.add_passkey_signer(pk)
			.await
			.map_err(|_| "Failed to add passkey signer".to_string())
	}

	pub async fn remove_passkey_signer_from_string(&self, pubkey_str: &str) -> Result<(), String> {
		let pk = parse_passkey_public_key(pubkey_str)?;
		self.remove_passkey_signer(pk)
			.await
			.map_err(|_| "Failed to remove passkey signer".to_string())
	}
}

/// Parse a passkey public key from string format
///
/// WebAuthn passkeys use COSE (CBOR Object Signing and Encryption) format,
/// and this implementation now supports both COSE and raw EC formats.
///
/// # Current behavior:
/// 1. Attempts COSE parsing first (CBOR-encoded keys with proper validation)
/// 2. Falls back to raw EC formats for backward compatibility
/// 3. Supports both hex-prefixed and non-prefixed formats
///
/// # COSE support:
/// - ✅ Full CBOR parsing with `ciborium` dependency
/// - ✅ P-256 COSE key validation (kty=2, alg=-7, crv=1)
/// - ✅ Proper extraction of x and y coordinates
/// - ✅ Comprehensive error handling for invalid COSE keys
pub fn parse_passkey_public_key(pubkey_str: &str) -> Result<PasskeyPublicKey, String> {
	let pubkey_str = pubkey_str.strip_prefix("0x").unwrap_or(pubkey_str);

	// Try COSE parsing first (works for any length)
	match parse_cose_public_key(pubkey_str) {
		Ok(cose_key) => return Ok(cose_key),
		Err(cose_err) => {
			// If COSE parsing failed with a structural error, return it
			// Otherwise, fall back to raw EC format parsing
			if cose_err.contains("Invalid key type") ||
			   cose_err.contains("Invalid algorithm") ||
			   cose_err.contains("Invalid curve") ||
			   cose_err.contains("Failed to parse CBOR") {
				return Err(cose_err);
			}
			// Continue to raw EC parsing for other errors (like hex decode failures)
		}
	}

	// Fallback to raw EC formats for backwards compatibility
	// Handle different formats: compressed (66 chars) or uncompressed (130 chars)
	let (x_hex, y_hex) = match pubkey_str.len() {
		66 => {
			// Compressed format: 02/03 + 32 bytes x coordinate
			// We need to decompress to get both x and y
			return Err("Compressed public key format not supported yet. Use uncompressed format."
				.to_string());
		},
		128 => {
			// Uncompressed format without prefix: 32 bytes x + 32 bytes y
			(&pubkey_str[..64], &pubkey_str[64..])
		},
		130 => {
			// Uncompressed format with 04 prefix: 04 + 32 bytes x + 32 bytes y
			if !pubkey_str.starts_with("04") {
				return Err("Invalid uncompressed public key prefix".to_string());
			}
			(&pubkey_str[2..66], &pubkey_str[66..])
		},
		_ => return Err("Invalid public key length. Expected COSE format or raw EC format.".to_string()),
	};

	let x_bytes =
		alloy::hex::decode(x_hex).map_err(|_| "Invalid hex in x coordinate".to_string())?;
	let y_bytes =
		alloy::hex::decode(y_hex).map_err(|_| "Invalid hex in y coordinate".to_string())?;

	if x_bytes.len() != 32 || y_bytes.len() != 32 {
		return Err("Invalid coordinate length".to_string());
	}

	let x = FixedBytes::<32>::from_slice(&x_bytes);
	let y = FixedBytes::<32>::from_slice(&y_bytes);

	Ok(PasskeyPublicKey { x, y })
}

/// Parse a COSE-encoded public key (WebAuthn standard format)
/// COSE format is CBOR-encoded and contains key type, algorithm, and coordinates
///
/// # WebAuthn COSE Key Format
/// WebAuthn passkeys use COSE (RFC 8152) encoding for public keys. For P-256 keys,
/// the COSE key structure contains:
/// - kty (label 1): 2 (EC2 - Elliptic Curve Keys w/ x- and y-coordinate pair)
/// - alg (label 3): -7 (ES256 - ECDSA w/ SHA-256)
/// - crv (label -1): 1 (P-256 curve)
/// - x (label -2): 32 bytes (x coordinate)
/// - y (label -3): 32 bytes (y coordinate)
///
/// # Example COSE Key Structure (CBOR diagnostic notation):
/// ```text
/// {
///   1: 2,           # kty: EC2
///   3: -7,          # alg: ES256
///   -1: 1,          # crv: P-256
///   -2: h'<32 bytes>', # x coordinate
///   -3: h'<32 bytes>'  # y coordinate
/// }
/// ```
fn parse_cose_public_key(pubkey_str: &str) -> Result<PasskeyPublicKey, String> {
	use ciborium::Value;
	use alloy::primitives::FixedBytes;

	// Decode hex string to bytes
	let cose_bytes = alloy::hex::decode(pubkey_str)
		.map_err(|_| "Invalid hex encoding in COSE key".to_string())?;

	// Only try CBOR parsing if the data looks like it could be CBOR
	// CBOR typically starts with specific bytes, but for robustness we'll try to parse anything
	// that isn't obviously a raw EC key format

	// Parse CBOR to extract map structure
	let cbor_value: Value = ciborium::from_reader(&cose_bytes[..])
		.map_err(|e| format!("Failed to parse CBOR: {}", e))?;

	// Extract the map from CBOR value
	let cose_map = match cbor_value {
		Value::Map(map) => map,
		_ => return Err("COSE key must be a CBOR map".to_string()),
	};

	// Helper function to get integer value from map
	let get_int_value = |map: &Vec<(Value, Value)>, key: i64| -> Result<i64, String> {
		let key_value = Value::Integer(key.into());
		for (k, v) in map.iter() {
			if k == &key_value {
				match v {
					Value::Integer(val) => {
						// Convert ciborium::value::Integer to i64
						let i64_val: i64 = (*val).try_into().map_err(|_| format!("Integer value too large for key {}", key))?;
						return Ok(i64_val);
					},
					_ => return Err(format!("Key {} must be an integer", key)),
				}
			}
		}
		Err(format!("Required key {} not found", key))
	};

	// Helper function to get bytes value from map
	let get_bytes_value = |map: &Vec<(Value, Value)>, key: i64| -> Result<Vec<u8>, String> {
		let key_value = Value::Integer(key.into());
		for (k, v) in map.iter() {
			if k == &key_value {
				match v {
					Value::Bytes(bytes) => return Ok(bytes.clone()),
					_ => return Err(format!("Key {} must be bytes", key)),
				}
			}
		}
		Err(format!("Required key {} not found", key))
	};

	// Validate required fields according to COSE spec
	// kty (key type): 2 (EC2 - Elliptic Curve Keys w/ x- and y-coordinate pair)
	let kty = get_int_value(&cose_map, 1)?;
	if kty != 2 {
		return Err(format!("Invalid key type: expected 2 (EC2), got {}", kty));
	}

	// alg (algorithm): -7 (ES256 - ECDSA w/ SHA-256)
	let alg = get_int_value(&cose_map, 3)?;
	if alg != -7 {
		return Err(format!("Invalid algorithm: expected -7 (ES256), got {}", alg));
	}

	// crv (curve): 1 (P-256)
	let crv = get_int_value(&cose_map, -1)?;
	if crv != 1 {
		return Err(format!("Invalid curve: expected 1 (P-256), got {}", crv));
	}

	// Extract x (-2) and y (-3) coordinates
	let x_bytes = get_bytes_value(&cose_map, -2)?;
	let y_bytes = get_bytes_value(&cose_map, -3)?;

	// Validate coordinate lengths (must be 32 bytes for P-256)
	if x_bytes.len() != 32 {
		return Err(format!("Invalid x coordinate length: expected 32 bytes, got {}", x_bytes.len()));
	}
	if y_bytes.len() != 32 {
		return Err(format!("Invalid y coordinate length: expected 32 bytes, got {}", y_bytes.len()));
	}

	// Convert to PasskeyPublicKey format
	let x = FixedBytes::<32>::from_slice(&x_bytes);
	let y = FixedBytes::<32>::from_slice(&y_bytes);

	Ok(PasskeyPublicKey { x, y })
}

#[cfg(test)]
pub mod test {
	use crate::OmniAccountClient;
	use super::parse_passkey_public_key;
	use alloy::network::EthereumWallet;
	use alloy::primitives::{address, U256};
	use alloy::signers::local::PrivateKeySigner;
	use ethereum_rpc::mocks::MockRpcProvider;
	use ethereum_rpc::AlloyRpcProvider;
	use std::str::FromStr;
	use std::sync::Arc;
	use test_log::test;

	#[test(tokio::test)]
	pub async fn test_get_nonce() {
		let expected_nonce = U256::from(42);
		let account_address = address!("0x3c50ecfcda4b0f93fa86baa72807208267a5013d");
		let mut rpc_client = MockRpcProvider::new();

		// Mock the call response - nonce encoded as U256
		rpc_client
			.expect_call()
			.with(mockall::predicate::always())
			.times(1)
			.returning(move |_| Ok(U256::from(42).to_be_bytes_vec()));

		let client = OmniAccountClient::new(account_address, Arc::new(rpc_client));
		let nonce = client.get_nonce().await.unwrap();

		assert_eq!(expected_nonce, nonce);
	}

	#[test(tokio::test)]
	pub async fn test_add_root_signer() {
		let account_address = address!("0x922D6956C99E12DFeB3224DEA977D0939758A1Fe");
		let root_signer = address!("0x1234567890123456789012345678901234567890");
		let mut rpc_client = MockRpcProvider::new();

		// Mock successful transaction
		rpc_client
			.expect_send_transaction()
			.with(mockall::predicate::always())
			.times(1)
			.returning(|_| Ok("0x1234567890abcdef".to_string()));

		let client = OmniAccountClient::new(account_address, Arc::new(rpc_client));
		let result = client.add_root_signer(root_signer).await;

		assert!(result.is_ok());
	}

	#[test(tokio::test)]
	pub async fn test_remove_root_signer() {
		let account_address = address!("0x922D6956C99E12DFeB3224DEA977D0939758A1Fe");
		let root_signer = address!("0x1234567890123456789012345678901234567890");
		let mut rpc_client = MockRpcProvider::new();

		// Mock successful transaction
		rpc_client
			.expect_send_transaction()
			.with(mockall::predicate::always())
			.times(1)
			.returning(|_| Ok("0x1234567890abcdef".to_string()));

		let client = OmniAccountClient::new(account_address, Arc::new(rpc_client));
		let result = client.remove_root_signer(root_signer).await;

		assert!(result.is_ok());
	}

	#[test(tokio::test)]
	pub async fn test_add_root_signer_failure() {
		let account_address = address!("0x922D6956C99E12DFeB3224DEA977D0939758A1Fe");
		let root_signer = address!("0x1234567890123456789012345678901234567890");
		let mut rpc_client = MockRpcProvider::new();

		// Mock transaction failure
		rpc_client
			.expect_send_transaction()
			.with(mockall::predicate::always())
			.times(1)
			.returning(|_| Err(()));

		let client = OmniAccountClient::new(account_address, Arc::new(rpc_client));
		let result = client.add_root_signer(root_signer).await;

		assert!(result.is_err());
	}

	#[test(tokio::test)]
	pub async fn test_remove_root_signer_failure() {
		let account_address = address!("0x922D6956C99E12DFeB3224DEA977D0939758A1Fe");
		let root_signer = address!("0x1234567890123456789012345678901234567890");
		let mut rpc_client = MockRpcProvider::new();

		// Mock transaction failure
		rpc_client
			.expect_send_transaction()
			.with(mockall::predicate::always())
			.times(1)
			.returning(|_| Err(()));

		let client = OmniAccountClient::new(account_address, Arc::new(rpc_client));
		let result = client.remove_root_signer(root_signer).await;

		assert!(result.is_err());
	}

	#[test(tokio::test)]
	pub async fn test_get_nonce_decode_failure() {
		let account_address = address!("0x922D6956C99E12DFeB3224DEA977D0939758A1Fe");
		let mut rpc_client = MockRpcProvider::new();

		// Mock call with invalid response data
		rpc_client
			.expect_call()
			.with(mockall::predicate::always())
			.times(1)
			.returning(|_| Ok(vec![0x12, 0x34])); // Invalid length for U256

		let client = OmniAccountClient::new(account_address, Arc::new(rpc_client));
		let result = client.get_nonce().await;

		assert!(result.is_err());
	}

	#[test(tokio::test)]
	pub async fn test_get_nonce_call_failure() {
		let account_address = address!("0x922D6956C99E12DFeB3224DEA977D0939758A1Fe");
		let mut rpc_client = MockRpcProvider::new();

		// Mock call failure
		rpc_client
			.expect_call()
			.with(mockall::predicate::always())
			.times(1)
			.returning(|_| Err(None));

		let client = OmniAccountClient::new(account_address, Arc::new(rpc_client));
		let result = client.get_nonce().await;

		assert!(result.is_err());
	}

	// Integration tests (marked as ignore for manual testing)
	// account_address should point to SmartAccount instance, run `try_full_flow` from entry_point_client.rs first to deploy it and reuse address from logs
	#[test(tokio::test)]
	#[ignore = "manual"]
	pub async fn try_get_nonce_integration() {
		let account_address = address!("0x3c50ecfcda4b0f93fa86baa72807208267a5013d");
		let rpc_client = Arc::new(AlloyRpcProvider::new("http://localhost:8545"));
		let client = OmniAccountClient::new(account_address, rpc_client);

		let nonce = client.get_nonce().await.unwrap();
		println!("Account nonce: {}", nonce);
	}

	#[test(tokio::test)]
	#[ignore = "manual"]
	pub async fn try_add_root_signer_integration() {
		let account_address = address!("0x3c50ecfcda4b0f93fa86baa72807208267a5013d");
		let root_signer = address!("0x1234567890123456789012345678901234567890");

		// Setup wallet for signing transactions
		let signer = PrivateKeySigner::from_str(
			"0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6",
		)
		.unwrap();
		let wallet = EthereumWallet::new(signer);
		let rpc_client =
			Arc::new(AlloyRpcProvider::new_with_wallet("http://localhost:8545", wallet));

		let client = OmniAccountClient::new(account_address, rpc_client);
		let result = client.add_root_signer(root_signer).await;

		assert!(result.is_ok());
		println!("Successfully added root signer: {}", root_signer);
	}

	#[test(tokio::test)]
	#[ignore = "manual"]
	pub async fn try_remove_root_signer_integration() {
		let account_address = address!("0x3c50ecfcda4b0f93fa86baa72807208267a5013d");
		let root_signer = address!("0x1234567890123456789012345678901234567890");

		// Setup wallet for signing transactions
		let signer = PrivateKeySigner::from_str(
			"0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6",
		)
		.unwrap();
		let wallet = EthereumWallet::new(signer);
		let rpc_client =
			Arc::new(AlloyRpcProvider::new_with_wallet("http://localhost:8545", wallet));

		let client = OmniAccountClient::new(account_address, rpc_client);
		let result = client.remove_root_signer(root_signer).await;

		assert!(result.is_ok());
		println!("Successfully removed root signer: {}", root_signer);
	}

	#[test]
	pub fn test_parse_passkey_public_key_raw_formats() {
		// Test uncompressed format without prefix (128 hex chars)
		let raw_uncompressed = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdeffedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321";
		let result = parse_passkey_public_key(raw_uncompressed);
		assert!(result.is_ok());

		// Test uncompressed format with 04 prefix (130 hex chars)
		let prefixed_uncompressed = "041234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdeffedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321";
		let result = parse_passkey_public_key(prefixed_uncompressed);
		assert!(result.is_ok());

		// Test with 0x prefix
		let hex_prefixed = "0x041234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdeffedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321";
		let result = parse_passkey_public_key(hex_prefixed);
		assert!(result.is_ok());

		// Test compressed format (should fail)
		let compressed = "021234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
		let result = parse_passkey_public_key(compressed);
		assert!(result.is_err());
		if let Err(err) = result {
			assert!(err.contains("Compressed public key format not supported"));
		}
	}

	#[test]
	pub fn test_parse_passkey_public_key_cose_format() {
		// Create a valid COSE key using ciborium for testing
		use ciborium::Value;

		// Add test coordinates (32 bytes each)
		let x_coords = vec![0x65, 0xed, 0xa5, 0xa1, 0x25, 0x77, 0xc2, 0xba, 0xe8, 0x29, 0x43, 0x7f, 0xe3, 0x38, 0x70, 0x1a, 0x10, 0xaa, 0xa3, 0x75, 0xe1, 0xbb, 0x5b, 0x5d, 0xe1, 0x08, 0xde, 0x43, 0x9c, 0x08, 0x55, 0x1d];
		let y_coords = vec![0x1e, 0x52, 0xed, 0x75, 0x70, 0x11, 0x63, 0xf7, 0xf9, 0xe4, 0x0d, 0xdf, 0x9f, 0x34, 0x1b, 0x3d, 0xc9, 0xba, 0x86, 0x0a, 0xf7, 0xe0, 0xca, 0x7c, 0xa7, 0xe9, 0xee, 0xcd, 0x00, 0x84, 0xd1, 0x9c];

		// Create COSE key map with required fields (using Vec of tuples)
		let cose_map = vec![
			(Value::Integer(1.into()), Value::Integer(2.into())), // kty: EC2
			(Value::Integer(3.into()), Value::Integer((-7).into())), // alg: ES256
			(Value::Integer((-1).into()), Value::Integer(1.into())), // crv: P-256
			(Value::Integer((-2).into()), Value::Bytes(x_coords.clone())), // x coordinate
			(Value::Integer((-3).into()), Value::Bytes(y_coords.clone())), // y coordinate
		];

		// Encode to CBOR
		let cose_value = Value::Map(cose_map);
		let mut cose_bytes = Vec::new();
		ciborium::into_writer(&cose_value, &mut cose_bytes).unwrap();

		// Convert to hex string
		let cose_hex = alloy::hex::encode(&cose_bytes);

		// Test parsing
		let result = parse_passkey_public_key(&cose_hex);

		// Should now succeed with proper COSE implementation
		assert!(result.is_ok(), "COSE parsing should succeed with valid key");

		let public_key = result.unwrap();
		// Verify the coordinates match what we put in
		assert_eq!(public_key.x.as_slice(), &x_coords[..]);
		assert_eq!(public_key.y.as_slice(), &y_coords[..]);
	}

	#[test]
	pub fn test_parse_passkey_public_key_invalid_cose() {
		use ciborium::Value;

		// Test invalid key type (should be 2 for EC2)
		let invalid_kty_map = vec![
			(Value::Integer(1.into()), Value::Integer(1.into())), // kty: OKP (invalid)
			(Value::Integer(3.into()), Value::Integer((-7).into())), // alg: ES256
			(Value::Integer((-1).into()), Value::Integer(1.into())), // crv: P-256
		];

		let cose_value = Value::Map(invalid_kty_map);
		let mut cose_bytes = Vec::new();
		ciborium::into_writer(&cose_value, &mut cose_bytes).unwrap();
		let cose_hex = alloy::hex::encode(&cose_bytes);

		let result = parse_passkey_public_key(&cose_hex);
		assert!(result.is_err());
		if let Err(err) = result {
			assert!(err.contains("Invalid key type: expected 2 (EC2), got 1"));
		}

		// Test invalid algorithm (should be -7 for ES256)
		let invalid_alg_map = vec![
			(Value::Integer(1.into()), Value::Integer(2.into())), // kty: EC2
			(Value::Integer(3.into()), Value::Integer((-8).into())), // alg: EdDSA (invalid)
			(Value::Integer((-1).into()), Value::Integer(1.into())), // crv: P-256
		];

		let cose_value = Value::Map(invalid_alg_map);
		let mut cose_bytes = Vec::new();
		ciborium::into_writer(&cose_value, &mut cose_bytes).unwrap();
		let cose_hex = alloy::hex::encode(&cose_bytes);

		let result = parse_passkey_public_key(&cose_hex);
		assert!(result.is_err());
		if let Err(err) = result {
			assert!(err.contains("Invalid algorithm: expected -7 (ES256), got -8"));
		}

		// Test invalid curve (should be 1 for P-256)
		let invalid_crv_map = vec![
			(Value::Integer(1.into()), Value::Integer(2.into())), // kty: EC2
			(Value::Integer(3.into()), Value::Integer((-7).into())), // alg: ES256
			(Value::Integer((-1).into()), Value::Integer(2.into())), // crv: P-384 (invalid)
		];

		let cose_value = Value::Map(invalid_crv_map);
		let mut cose_bytes = Vec::new();
		ciborium::into_writer(&cose_value, &mut cose_bytes).unwrap();
		let cose_hex = alloy::hex::encode(&cose_bytes);

		let result = parse_passkey_public_key(&cose_hex);
		assert!(result.is_err());
		if let Err(err) = result {
			assert!(err.contains("Invalid curve: expected 1 (P-256), got 2"));
		}
	}

	#[test]
	pub fn test_parse_passkey_public_key_invalid_formats() {
		// Test invalid length
		let too_short = "1234567890abcdef";
		let result = parse_passkey_public_key(too_short);
		assert!(result.is_err());

		// Test invalid hex
		let invalid_hex = "gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg";
		let result = parse_passkey_public_key(invalid_hex);
		assert!(result.is_err());

		// Test wrong prefix for uncompressed
		let wrong_prefix = "051234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdeffedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321";
		let result = parse_passkey_public_key(wrong_prefix);
		assert!(result.is_err());
		if let Err(err) = result {
			assert!(err.contains("Invalid uncompressed public key prefix"));
		}
	}
}
