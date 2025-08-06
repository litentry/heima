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
	Pumpx(String),
	Email(String),
	Twitter(String),
	Discord(String),
	Github(String),
	Substrate(String), // hex-encoded
	Evm(String),       // hex-encoded
	Bitcoin(String),   // hex-encoded
	Solana(String),    // base58-encoded
	Google(String),
	Passkey(String), // unique user_id, even for multiple credential_id
}

impl TryFrom<UserId> for Identity {
	type Error = &'static str;

	fn try_from(value: UserId) -> Result<Self, Self::Error> {
		match value {
			UserId::Pumpx(handle) => {
				Ok(Identity::Pumpx(IdentityString::new(handle.as_bytes().to_vec())))
			},
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
			UserId::Passkey(handle) => {
				Ok(Identity::Passkey(IdentityString::new(handle.as_bytes().to_vec())))
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
	Passkey(PasskeyData),
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
			Identity::Pumpx(handle) => {
				Ok(UserId::Pumpx(String::from_utf8(handle.inner.to_vec()).map_err(|_| ())?))
			},
			Identity::Passkey(handle) => {
				Ok(UserId::Passkey(String::from_utf8(handle.inner.to_vec()).map_err(|_| ())?))
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
	Passkey(PasskeyData),
}

impl From<OmniAuth> for OmniAccountAuthType {
	fn from(value: OmniAuth) -> Self {
		match value {
			OmniAuth::Web3(..) => Self::Web3,
			OmniAuth::Email(..) => Self::Email,
			OmniAuth::OAuth2(..) => Self::OAuth2,
			OmniAuth::AuthToken(..) => Self::AuthToken,
			OmniAuth::Passkey(..) => Self::Passkey,
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

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasskeyData {
	pub user_id: String,
	pub credential_id: String,
	pub pubkey: String, // uncompressed, compressed, or COSE-encoded - have to figure out what authenticator sends
	pub signature: String, // raw 64 byte, or DER encoded - have to figure out what authenticator sends
	pub auth_data: String,
	pub client_data_json: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", content = "value")]
pub enum ClientAuth {
	Wildmeta {
		google_code: Option<String>,
		invite_code: Option<String>,
	},
	WildmetaHl {
		agent_address: String,
		business_json: String,
		main_address: String,
		signature: String,
		login_type: u32,
	},
}

/// Convert UserAuth + UserId + client_id into OmniAuth
pub fn to_omni_auth(
	user_auth: &UserAuth,
	user_id: &UserId,
	client_id: &str,
) -> Result<OmniAuth, &'static str> {
	let omni_auth = match user_auth {
		UserAuth::Email(code) => {
			let UserId::Email(email) = user_id else {
				return Err("User ID must be an email for Email authentication");
			};
			OmniAuth::Email(client_id.to_string(), email.clone(), code.clone())
		},
		UserAuth::Substrate(signature) => {
			let identity =
				Identity::try_from(user_id.clone()).map_err(|_| "Invalid user ID format")?;
			if !identity.is_substrate() {
				return Err("User ID must be a Substrate identity for Substrate authentication");
			}
			OmniAuth::Web3(client_id.to_string(), identity, signature.clone().into())
		},
		UserAuth::Evm(signature) => {
			let identity =
				Identity::try_from(user_id.clone()).map_err(|_| "Invalid user ID format")?;
			if !identity.is_evm() {
				return Err("User ID must be an EVM identity for EVM authentication");
			}
			OmniAuth::Web3(client_id.to_string(), identity, signature.clone().into())
		},
		UserAuth::Solana(signature) => {
			let identity =
				Identity::try_from(user_id.clone()).map_err(|_| "Invalid user ID format")?;
			if !identity.is_solana() {
				return Err("User ID must be a Solana identity for Solana authentication");
			}
			OmniAuth::Web3(client_id.to_string(), identity, signature.clone().into())
		},
		UserAuth::Bitcoin(signature) => {
			let identity =
				Identity::try_from(user_id.clone()).map_err(|_| "Invalid user ID format")?;
			if !identity.is_bitcoin() {
				return Err("User ID must be a Bitcoin identity for Bitcoin authentication");
			}
			OmniAuth::Web3(client_id.to_string(), identity, signature.clone().into())
		},
		UserAuth::AuthToken(token) => OmniAuth::AuthToken(token.clone()),
		UserAuth::OAuth2(data) => {
			let identity =
				Identity::try_from(user_id.clone()).map_err(|_| "Invalid user ID format")?;
			OmniAuth::OAuth2(identity, data.clone())
		},
		UserAuth::Passkey(data) => OmniAuth::Passkey(data.clone()),
	};

	Ok(omni_auth)
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

	#[test]
	fn test_to_omni_auth_email() {
		let user_auth = UserAuth::Email("123456".to_string());
		let user_id = UserId::Email("test@test.com".to_string());
		let client_id = "test_client";

		let result = to_omni_auth(&user_auth, &user_id, client_id).unwrap();
		match result {
			OmniAuth::Email(client, email, code) => {
				assert_eq!(client, "test_client");
				assert_eq!(email, "test@test.com");
				assert_eq!(code, "123456");
			},
			_ => panic!("Expected Email auth"),
		}
	}
}
