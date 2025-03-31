use crate::{signature::HeimaMultiSignature, OmniAccountAuthType};

use parity_scale_codec::{Decode, Encode};

pub type VerificationCode = String;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum OmniAuth {
	Web3(HeimaMultiSignature),
	Email(VerificationCode),
	AuthToken(String),
	OAuth2(OAuth2Data),
}

impl From<OmniAuth> for OmniAccountAuthType {
	fn from(value: OmniAuth) -> Self {
		match value {
			OmniAuth::Web3(_) => Self::Web3,
			OmniAuth::Email(_) => Self::Email,
			OmniAuth::OAuth2(_) => Self::OAuth2,
			OmniAuth::AuthToken(_) => Self::AuthToken,
		}
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum OAuth2Provider {
	Google,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct OAuth2Data {
	pub provider: OAuth2Provider,
	pub code: String,
	pub state: String,
	pub redirect_uri: String,
}
