use executor_primitives::Identity;
use heima_authentication::auth_token::AuthOptions;
use parity_scale_codec::{Decode, Encode};

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum NativeCall {
	request_auth_token(Identity, AuthOptions),
	noop(Identity),
}

impl NativeCall {
	pub fn sender_identity(&self) -> &Identity {
		match self {
			NativeCall::request_auth_token(sender_identity, ..) => sender_identity,
			NativeCall::noop(sender_identity) => sender_identity,
		}
	}

	pub fn signature_message_prefix(&self) -> String {
		// TODO: update this when adding request_batch_vc variant
		"Token: ".to_string()
	}
}
