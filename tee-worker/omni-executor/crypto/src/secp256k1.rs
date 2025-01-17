// This code is copied from `sp-io` crate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use secp256k1::{
	ecdsa::{RecoverableSignature, RecoveryId},
	Message, SECP256K1,
};

/// Error verifying ECDSA signature
pub enum EcdsaVerifyError {
	/// Incorrect value of R or S
	BadRS,
	/// Incorrect value of V
	BadV,
	/// Invalid signature
	BadSignature,
}

/// Verify and recover a SECP256k1 ECDSA signature.
///
/// - `sig` is passed in RSV format. V should be either `0/1` or `27/28`.
/// - `msg` is the blake2-256 hash of the message.
///
/// Returns `Err` if the signature is bad, otherwise the 64-byte pubkey
/// (doesn't include the 0x04 prefix).
pub fn secp256k1_ecdsa_recover(
	sig: &[u8; 65],
	msg: &[u8; 32],
) -> Result<[u8; 64], EcdsaVerifyError> {
	let rid = RecoveryId::from_i32(if sig[64] > 26 { sig[64] - 27 } else { sig[64] } as i32)
		.map_err(|_| EcdsaVerifyError::BadV)?;
	let sig =
		RecoverableSignature::from_compact(&sig[..64], rid).map_err(|_| EcdsaVerifyError::BadRS)?;
	let msg = Message::from_digest_slice(msg).expect("Message is 32 bytes; qed");
	let pubkey = SECP256K1
		.recover_ecdsa(&msg, &sig)
		.map_err(|_| EcdsaVerifyError::BadSignature)?;
	let mut res = [0u8; 64];
	res.copy_from_slice(&pubkey.serialize_uncompressed()[1..]);
	Ok(res)
}

/// Verify and recover a SECP256k1 ECDSA signature.
///
/// - `sig` is passed in RSV format. V should be either `0/1` or `27/28`.
/// - `msg` is the blake2-256 hash of the message.
///
/// Returns `Err` if the signature is bad, otherwise the 33-byte compressed pubkey.
pub fn secp256k1_ecdsa_recover_compressed(
	sig: &[u8; 65],
	msg: &[u8; 32],
) -> Result<[u8; 33], EcdsaVerifyError> {
	let rid = RecoveryId::from_i32(if sig[64] > 26 { sig[64] - 27 } else { sig[64] } as i32)
		.map_err(|_| EcdsaVerifyError::BadV)?;
	let sig =
		RecoverableSignature::from_compact(&sig[..64], rid).map_err(|_| EcdsaVerifyError::BadRS)?;
	let msg = Message::from_digest_slice(msg).expect("Message is 32 bytes; qed");
	let pubkey = SECP256K1
		.recover_ecdsa(&msg, &sig)
		.map_err(|_| EcdsaVerifyError::BadSignature)?;
	Ok(pubkey.serialize())
}
