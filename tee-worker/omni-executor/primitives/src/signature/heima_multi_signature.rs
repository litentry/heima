use crate::signature::{BitcoinSignature, EthereumSignature};
use bitcoin::sign_message::{signed_msg_hash, MessageSignature};
use executor_crypto::{
	ecdsa, ed25519,
	hashing::{blake2_256, keccak_256},
	secp256k1::{
		secp256k1_ecdsa_recover, secp256k1_ecdsa_recover_compressed, EcdsaVerifyError, Secp256k1,
	},
	sr25519, ByteArray, PairTrait,
};
use heima_primitives::{Address20, Address32, Address33, Identity};
use log::error;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(
	Encode, Decode, Clone, Debug, PartialEq, Eq, TypeInfo, MaxEncodedLen, Serialize, Deserialize,
)]
pub enum HeimaMultiSignature {
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
	/// Bitcoin signed message, a hex-encoded string of original &[u8] message, without `0x` prefix
	#[codec(index = 4)]
	Bitcoin(BitcoinSignature),
}

impl HeimaMultiSignature {
	pub fn verify(&self, msg: &[u8], signer: &Identity) -> bool {
		match signer {
			Identity::Substrate(address) => {
				self.verify_substrate(substrate_wrap(msg).as_slice(), address)
					|| self.verify_substrate(msg, address)
			},
			Identity::Evm(address) => self.verify_evm(msg, address),
			Identity::Bitcoin(address) => self.verify_bitcoin(msg, address),
			Identity::Solana(address) => self.verify_solana(msg, address),
			_ => false,
		}
	}

	fn verify_substrate(&self, msg: &[u8], signer: &Address32) -> bool {
		match (self, signer) {
			(Self::Ed25519(ref sig), who) => match ed25519::Public::from_slice(who.as_ref()) {
				Ok(signer) => ed25519::Pair::verify(sig, msg, &signer),
				Err(()) => false,
			},
			(Self::Sr25519(ref sig), who) => match sr25519::Public::from_slice(who.as_ref()) {
				Ok(signer) => sr25519::Pair::verify(sig, msg, &signer),
				Err(()) => false,
			},
			(Self::Ecdsa(ref sig), who) => {
				let m = blake2_256(msg);
				match secp256k1_ecdsa_recover_compressed(sig.as_ref(), &m) {
					Ok(pubkey) => {
						&blake2_256(pubkey.as_ref()) == <dyn AsRef<[u8; 32]>>::as_ref(who)
					},
					_ => false,
				}
			},
			_ => false,
		}
	}

	fn verify_evm(&self, msg: &[u8], signer: &Address20) -> bool {
		match self {
			Self::Ethereum(ref sig) => {
				return verify_evm_signature(evm_eip191_wrap(msg).as_slice(), sig, signer)
					|| verify_evm_signature(msg, sig, signer)
			},
			_ => false,
		}
	}

	fn verify_bitcoin(&self, msg: &[u8], signer: &Address33) -> bool {
		match self {
			Self::Bitcoin(ref sig) => {
				verify_bitcoin_signature(hex::encode(msg).as_str(), sig, signer)
					|| match std::str::from_utf8(msg) {
						Err(_) => false,
						Ok(prettified) => verify_bitcoin_signature(prettified, sig, signer),
					}
			},
			_ => false,
		}
	}

	// https://github.com/solana-labs/solana/blob/master/docs/src/proposals/off-chain-message-signing.md
	fn verify_solana(&self, msg: &[u8], signer: &Address32) -> bool {
		match (self, signer) {
			(Self::Ed25519(ref sig), who) => match ed25519::Public::from_slice(who.as_ref()) {
				Ok(signer) => ed25519::Pair::verify(sig, msg, &signer),
				Err(()) => false,
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
	};
}

pub fn verify_bitcoin_signature(msg: &str, sig: &BitcoinSignature, who: &Address33) -> bool {
	if let Ok(msg_sig) = MessageSignature::from_slice(sig.as_ref()) {
		let msg_hash = signed_msg_hash(msg);
		let secp = Secp256k1::new();
		return match msg_sig.recover_pubkey(&secp, msg_hash) {
			Ok(recovered_pub_key) => &recovered_pub_key.inner.serialize() == who.as_ref(),
			Err(_) => {
				error!("Could not recover pubkey from bitcoin msg: {:?}, signer {:?}", msg, who);
				false
			},
		};
	}

	false
}

impl From<ed25519::Signature> for HeimaMultiSignature {
	fn from(x: ed25519::Signature) -> Self {
		Self::Ed25519(x)
	}
}

impl From<sr25519::Signature> for HeimaMultiSignature {
	fn from(x: sr25519::Signature) -> Self {
		Self::Sr25519(x)
	}
}

impl From<ecdsa::Signature> for HeimaMultiSignature {
	fn from(x: ecdsa::Signature) -> Self {
		Self::Ecdsa(x)
	}
}

pub fn recover_evm_address(msg: &[u8; 32], sig: &[u8; 65]) -> Result<[u8; 20], EcdsaVerifyError> {
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

#[cfg(test)]
mod tests {
	use super::*;
	use base64::{engine::general_purpose::STANDARD, Engine};

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
		let sig: Vec<u8> = STANDARD.decode("G2LhyYzWT2o8UoBsuhJsqFgwm3tlE0cW4aseCXKqVuNATk6K/uEHlPzDFmtlMADywDHl5vLCWcNpwmQLD7n/yvc=").unwrap();

		let pubkey_ref: &[u8] = pubkey.as_ref();
		let sig_ref: &[u8] = sig.as_ref();
		assert!(verify_bitcoin_signature(
			hex::encode(msg).as_str(),
			&sig_ref.try_into().unwrap(),
			&pubkey_ref.try_into().unwrap()
		));
	}

	#[test]
	fn verify_solana_signature_works() {
		let signer =
			Identity::from_did("did:litentry:solana:E9SegbpSr21FPLbUhoTNH6C2ja7KDkptybqSaT84wMH6")
				.unwrap();
		let signature: [u8; 64] = [
			62, 25, 148, 186, 53, 137, 248, 174, 149, 187, 225, 24, 186, 48, 24, 109, 100, 27, 149,
			196, 66, 5, 222, 140, 22, 16, 136, 239, 154, 22, 133, 96, 79, 2, 180, 106, 150, 112,
			116, 11, 6, 35, 32, 4, 145, 240, 54, 130, 206, 193, 200, 57, 241, 112, 35, 122, 226,
			97, 174, 231, 221, 13, 98, 2,
		];
		let result = HeimaMultiSignature::Ed25519(ed25519::Signature::from_raw(signature))
			.verify(b"test message", &signer);
		assert_eq!(result, true);
	}
}
