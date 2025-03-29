use executor_primitives::{
	Identity, Intent, Nonce, OmniAccountPermission, OmniAuth, ValidationData,
};
use parity_scale_codec::{Codec, Decode, Encode};
use std::vec::Vec;

pub trait NativeTaskTrait: Codec {
	fn sender(&self) -> &Identity;

	fn signature_message_prefix(&self) -> String {
		"Token: ".to_string()
	}

	fn require_auth(&self) -> bool;

	fn require_encrypt(&self) -> bool;
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct NativeTaskWrapper<T: NativeTaskTrait> {
	pub task: T,
	pub nonce: Option<Nonce>,
	pub auth: Option<OmniAuth>,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum NativeTask {
	RequestAuthToken(Identity),
	RequestIntent(Identity, Intent),
	CreateAccountStore(Identity),
	AddAccount(Identity, Identity, ValidationData, bool, Option<Vec<OmniAccountPermission>>),
	RemoveAccounts(Identity, Vec<Identity>),
	PublicizeAccount(Identity, Identity),
	SetPermissions(Identity, Identity, Vec<OmniAccountPermission>),

	// pumpx specific, starting from index 20
	#[codec(index = 20)]
	PumpxRequestJwt(Identity),
}

impl NativeTaskTrait for NativeTask {
	fn sender(&self) -> &Identity {
		match self {
			Self::RequestAuthToken(sender, ..) => sender,
			Self::RequestIntent(sender, ..) => sender,
			Self::CreateAccountStore(sender) => sender,
			Self::AddAccount(sender, ..) => sender,
			Self::RemoveAccounts(sender, ..) => sender,
			Self::PublicizeAccount(sender, ..) => sender,
			Self::SetPermissions(sender, ..) => sender,
			Self::PumpxRequestJwt(sender) => sender,
		}
	}

	fn require_auth(&self) -> bool {
		// currently all tasks require auth
		true
	}

	fn require_encrypt(&self) -> bool {
		// encryption is not mandatory for any of these tasks
		// TODO: export_wallet will require encryption
		false
	}
}
