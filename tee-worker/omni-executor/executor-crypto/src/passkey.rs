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

use base64::Engine;
use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Debug)]
pub enum PasskeyError {
	InvalidPublicKeyFormat,
	InvalidSignatureFormat,
	ParseError(String),
	AttestationParseError(String),
	ChallengeVerificationFailed,
	OriginVerificationFailed,
}

impl fmt::Display for PasskeyError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			PasskeyError::InvalidPublicKeyFormat => write!(f, "Invalid public key format"),
			PasskeyError::InvalidSignatureFormat => write!(f, "Invalid signature format"),
			PasskeyError::ParseError(msg) => write!(f, "Parse error: {}", msg),
			PasskeyError::AttestationParseError(msg) => {
				write!(f, "Attestation parse error: {}", msg)
			},
			PasskeyError::ChallengeVerificationFailed => write!(f, "Challenge verification failed"),
			PasskeyError::OriginVerificationFailed => write!(f, "Origin verification failed"),
		}
	}
}

impl std::error::Error for PasskeyError {}

#[derive(Debug, Clone)]
pub struct PasskeyPublicKey {
	pub verifying_key: VerifyingKey,
}

#[derive(Debug, Clone)]
pub struct ParsedAttestationObject {
	pub auth_data: Vec<u8>,
	pub fmt: String,
	pub att_stmt: Vec<u8>, // Simplified - could be parsed further
}

#[derive(Debug, Clone)]
pub struct ClientData {
	pub type_: String,
	pub challenge: String,
	pub origin: String,
	pub cross_origin: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct AttestationResult {
	pub credential_id: String,
	pub public_key: PasskeyPublicKey,
}

pub struct PasskeyVerifier;

impl PasskeyVerifier {
	pub fn verify_client_data_json<F>(
		client_data_json: &str,
		omni_account: &[u8; 32],
		expected_origin: &str,
		expected_type: &str,
		verify_and_consume_challenge: F,
	) -> Result<(), PasskeyError>
	where
		F: FnOnce(&str, &[u8; 32]) -> Result<(), PasskeyError>,
	{
		// Parse client data JSON
		let client_data = Self::parse_client_data_json(client_data_json)?;

		// Verify origin
		if client_data.origin != expected_origin {
			return Err(PasskeyError::OriginVerificationFailed);
		}

		// Verify type
		if client_data.type_ != expected_type {
			return Err(PasskeyError::AttestationParseError(format!(
				"Invalid type, expected {}",
				expected_type
			)));
		}

		// Verify and consume challenge using the provided function
		verify_and_consume_challenge(&client_data.challenge, omni_account)
			.map_err(|_| PasskeyError::ChallengeVerificationFailed)?;

		Ok(())
	}

	pub fn from_sec1_bytes(sec1_bytes: &[u8]) -> Result<PasskeyPublicKey, PasskeyError> {
		let verifying_key = VerifyingKey::from_sec1_bytes(sec1_bytes)
			.map_err(|_| PasskeyError::InvalidPublicKeyFormat)?;
		Ok(PasskeyPublicKey { verifying_key })
	}

	pub fn verify_attestation(attestation_object: &str) -> Result<AttestationResult, PasskeyError> {
		let parsed_attestation_object = Self::parse_attestation_object(attestation_object)?;

		let (credential_id, public_key) =
			Self::extract_credential_and_key_from_auth_data(&parsed_attestation_object.auth_data)?;

		Ok(AttestationResult { credential_id, public_key })
	}

	pub fn verify_rp_id_hash(
		auth_data_bytes: &[u8],
		expected_rp_id: &str,
	) -> Result<(), PasskeyError> {
		if auth_data_bytes.len() < 32 {
			return Err(PasskeyError::ParseError(
				"Auth data too short to contain RP ID hash".to_string(),
			));
		}
		let rp_id_hash_from_auth_data = &auth_data_bytes[0..32];
		let expected_rp_id_hash = Sha256::digest(expected_rp_id.as_bytes());
		if rp_id_hash_from_auth_data != &expected_rp_id_hash[..] {
			return Err(PasskeyError::ParseError(format!(
				"RP ID hash mismatch. Expected RP ID: '{}', hash: {:02x?}, but got: {:02x?}",
				expected_rp_id,
				&expected_rp_id_hash[..],
				rp_id_hash_from_auth_data
			)));
		}

		Ok(())
	}

	pub fn parse_client_data_json(client_data_json_b64: &str) -> Result<ClientData, PasskeyError> {
		// Decode base64
		let client_data_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
			.decode(client_data_json_b64)
			.map_err(|e| {
				PasskeyError::AttestationParseError(format!("Base64 decode error: {}", e))
			})?;

		// Parse JSON
		let client_data_json = std::str::from_utf8(&client_data_bytes)
			.map_err(|e| PasskeyError::AttestationParseError(format!("UTF-8 error: {}", e)))?;

		let json_value: serde_json::Value = serde_json::from_str(client_data_json)
			.map_err(|e| PasskeyError::AttestationParseError(format!("JSON parse error: {}", e)))?;

		let client_data = ClientData {
			type_: json_value
				.get("type")
				.and_then(|v| v.as_str())
				.ok_or_else(|| {
					PasskeyError::AttestationParseError("Missing type field".to_string())
				})?
				.to_string(),
			challenge: json_value
				.get("challenge")
				.and_then(|v| v.as_str())
				.ok_or_else(|| {
					PasskeyError::AttestationParseError("Missing challenge field".to_string())
				})?
				.to_string(),
			origin: json_value
				.get("origin")
				.and_then(|v| v.as_str())
				.ok_or_else(|| {
					PasskeyError::AttestationParseError("Missing origin field".to_string())
				})?
				.to_string(),
			cross_origin: json_value.get("crossOrigin").and_then(|v| v.as_bool()),
		};

		Ok(client_data)
	}

	fn parse_attestation_object(
		attestation_object_b64: &str,
	) -> Result<ParsedAttestationObject, PasskeyError> {
		// Decode base64
		let attestation_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
			.decode(attestation_object_b64)
			.map_err(|e| {
				PasskeyError::AttestationParseError(format!("Base64 decode error: {}", e))
			})?;

		// Parse CBOR using ciborium
		// WebAuthn attestation object structure:
		// {
		//   "fmt": text string,
		//   "attStmt": map,
		//   "authData": bytes
		// }
		let cbor_value: ciborium::Value = ciborium::de::from_reader(&attestation_bytes[..])
			.map_err(|e| {
				PasskeyError::AttestationParseError(format!("CBOR decode error: {}", e))
			})?;

		// Extract the map from CBOR value
		let map = match cbor_value {
			ciborium::Value::Map(m) => m,
			_ => {
				return Err(PasskeyError::AttestationParseError(
					"Attestation object must be a CBOR map".to_string(),
				))
			},
		};

		// Extract authData (required field)
		let auth_data = Self::extract_field_from_cbor_map(&map, "authData")?;
		let auth_data_bytes = match auth_data {
			ciborium::Value::Bytes(bytes) => bytes.clone(),
			_ => {
				return Err(PasskeyError::AttestationParseError(
					"authData must be bytes".to_string(),
				))
			},
		};

		// Extract fmt (format, required field)
		let fmt = Self::extract_field_from_cbor_map(&map, "fmt")?;
		let fmt_string = match fmt {
			ciborium::Value::Text(text) => text.clone(),
			_ => return Err(PasskeyError::AttestationParseError("fmt must be text".to_string())),
		};

		// Extract attStmt (attestation statement, optional but usually present)
		let att_stmt = match Self::extract_field_from_cbor_map(&map, "attStmt") {
			Ok(val) => {
				// Serialize attStmt back to bytes for storage (if needed)
				let mut att_stmt_bytes = Vec::new();
				ciborium::ser::into_writer(&val, &mut att_stmt_bytes).map_err(|e| {
					PasskeyError::AttestationParseError(format!(
						"Failed to serialize attStmt: {}",
						e
					))
				})?;
				att_stmt_bytes
			},
			Err(_) => vec![], // attStmt is optional for "none" format
		};

		Ok(ParsedAttestationObject { auth_data: auth_data_bytes, fmt: fmt_string, att_stmt })
	}

	fn extract_field_from_cbor_map<'a>(
		map: &'a [(ciborium::Value, ciborium::Value)],
		field_name: &str,
	) -> Result<&'a ciborium::Value, PasskeyError> {
		for (key, value) in map {
			if let ciborium::Value::Text(key_text) = key {
				if key_text == field_name {
					return Ok(value);
				}
			}
		}
		Err(PasskeyError::AttestationParseError(format!(
			"Required field '{}' not found in attestation object",
			field_name
		)))
	}

	fn extract_credential_and_key_from_auth_data(
		auth_data: &[u8],
	) -> Result<(String, PasskeyPublicKey), PasskeyError> {
		if auth_data.len() < 55 {
			return Err(PasskeyError::AttestationParseError(
				"Auth data too short to contain credential data".to_string(),
			));
		}

		// Check if attested credential data is present (AT flag)
		let flags = auth_data[32];
		let at_flag = (flags & 0x40) != 0;

		if !at_flag {
			return Err(PasskeyError::AttestationParseError(
				"No attested credential data present".to_string(),
			));
		}

		// Skip: RP ID hash (32) + flags (1) + counter (4) + AAGUID (16) = 53 bytes
		// Then: credential ID length (2 bytes) + credential ID + public key (COSE format)

		let cred_id_len = u16::from_be_bytes([auth_data[53], auth_data[54]]) as usize;
		let cred_id_start = 55;
		let pub_key_start = 55 + cred_id_len;

		if auth_data.len() < pub_key_start {
			return Err(PasskeyError::AttestationParseError(
				"Auth data too short for credential ID".to_string(),
			));
		}

		// Extract credential ID
		let credential_id_bytes = &auth_data[cred_id_start..cred_id_start + cred_id_len];
		let credential_id =
			base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(credential_id_bytes);

		// Extract and parse COSE public key (CBOR-encoded)
		let cose_key_bytes = &auth_data[pub_key_start..];

		// Parse COSE key using ciborium
		let cose_key: ciborium::Value = ciborium::de::from_reader(cose_key_bytes).map_err(|e| {
			PasskeyError::AttestationParseError(format!("Failed to parse COSE key: {}", e))
		})?;

		// COSE key is a CBOR map with integer keys
		// For ES256 (P-256), we need:
		// -1: x coordinate (bytes)
		// -2: y coordinate (bytes)
		// -3: curve (1 = P-256)
		let cose_map = match cose_key {
			ciborium::Value::Map(m) => m,
			_ => {
				return Err(PasskeyError::AttestationParseError(
					"COSE key must be a map".to_string(),
				))
			},
		};

		let x_coord = Self::extract_cose_key_param(&cose_map, -2)?;
		let y_coord = Self::extract_cose_key_param(&cose_map, -3)?;

		// Construct SEC1 uncompressed public key: 0x04 || x || y
		let mut sec1_bytes = vec![0x04];
		sec1_bytes.extend_from_slice(&x_coord);
		sec1_bytes.extend_from_slice(&y_coord);

		let verifying_key = VerifyingKey::from_sec1_bytes(&sec1_bytes)
			.map_err(|_| PasskeyError::InvalidPublicKeyFormat)?;

		Ok((credential_id, PasskeyPublicKey { verifying_key }))
	}

	fn extract_cose_key_param(
		cose_map: &[(ciborium::Value, ciborium::Value)],
		key: i32,
	) -> Result<Vec<u8>, PasskeyError> {
		for (k, v) in cose_map {
			if let ciborium::Value::Integer(int_key) = k {
				if *int_key == key.into() {
					if let ciborium::Value::Bytes(bytes) = v {
						return Ok(bytes.clone());
					}
				}
			}
		}
		Err(PasskeyError::AttestationParseError(format!(
			"COSE key parameter {} not found or invalid",
			key
		)))
	}

	pub fn verify_passkey_signature_only(
		auth_data: &str,
		client_data_json: &str,
		signature: &str,
		public_key: &PasskeyPublicKey,
	) -> Result<bool, PasskeyError> {
		// Decode signature from base64url as per WebAuthn spec
		let signature_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
			.decode(signature)
			.map_err(|_| PasskeyError::InvalidSignatureFormat)?;

		// WebAuthn signatures can be in two formats:
		// 1. Raw r||s format: exactly 64 bytes (32 bytes r + 32 bytes s)
		// 2. DER format: variable length (typically 70-72 bytes)
		let ecdsa_signature = if signature_bytes.len() == 64 {
			// Raw format: use directly
			Signature::from_slice(&signature_bytes)
				.map_err(|_| PasskeyError::InvalidSignatureFormat)?
		} else if signature_bytes.len() > 64 && signature_bytes[0] == 0x30 {
			// DER format: parse and convert to raw
			Signature::from_der(&signature_bytes)
				.map_err(|_| PasskeyError::InvalidSignatureFormat)?
		} else {
			return Err(PasskeyError::InvalidSignatureFormat);
		};

		let client_data_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
			.decode(client_data_json)
			.map_err(|e| PasskeyError::ParseError(format!("Client data decode error: {}", e)))?;

		// auth_data is base64url-encoded as per WebAuthn spec
		let auth_data_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
			.decode(auth_data)
			.map_err(|e| PasskeyError::ParseError(format!("Auth data decode error: {}", e)))?;
		let client_data_hash = Sha256::digest(&client_data_bytes);
		// Per WebAuthn spec: signature = sign(authenticatorData || SHA256(clientDataJSON))
		let mut signing_data = Vec::new();
		signing_data.extend_from_slice(&auth_data_bytes);
		signing_data.extend_from_slice(&client_data_hash);

		Ok(public_key.verifying_key.verify(&signing_data, &ecdsa_signature).is_ok())
	}
}
