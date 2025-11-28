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

use alloy::primitives::B256;
use alloy::signers::{local::PrivateKeySigner, SignerSync};
use serde_json::{json, Value};
use sp_core::keccak_256;

// Mock private key for testing - corresponds to 0xa0Ee7A142d267C1f36714E4a8F75612F20a79720
const MOCK_PRIVATE_KEY: &str = "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6";

fn ethereum_to_substrate_signature(ethereum_sig: &[u8]) -> [u8; 65] {
	let mut substrate_sig = [0u8; 65];
	substrate_sig[0..64].copy_from_slice(&ethereum_sig[0..64]);
	substrate_sig[64] = match ethereum_sig[64] {
		27 => 0,
		28 => 1,
		v => v, // Keep as-is if already in Substrate format
	};
	substrate_sig
}

fn get_mock_pubkey_bytes(chain_type: &str) -> Vec<u8> {
	match chain_type {
		"Evm" | "Tron" => {
			// Calculate the compressed public key from MOCK_PRIVATE_KEY for EVM/Tron chains
			// We'll use secp256k1 to derive the public key properly
			let private_key_bytes =
				hex::decode(&MOCK_PRIVATE_KEY[2..]).expect("Invalid private key");

			// Use secp256k1 to get the compressed public key
			let secret_key = libsecp256k1::SecretKey::parse_slice(&private_key_bytes)
				.expect("Invalid private key for secp256k1");
			let public_key = libsecp256k1::PublicKey::from_secret_key(&secret_key);

			// Serialize as compressed (33 bytes)
			public_key.serialize_compressed().to_vec()
		},
		"Solana" => {
			// For Solana, we use a derived 32-byte Ed25519-style key
			// Since the actual signer service would derive this differently for Solana,
			// we'll use a deterministic derivation from the MOCK_PRIVATE_KEY
			let private_key_bytes =
				hex::decode(&MOCK_PRIVATE_KEY[2..]).expect("Invalid private key");

			// Use keccak256 hash of the private key as a mock Ed25519 public key (32 bytes)
			let ed25519_pubkey = keccak_256(&private_key_bytes);
			ed25519_pubkey.to_vec()
		},
		_ => {
			// Default to EVM format for unknown chain types
			let private_key_bytes =
				hex::decode(&MOCK_PRIVATE_KEY[2..]).expect("Invalid private key");

			let secret_key = libsecp256k1::SecretKey::parse_slice(&private_key_bytes)
				.expect("Invalid private key for secp256k1");
			let public_key = libsecp256k1::PublicKey::from_secret_key(&secret_key);

			public_key.serialize_compressed().to_vec()
		},
	}
}

fn build_success_response(body: Value) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
	Ok(Box::new(warp::reply::with_status(warp::reply::json(&body), warp::http::StatusCode::OK)))
}

pub async fn handle_dex_method(request: Value) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
	let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
	let id = request.get("id").cloned().unwrap_or(json!(1));

	match method {
		"dex_getShieldingKey" => {
			let response = json!({
				"jsonrpc": "2.0",
				"result": {
					"n": "abcdef1234567890",
					"e": "010001"
				},
				"id": id
			});
			build_success_response(response)
		},
		"dex_getWallet" => {
			// Extract chain type from the request params
			let empty_params = json!({});
			let params = request.get("params").unwrap_or(&empty_params);
			let empty_payload = json!({});
			let payload = params.get("payload").unwrap_or(&empty_payload);
			let empty_wallet = json!({});
			let wallet = payload.get("wallet").unwrap_or(&empty_wallet);
			let chain_type = wallet.get("chain_type").and_then(|ct| ct.as_str()).unwrap_or("Evm");

			// Return the public key bytes as hex string (instead of address)
			let pubkey_bytes = get_mock_pubkey_bytes(chain_type);
			let pubkey_hex = hex::encode(&pubkey_bytes);

			tracing::info!("Returning public key for chain type {}: 0x{}", chain_type, pubkey_hex);

			let response = json!({
				"jsonrpc": "2.0",
				"result": pubkey_hex,
				"id": id
			});
			build_success_response(response)
		},
		"dex_signWallet" => {
			// Extract the message to sign from the request params
			let empty_params = json!({});
			let params = request.get("params").unwrap_or(&empty_params);
			let empty_payload = json!({});
			let payload = params.get("payload").unwrap_or(&empty_payload);
			let msg_hex = payload.get("msg").and_then(|m| m.as_str()).unwrap_or("00");

			// Decode the message and sign it with the mock private key using alloy
			let message_bytes = hex::decode(msg_hex).unwrap_or_else(|_| vec![0u8; 32]);

			// Ensure we have exactly 32 bytes for the message hash
			let message_hash: [u8; 32] = if message_bytes.len() == 32 {
				message_bytes.try_into().unwrap()
			} else {
				// If not 32 bytes, hash it with keccak256 to get 32 bytes
				keccak_256(&message_bytes)
			};

			// Use alloy signer for Ethereum-compatible signatures, then convert to Substrate format
			let private_key_bytes =
				hex::decode(&MOCK_PRIVATE_KEY[2..]).expect("Invalid private key");
			let private_key = B256::from_slice(&private_key_bytes);
			let signer = PrivateKeySigner::from_bytes(&private_key).expect("Invalid private key");

			// Sign the hash directly (alloy will handle recovery ID correctly)
			let ethereum_signature =
				signer.sign_hash_sync(&B256::from(message_hash)).expect("Failed to sign");

			// Convert Ethereum signature (v=27/28) to Substrate format (v=0/1)
			let substrate_signature =
				ethereum_to_substrate_signature(&ethereum_signature.as_bytes());

			// Format signature as hex string (65 bytes: r + s + v with Substrate recovery ID)
			let signature_hex = hex::encode(substrate_signature);

			tracing::info!("Signing message: {}, signature: {}", msg_hex, signature_hex);

			let response = json!({
				"jsonrpc": "2.0",
				"result": signature_hex,
				"id": id
			});
			build_success_response(response)
		},
		"dex_multiSignWallet" => {
			// Extract the messages to sign from the request params
			let empty_params = json!({});
			let params = request.get("params").unwrap_or(&empty_params);
			let empty_payload = json!({});
			let payload = params.get("payload").unwrap_or(&empty_payload);
			let empty_msgs = vec![];
			let msgs = payload.get("msgs").and_then(|m| m.as_array()).unwrap_or(&empty_msgs);

			// Use alloy signer for Ethereum-compatible signatures, then convert to Substrate format
			let private_key_bytes =
				hex::decode(&MOCK_PRIVATE_KEY[2..]).expect("Invalid private key");
			let private_key = B256::from_slice(&private_key_bytes);
			let signer = PrivateKeySigner::from_bytes(&private_key).expect("Invalid private key");

			let mut signatures = Vec::new();

			for msg in msgs {
				let msg_hex = msg.as_str().unwrap_or("00");
				let message_bytes = hex::decode(msg_hex).unwrap_or_else(|_| vec![0u8; 32]);

				// Ensure we have exactly 32 bytes for the message hash
				let message_hash: [u8; 32] = if message_bytes.len() == 32 {
					message_bytes.try_into().unwrap()
				} else {
					// If not 32 bytes, hash it with keccak256 to get 32 bytes
					keccak_256(&message_bytes)
				};

				// Sign the hash directly (alloy will handle recovery ID correctly)
				let ethereum_signature =
					signer.sign_hash_sync(&B256::from(message_hash)).expect("Failed to sign");

				// Convert Ethereum signature (v=27/28) to Substrate format (v=0/1)
				let substrate_signature =
					ethereum_to_substrate_signature(&ethereum_signature.as_bytes());
				let signature_hex = hex::encode(substrate_signature);
				signatures.push(signature_hex);
			}

			tracing::info!("Multi-signing {} messages", msgs.len());

			let response = json!({
				"jsonrpc": "2.0",
				"result": signatures,
				"id": id
			});
			build_success_response(response)
		},
		"dex_exportWallet" => {
			let response = json!({
				"jsonrpc": "2.0",
				"result": {
					"ciphertext": "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
					"aad": "fedcba0987654321fedcba0987654321",
					"nonce": "123456789abcdef0123456789abcdef0"
				},
				"id": id
			});
			build_success_response(response)
		},
		_ => {
			let response = json!({
				"jsonrpc": "2.0",
				"error": {
					"code": -32601,
					"message": "Method not found"
				},
				"id": id
			});
			build_success_response(response)
		},
	}
}
