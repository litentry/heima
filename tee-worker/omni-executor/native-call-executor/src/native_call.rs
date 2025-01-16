use parity_scale_codec::{Decode, Encode};

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum NativeCall {
	// TODO: define the actual calls
	noop,
}
