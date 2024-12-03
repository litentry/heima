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

use alloc::string::String;
use codec::{Decode, Encode};
use ita_stf::{LitentryMultiSignature, TrustedCall};
use itp_stf_primitives::traits::TrustedCallVerification;
use itp_types::parentchain::Index as ParentchainIndex;
use lc_identity_verification::web2::email::VerificationCodeStore;
use lc_omni_account::InMemoryStore as OmniAccountStore;
use litentry_hex_utils::hex_encode;
use litentry_primitives::{Identity, ShardIdentifier};
use sp_core::{
	blake2_256,
	crypto::{AccountId32 as AccountId, UncheckedFrom},
	ed25519,
};

type VerificationCode = String;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum TCAuthentication {
	Web3(LitentryMultiSignature),
	Email(VerificationCode),
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct TrustedCallAuthenticated {
	pub call: TrustedCall,
	pub nonce: ParentchainIndex,
	pub authentication: TCAuthentication,
}

impl TrustedCallAuthenticated {
	pub fn new(
		call: TrustedCall,
		nonce: ParentchainIndex,
		authentication: TCAuthentication,
	) -> Self {
		Self { call, nonce, authentication }
	}
}

impl Default for TrustedCallAuthenticated {
	fn default() -> Self {
		Self {
			call: TrustedCall::noop(AccountId::unchecked_from([0u8; 32].into()).into()),
			nonce: 0,
			authentication: TCAuthentication::Web3(LitentryMultiSignature::Ed25519(
				ed25519::Signature::unchecked_from([0u8; 64]),
			)),
		}
	}
}

// TODO: we should refactor this as verify_signature should not be part of TrustedCallAuthenticated
impl TrustedCallVerification for TrustedCallAuthenticated {
	fn sender_identity(&self) -> &Identity {
		self.call.sender_identity()
	}

	fn nonce(&self) -> ita_stf::Index {
		self.nonce
	}

	fn verify_signature(&self, _mrenclave: &[u8; 32], _shard: &ShardIdentifier) -> bool {
		log::error!("verify_signature shold not be used for TrustedCallAuthenticated");
		false
	}

	fn metric_name(&self) -> &'static str {
		self.call.metric_name()
	}
}

pub fn verify_tca_web3_authentication(
	signature: &LitentryMultiSignature,
	call: &TrustedCall,
	nonce: ParentchainIndex,
	mrenclave: &[u8; 32],
	shard: &ShardIdentifier,
) -> bool {
	let mut payload = call.encode();
	payload.append(&mut nonce.encode());
	payload.append(&mut mrenclave.encode());
	payload.append(&mut shard.encode());

	// The signature should be valid in either case:
	// 1. blake2_256(payload)
	// 2. Signature Prefix + blake2_256(payload)

	let hashed = blake2_256(&payload);

	let prettified_msg_hash = call.signature_message_prefix() + &hex_encode(&hashed);
	let prettified_msg_hash = prettified_msg_hash.as_bytes();

	// Most common signatures variants by clients are verified first (4 and 2).
	signature.verify(prettified_msg_hash, call.sender_identity())
		|| signature.verify(&hashed, call.sender_identity())
}

pub fn verify_tca_email_authentication(
	call: &TrustedCall,
	verification_code: VerificationCode,
) -> bool {
	let identity_hash = call.sender_identity().hash();
	let omni_account = extract_omni_account_from_call(call);
	match VerificationCodeStore::get(&omni_account, identity_hash) {
		Ok(Some(code)) => code == verification_code,
		_ => false,
	}
}

fn extract_omni_account_from_call(call: &TrustedCall) -> AccountId {
	let member_identity = call.sender_identity();
	if let Ok(Some(account_id)) = OmniAccountStore::get_omni_account(member_identity.hash()) {
		account_id
	} else {
		member_identity.to_omni_account()
	}
}
