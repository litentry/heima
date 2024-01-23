// Copyright 2020-2023 Trust Computing GmbH.
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

#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;
#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

mod aes;
mod aes_request;
mod bitcoin_address;
mod bitcoin_signature;
mod ethereum_signature;
mod validation_data;

pub use aes::*;
pub use aes_request::*;
pub use bitcoin_address::*;
pub use bitcoin_signature::*;
pub use ethereum_signature::*;
use sp_std::{boxed::Box, fmt::Debug, vec::Vec};
pub use validation_data::*;

use bitcoin::sign_message::{signed_msg_hash, MessageSignature};
use codec::{Decode, Encode, MaxEncodedLen};
use itp_sgx_crypto::ShieldingCryptoDecrypt;
use itp_utils::hex::hex_encode;
use log::error;
pub use parentchain_primitives::{
	all_bitcoin_web3networks, all_evm_web3networks, all_substrate_web3networks, all_web3networks,
	identity::*, AccountId as ParentchainAccountId, AchainableAmount, AchainableAmountHolding,
	AchainableAmountToken, AchainableAmounts, AchainableBasic, AchainableBetweenPercents,
	AchainableClassOfYear, AchainableDate, AchainableDateInterval, AchainableDatePercent,
	AchainableMirror, AchainableParams, AchainableToken, AmountHoldingTimeType, Assertion,
	Balance as ParentchainBalance, BlockNumber as ParentchainBlockNumber, BnbDigitDomainType,
	BoundedWeb3Network, ContestType, EVMTokenType, ErrorDetail, ErrorString,
	GenericDiscordRoleType, Hash as ParentchainHash, Header as ParentchainHeader, IMPError,
	Index as ParentchainIndex, IntoErrorDetail, OneBlockCourseType, ParameterString,
	SchemaContentString, SchemaIdString, Signature as ParentchainSignature, SoraQuizType,
	VCMPError, VIP3MembershipCardLevel, Web3Network, Web3TokenType, MINUTES,
};
use scale_info::TypeInfo;
use sp_core::{ecdsa, ed25519, sr25519, ByteArray};
use sp_io::{
	crypto::secp256k1_ecdsa_recover,
	hashing::{blake2_256, keccak_256},
};
use sp_runtime::traits::Verify;
use std::string::{String, ToString};
pub use teerex_primitives::{decl_rsa_request, ShardIdentifier, SidechainBlockNumber};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub const LITENTRY_PRETTIFIED_MESSAGE_PREFIX: &str = "Litentry authorization token: ";

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum LitentryMultiSignature {
	/// An Ed25519 signature.
	#[codec(index = 0)]
	Ed25519(ed25519::Signature),
	/// An Sr25519 signature.
	#[codec(index = 1)]
	Sr25519(sr25519::Signature),
	/// An ECDSA/SECP256k1 signature.
	#[codec(index = 2)]
	Ecdsa(ecdsa::Signature),
	/// An ECDSA/keccak256 signature. An Ethereum signature. hash message with keccak256
	#[codec(index = 3)]
	Ethereum(EthereumSignature),
	/// Same as above, but the payload bytes are prepended with a readable prefix and `0x`
	#[codec(index = 4)]
	EthereumPrettified(EthereumSignature),
	/// Bitcoin signed message, a hex-encoded string of original &[u8] message, without `0x` prefix
	#[codec(index = 5)]
	Bitcoin(BitcoinSignature),
	/// Same as above, but the payload bytes are prepended with a readable prefix and `0x`
	#[codec(index = 6)]
	BitcoinPrettified(BitcoinSignature),
}

impl LitentryMultiSignature {
	pub fn verify(&self, msg: &[u8], signer: &Identity) -> bool {
		match signer {
			Identity::Substrate(address) =>
				self.verify_substrate(substrate_wrap(msg).as_slice(), address)
					|| self.verify_substrate(msg, address),
			Identity::Evm(address) => self.verify_evm(msg, address),
			Identity::Bitcoin(address) => self.verify_bitcoin(msg, address),
			_ => false,
		}
	}

	fn verify_substrate(&self, msg: &[u8], signer: &Address32) -> bool {
		match (self, signer) {
			(Self::Ed25519(ref sig), who) => match ed25519::Public::from_slice(who.as_ref()) {
				Ok(signer) => sig.verify(msg, &signer),
				Err(()) => false,
			},
			(Self::Sr25519(ref sig), who) => match sr25519::Public::from_slice(who.as_ref()) {
				Ok(signer) => sig.verify(msg, &signer),
				Err(()) => false,
			},
			(Self::Ecdsa(ref sig), who) => {
				let m = blake2_256(msg);
				match sp_io::crypto::secp256k1_ecdsa_recover_compressed(sig.as_ref(), &m) {
					Ok(pubkey) =>
						&blake2_256(pubkey.as_ref()) == <dyn AsRef<[u8; 32]>>::as_ref(who),
					_ => false,
				}
			},
			_ => false,
		}
	}

	fn verify_evm(&self, msg: &[u8], signer: &Address20) -> bool {
		match self {
			Self::Ethereum(ref sig) =>
				return verify_evm_signature(evm_eip191_wrap(msg).as_slice(), sig, signer)
					|| verify_evm_signature(msg, sig, signer),
			Self::EthereumPrettified(ref sig) => {
				let prettified_msg =
					LITENTRY_PRETTIFIED_MESSAGE_PREFIX.to_string() + &hex_encode(msg);
				let msg = prettified_msg.as_bytes();
				return verify_evm_signature(evm_eip191_wrap(msg).as_slice(), sig, signer)
					|| verify_evm_signature(msg, sig, signer)
			},
			_ => false,
		}
	}

	fn verify_bitcoin(&self, msg: &[u8], signer: &Address33) -> bool {
		match self {
			Self::Bitcoin(ref sig) =>
				verify_bitcoin_signature(hex::encode(msg).as_str(), sig, signer),
			Self::BitcoinPrettified(ref sig) => {
				let prettified_msg =
					LITENTRY_PRETTIFIED_MESSAGE_PREFIX.to_string() + &hex_encode(msg);
				verify_bitcoin_signature(prettified_msg.as_str(), sig, signer)
			},
			_ => false,
		}
	}
}

pub fn verify_evm_signature(msg: &[u8], sig: &EthereumSignature, who: &Address20) -> bool {
	let digest = keccak_256(msg);
	return match recover_evm_address(&digest, sig.as_ref()) {
		Ok(recovered_evm_address) => recovered_evm_address == who.as_ref().as_slice(),
		Err(_e) => {
			error!("Could not verify evm signature msg: {:?}, signer {:?}", msg, who);
			false
		},
	}
}

pub fn verify_bitcoin_signature(msg: &str, sig: &BitcoinSignature, who: &Address33) -> bool {
	if let Ok(msg_sig) = MessageSignature::from_slice(sig.as_ref()) {
		let msg_hash = signed_msg_hash(msg);
		let secp = secp256k1::Secp256k1::new();
		return match msg_sig.recover_pubkey(&secp, msg_hash) {
			Ok(recovered_pub_key) => &recovered_pub_key.inner.serialize() == who.as_ref(),
			Err(_) => {
				error!("Could not recover pubkey from bitcoin msg: {:?}, signer {:?}", msg, who);
				false
			},
		}
	}

	false
}

impl From<ed25519::Signature> for LitentryMultiSignature {
	fn from(x: ed25519::Signature) -> Self {
		Self::Ed25519(x)
	}
}

impl From<sr25519::Signature> for LitentryMultiSignature {
	fn from(x: sr25519::Signature) -> Self {
		Self::Sr25519(x)
	}
}

impl From<ecdsa::Signature> for LitentryMultiSignature {
	fn from(x: ecdsa::Signature) -> Self {
		Self::Ecdsa(x)
	}
}

pub fn recover_evm_address(
	msg: &[u8; 32],
	sig: &[u8; 65],
) -> Result<[u8; 20], sp_io::EcdsaVerifyError> {
	let pubkey = secp256k1_ecdsa_recover(sig, msg)?;
	let hashed_pk = keccak_256(&pubkey);

	let mut addr = [0u8; 20];
	addr[..20].copy_from_slice(&hashed_pk[12..32]);
	Ok(addr)
}

// see https://github.com/litentry/litentry-parachain/issues/1137
fn substrate_wrap(msg: &[u8]) -> Vec<u8> {
	["<Bytes>".as_bytes(), msg, "</Bytes>".as_bytes()].concat()
}

// see https://github.com/litentry/litentry-parachain/issues/1970
fn evm_eip191_wrap(msg: &[u8]) -> Vec<u8> {
	["\x19Ethereum Signed Message:\n".as_bytes(), msg.len().to_string().as_bytes(), msg].concat()
}

pub type IdentityNetworkTuple = (Identity, Vec<Web3Network>);

// Represent a request that can be decrypted by the enclave
// Both itp_types::RsaRequest and AesRequest should impelement this
pub trait DecryptableRequest {
	type Error;
	// the shard getter
	fn shard(&self) -> ShardIdentifier;
	// the raw payload - AFAICT only used in mock
	fn payload(&self) -> &[u8];
	// how to decrypt the payload
	fn decrypt<T: Debug>(
		&mut self,
		enclave_shielding_key: Box<dyn ShieldingCryptoDecrypt<Error = T>>,
	) -> Result<Vec<u8>, Self::Error>;
}

pub struct BroadcastedRequest {
	pub id: String,
	pub payload: String,
	pub rpc_method: String,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn verify_bitcoin_signature_works() {
		// generated by unisat-wallet API: https://docs.unisat.io/dev/unisat-developer-service/unisat-wallet
		let msg: Vec<u8> = vec![
			3, 93, 250, 112, 216, 101, 89, 57, 83, 88, 100, 252, 203, 15, 64, 127, 138, 37, 2, 40,
			147, 95, 245, 27, 97, 202, 62, 205, 151, 0, 175, 177,
		];
		let pubkey: Vec<u8> = vec![
			3, 93, 250, 112, 216, 101, 89, 57, 83, 88, 100, 252, 203, 15, 64, 127, 138, 37, 2, 40,
			147, 95, 245, 27, 97, 202, 62, 205, 151, 0, 175, 177, 216,
		];
		let sig: Vec<u8> = base64::decode("G2LhyYzWT2o8UoBsuhJsqFgwm3tlE0cW4aseCXKqVuNATk6K/uEHlPzDFmtlMADywDHl5vLCWcNpwmQLD7n/yvc=").unwrap();

		let pubkey_ref: &[u8] = pubkey.as_ref();
		let sig_ref: &[u8] = sig.as_ref();
		assert!(verify_bitcoin_signature(
			hex::encode(msg).as_str(),
			&sig_ref.try_into().unwrap(),
			&pubkey_ref.try_into().unwrap()
		));
	}
}
