use parentchain_primitives::Identity;
use parity_scale_codec::{Decode, Encode};

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum NativeCall {
	// TODO: define the actual calls
	noop(Identity),
}

impl NativeCall {
	pub fn sender_identity(&self) -> &Identity {
		match self {
			NativeCall::noop(sender_identity) => sender_identity,
		}
	}
}
