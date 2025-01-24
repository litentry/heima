use parity_scale_codec::{Decode, Encode};

#[derive(Encode, Decode, Debug, PartialEq, Eq)]
pub enum NativeCallOk {
	AuthToken(String),
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum NativeCallError {
	UnexpectedCall(String),
	UnauthorizedSender,
	AuthTokenCreationFailed,
}
