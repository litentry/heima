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

use crate::authentication_utils::OAuth2Data;
use alloc::string::String;
use codec::{Decode, Encode};
use ita_stf::{LitentryMultiSignature, TrustedCall};
use itp_stf_primitives::traits::TrustedCallVerification;
use itp_types::parentchain::Index as ParentchainIndex;
use litentry_primitives::{Identity, ShardIdentifier};
use sp_core::{
	crypto::{AccountId32 as AccountId, UncheckedFrom},
	ed25519,
};

pub type VerificationCode = String;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum TCAuthentication {
	Web3(LitentryMultiSignature),
	Email(VerificationCode),
	OAuth2(OAuth2Data),
	AuthToken(String),
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
