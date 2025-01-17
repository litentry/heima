use crate::signature::{BitcoinSignature, EthereumSignature};
use crypto::{ecdsa, ed25519, sr25519};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

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
