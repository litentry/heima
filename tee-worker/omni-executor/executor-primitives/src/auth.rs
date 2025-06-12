use crate::{
	signature::{
		BitcoinSignature, EthereumSignature, HeimaMultiSignature, SolanaSignature,
		SubstrateSignature,
	},
	utils::hex::{decode_hex, ToHexPrefixed},
	OmniAccountAuthType,
};
use base58::{FromBase58, ToBase58};
use heima_primitives::{Address20, Address32, Address33, Identity, IdentityString};
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use tracing::error;

pub type VerificationCode = String;
type Email = String;
type JwtToken = String;

/// A serializable representation of User Identity for JSON interchange.
/// ```json
/// {
///  "type": "Email",
///  "value": "test@test.com",
/// }
/// ```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Encode, Decode)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", content = "value")]
pub enum UserId {
	Twitter(String),
	Discord(String),
	Github(String),
	Substrate(String), // hex-encoded
	Evm(String),       // hex-encoded
	Bitcoin(String),   // hex-encoded
	Solana(String),    // base58-encoded
	Email(String),
	Google(String),
}

impl TryFrom<UserId> for Identity {
	type Error = &'static str;

	fn try_from(value: UserId) -> Result<Self, Self::Error> {
		match value {
			UserId::Twitter(handle) => {
				Ok(Identity::Twitter(IdentityString::new(handle.as_bytes().to_vec())))
			},
			UserId::Discord(handle) => {
				Ok(Identity::Discord(IdentityString::new(handle.as_bytes().to_vec())))
			},
			UserId::Github(handle) => {
				Ok(Identity::Github(IdentityString::new(handle.as_bytes().to_vec())))
			},
			UserId::Substrate(hex_address) => {
				let bytes = decode_hex(&hex_address).map_err(|_| "Invalid hex encoding")?;
				let address =
					Address32::try_from(bytes.as_slice()).map_err(|_| "Invalid address")?;
				Ok(Identity::Substrate(address))
			},
			UserId::Evm(hex_address) => {
				let bytes = decode_hex(&hex_address).map_err(|_| "Invalid hex encoding")?;
				let address =
					Address20::try_from(bytes.as_slice()).map_err(|_| "Invalid address")?;
				Ok(Identity::Evm(address))
			},
			UserId::Bitcoin(hex_address) => {
				let bytes = decode_hex(&hex_address).map_err(|_| "Invalid hex encoding")?;
				let address =
					Address33::try_from(bytes.as_slice()).map_err(|_| "Invalid address")?;
				Ok(Identity::Bitcoin(address))
			},
			UserId::Solana(base58_address) => {
				let address: Address32 = base58_address
					.from_base58()
					.map_err(|_| "Invalid base58 encoding")?
					.as_slice()
					.try_into()
					.map_err(|_| "Invalid address")?;
				Ok(Identity::Solana(address))
			},
			UserId::Email(handle) => {
				Ok(Identity::Email(IdentityString::new(handle.as_bytes().to_vec())))
			},
			UserId::Google(handle) => {
				Ok(Identity::Google(IdentityString::new(handle.as_bytes().to_vec())))
			},
		}
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum OmniAuth {
	Web3(String, Identity, HeimaMultiSignature), // (client_id, Signer, Signature)
	Email(String, Email, VerificationCode),      // (client_id, Email, VerificationCode)
	AuthToken(JwtToken),
	OAuth2(Identity, OAuth2Data), // (Sender, OAuth2Data)
}

impl TryFrom<Identity> for UserId {
	type Error = ();

	fn try_from(value: Identity) -> Result<Self, Self::Error> {
		match value {
			Identity::Twitter(handle) => {
				Ok(UserId::Twitter(String::from_utf8(handle.inner.to_vec()).map_err(|_| ())?))
			},
			Identity::Discord(handle) => {
				Ok(UserId::Discord(String::from_utf8(handle.inner.to_vec()).map_err(|_| ())?))
			},
			Identity::Github(handle) => {
				Ok(UserId::Github(String::from_utf8(handle.inner.to_vec()).map_err(|_| ())?))
			},
			Identity::Substrate(address) => Ok(UserId::Substrate(address.to_hex())),
			Identity::Evm(address) => Ok(UserId::Evm(address.to_hex())),
			Identity::Bitcoin(address) => Ok(UserId::Bitcoin(address.to_hex())),
			Identity::Solana(address) => Ok(UserId::Solana(address.as_ref().to_base58())),
			Identity::Email(handle) => {
				Ok(UserId::Email(String::from_utf8(handle.inner.to_vec()).map_err(|_| ())?))
			},
			Identity::Google(handle) => {
				Ok(UserId::Google(String::from_utf8(handle.inner.to_vec()).map_err(|_| ())?))
			},
			Identity::Pumpx(_) => {
				// TODO: should we remove this identity type?
				error!("Pumpx identity is not supported in UserId conversion");
				Err(())
			},
		}
	}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", content = "value")]
pub enum UserAuth {
	Email(VerificationCode),
	AuthToken(JwtToken),
	Substrate(SubstrateSignature),
	Evm(EthereumSignature),
	Bitcoin(BitcoinSignature),
	Solana(SolanaSignature),
	OAuth2(OAuth2Data),
}

impl From<OmniAuth> for OmniAccountAuthType {
	fn from(value: OmniAuth) -> Self {
		match value {
			OmniAuth::Web3(..) => Self::Web3,
			OmniAuth::Email(..) => Self::Email,
			OmniAuth::OAuth2(..) => Self::OAuth2,
			OmniAuth::AuthToken(..) => Self::AuthToken,
		}
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OAuth2Provider {
	Google,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OAuth2Data {
	pub provider: OAuth2Provider,
	pub code: String,
	pub state: String,
	pub redirect_uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", content = "value")]
pub enum ClientAuth {
	Wildmeta { google_code: Option<String>, invite_code: Option<String> },
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_user_id_to_identity() {
		let json = r#"{"type": "email", "value": "test@test.com"}"#;
		let deserialized: UserId = serde_json::from_str(json).unwrap();
		let identity = Identity::try_from(deserialized).unwrap();

		assert_eq!(identity, Identity::Email(IdentityString::new(b"test@test.com".to_vec())));
	}

	#[test]
	fn test_wildmeta_client_auth_serde() {
		let json =
			r#"{"type": "wildmeta", "value": { "google_code": "123", "invite_code": "456" }}"#;
		let deserialized: ClientAuth = serde_json::from_str(json).unwrap();
		let expected = ClientAuth::Wildmeta {
			google_code: Some("123".to_string()),
			invite_code: Some("456".to_string()),
		};
		assert_eq!(deserialized, expected);
	}
}
