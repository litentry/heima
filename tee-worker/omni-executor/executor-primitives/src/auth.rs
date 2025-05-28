use crate::{signature::HeimaMultiSignature, utils::hex::decode_hex, OmniAccountAuthType};
use base58::FromBase58;
use heima_primitives::{Address20, Address32, Address33, Identity, IdentityString};
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub type VerificationCode = String;
type Email = String;
type JwtToken = String;

/// A serializable representation of Identity for JSON interchange.
/// ```json
/// {
///  "Email": "test@test.com",
/// }
/// ```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum IdentitySerde {
	Twitter(String),
	Discord(String),
	Github(String),
	Substrate(String), // hex-encoded
	Evm(String),       // hex-encoded
	Bitcoin(String),   // hex-encoded
	Solana(String),    // base58-encoded
	Email(String),
	Google(String),
	Pumpx(String),
}

impl TryFrom<IdentitySerde> for Identity {
	type Error = &'static str;

	fn try_from(value: IdentitySerde) -> Result<Self, Self::Error> {
		match value {
			IdentitySerde::Twitter(handle) => {
				Ok(Identity::Twitter(IdentityString::new(handle.as_bytes().to_vec())))
			},
			IdentitySerde::Discord(handle) => {
				Ok(Identity::Discord(IdentityString::new(handle.as_bytes().to_vec())))
			},
			IdentitySerde::Github(handle) => {
				Ok(Identity::Github(IdentityString::new(handle.as_bytes().to_vec())))
			},
			IdentitySerde::Substrate(hex_address) => {
				let bytes = decode_hex(&hex_address).map_err(|_| "Invalid hex encoding")?;
				let address =
					Address32::try_from(bytes.as_slice()).map_err(|_| "Invalid address")?;
				Ok(Identity::Substrate(address))
			},
			IdentitySerde::Evm(hex_address) => {
				let bytes = decode_hex(&hex_address).map_err(|_| "Invalid hex encoding")?;
				let address =
					Address20::try_from(bytes.as_slice()).map_err(|_| "Invalid address")?;
				Ok(Identity::Evm(address))
			},
			IdentitySerde::Bitcoin(hex_address) => {
				let bytes = decode_hex(&hex_address).map_err(|_| "Invalid hex encoding")?;
				let address =
					Address33::try_from(bytes.as_slice()).map_err(|_| "Invalid address")?;
				Ok(Identity::Bitcoin(address))
			},
			IdentitySerde::Solana(base58_address) => {
				let address: Address32 = base58_address
					.from_base58()
					.map_err(|_| "Invalid base58 encoding")?
					.as_slice()
					.try_into()
					.map_err(|_| "Invalid address")?;
				Ok(Identity::Solana(address))
			},
			IdentitySerde::Email(handle) => {
				Ok(Identity::Email(IdentityString::new(handle.as_bytes().to_vec())))
			},
			IdentitySerde::Google(handle) => {
				Ok(Identity::Google(IdentityString::new(handle.as_bytes().to_vec())))
			},
			IdentitySerde::Pumpx(handle) => {
				Ok(Identity::Pumpx(IdentityString::new(handle.as_bytes().to_vec())))
			},
		}
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum OmniAuth {
	Web3(Identity, HeimaMultiSignature), // (Signer, Signature)
	Email(Email, VerificationCode),
	AuthToken(JwtToken),
	OAuth2(Identity, OAuth2Data), // (Sender, OAuth2Data)
}

impl TryFrom<OmniAuthSerde> for OmniAuth {
	type Error = &'static str;

	fn try_from(value: OmniAuthSerde) -> Result<Self, Self::Error> {
		match value {
			OmniAuthSerde::Web3(id, sig) => Ok(OmniAuth::Web3(id.try_into()?, sig)),
			OmniAuthSerde::Email(email, code) => Ok(OmniAuth::Email(email, code)),
			OmniAuthSerde::AuthToken(account, token) => Ok(OmniAuth::AuthToken(account, token)),
			OmniAuthSerde::OAuth2(id, data) => Ok(OmniAuth::OAuth2(id.try_into()?, data)),
		}
	}
}

#[derive(Deserialize, Debug)]
pub enum OmniAuthSerde {
	Web3(IdentitySerde, HeimaMultiSignature),
	Email(Email, VerificationCode),
	AuthToken(JwtToken),
	OAuth2(IdentitySerde, OAuth2Data),
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

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum OAuth2Provider {
	Google,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct OAuth2Data {
	pub provider: OAuth2Provider,
	pub code: String,
	pub state: String,
	pub redirect_uri: String,
}

#[derive(Encode, Decode, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct UserInfo {
	pub id: IdentitySerde,
	pub auth: OmniAuthSerde,
}

#[cfg(test)]
mod tests {
	use super::*;
	use executor_crypto::ed25519;

	#[test]
	fn test_identity_serde() {
		let json = r#"{"Twitter":"handle"}"#;
		let deserialized: IdentitySerde = serde_json::from_str(json).unwrap();
		let identity = Identity::try_from(deserialized).unwrap();

		assert_eq!(identity, Identity::Twitter(IdentityString::new(b"handle".to_vec())));
	}

	#[test]
	fn test_omni_auth_serde() {
		let raw_signature: [u8; 64] = [
			62, 25, 148, 186, 53, 137, 248, 174, 149, 187, 225, 24, 186, 48, 24, 109, 100, 27, 149,
			196, 66, 5, 222, 140, 22, 16, 136, 239, 154, 22, 133, 96, 79, 2, 180, 106, 150, 112,
			116, 11, 6, 35, 32, 4, 145, 240, 54, 130, 206, 193, 200, 57, 241, 112, 35, 122, 226,
			97, 174, 231, 221, 13, 98, 2,
		];
		let address = "E9SegbpSr21FPLbUhoTNH6C2ja7KDkptybqSaT84wMH6";
		let json = format!(
			r#"{{
		       "Web3":[
		            {{"Solana": "{}"}},
		            {{"Ed25519":"{}"}}
		        ]
		    }}"#,
			address,
			hex::encode(raw_signature)
		);
		let deserialized: OmniAuthSerde = serde_json::from_str(&json).unwrap();
		let omni_auth = OmniAuth::try_from(deserialized).unwrap();

		assert_eq!(
			omni_auth,
			OmniAuth::Web3(
				Identity::Solana(Address32::try_from(address).unwrap()),
				HeimaMultiSignature::Ed25519(ed25519::Signature::from_raw(raw_signature))
			)
		);
	}
}
