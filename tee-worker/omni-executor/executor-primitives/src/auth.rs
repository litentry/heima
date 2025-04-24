use crate::{signature::HeimaMultiSignature, OmniAccountAuthType};
use heima_primitives::Identity;
use parity_scale_codec::{Decode, Encode};

pub type VerificationCode = String;
type Email = String;
type OmniAccount = String;
type JwtToken = String;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum OmniAuth {
	Web3(Identity, HeimaMultiSignature), // (Signer, Signature)
	Email(Email, VerificationCode),
	AuthToken(OmniAccount, JwtToken),
	OAuth2(Identity, OAuth2Data), // (Sender, OAuth2Data)
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
