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
#[allow(non_camel_case_types)]
pub enum NativeTask {
	request_auth_token(Identity),
	request_intent(Identity, Intent),
	create_account_store(Identity),
	add_account(Identity, Identity, ValidationData, bool, Option<Vec<OmniAccountPermission>>),
	remove_accounts(Identity, Vec<Identity>),
	publicize_account(Identity, Identity),
	set_permissions(Identity, Identity, Vec<OmniAccountPermission>),

	// pumpx specific
	request_pumpx_jwt(Identity),
}

impl NativeTaskTrait for NativeTask {
	fn sender(&self) -> &Identity {
		match self {
			Self::request_auth_token(sender, ..) => sender,
			Self::request_intent(sender, ..) => sender,
			Self::create_account_store(sender) => sender,
			Self::add_account(sender, ..) => sender,
			Self::remove_accounts(sender, ..) => sender,
			Self::publicize_account(sender, ..) => sender,
			Self::set_permissions(sender, ..) => sender,
			Self::request_pumpx_jwt(sender) => sender,
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
