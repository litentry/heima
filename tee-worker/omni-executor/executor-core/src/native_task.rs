use crate::types::SerializablePackedUserOperation;
use executor_primitives::{AccountId, ChainId, Identity, Intent, IntentId, Nonce, OmniAuth};
use parity_scale_codec::{Codec, Decode, Encode};
use std::fmt::Debug;
use std::vec::Vec;
use uuid::Uuid;

pub trait NativeTaskTrait: Codec {
	fn signature_message_prefix(&self) -> String {
		"Token: ".to_string()
	}

	fn require_auth(&self) -> bool;

	fn require_encrypt(&self) -> bool;
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct NativeTaskWrapper<T: NativeTaskTrait> {
	pub id: String,
	pub task: T,
	pub nonce: Option<Nonce>,
	pub auth: Option<OmniAuth>,
	pub client_id: String,
}

impl<T: NativeTaskTrait + Debug> NativeTaskWrapper<T> {
	pub fn new(task: T, nonce: Option<Nonce>, auth: Option<OmniAuth>, client_id: String) -> Self {
		let id: String = Uuid::new_v4().into();
		Self { id, task, nonce, auth, client_id }
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum NativeTask {
	RequestAuthToken(Identity),
	RequestIntent(AccountId, IntentId, Box<Intent>),
	SubmitUserOp(AccountId, Vec<SerializablePackedUserOperation>, ChainId, u32),
	EstimateUserOpGas(AccountId, SerializablePackedUserOperation, ChainId, u32),
}

impl NativeTaskTrait for NativeTask {
	fn require_auth(&self) -> bool {
		// currently all tasks require auth
		true
	}

	fn require_encrypt(&self) -> bool {
		false
	}
}
