use executor_primitives::{intent::Intent, Identity, OmniAccountPermission, ValidationData};
use heima_authentication::auth_token::AuthOptions;
use parity_scale_codec::{Decode, Encode};
use std::vec::Vec;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum NativeCall {
	request_auth_token(Identity, AuthOptions),
	request_intent(Identity, Intent),
	create_account_store(Identity),
	add_account(Identity, Identity, ValidationData, bool, Option<Vec<OmniAccountPermission>>),
	remove_accounts(Identity, Vec<Identity>),
	publicize_account(Identity, Identity),
	set_permissions(Identity, Identity, Vec<OmniAccountPermission>),
	noop(Identity),
}

impl NativeCall {
	pub fn sender_identity(&self) -> &Identity {
		match self {
			NativeCall::request_auth_token(sender_identity, ..) => sender_identity,
			NativeCall::request_intent(sender_identity, ..) => sender_identity,
			NativeCall::create_account_store(sender_identity) => sender_identity,
			NativeCall::add_account(sender_identity, ..) => sender_identity,
			NativeCall::remove_accounts(sender_identity, ..) => sender_identity,
			NativeCall::publicize_account(sender_identity, ..) => sender_identity,
			NativeCall::set_permissions(sender_identity, ..) => sender_identity,
			NativeCall::noop(sender_identity) => sender_identity,
		}
	}

	pub fn signature_message_prefix(&self) -> String {
		// TODO: update this when adding request_batch_vc variant
		"Token: ".to_string()
	}
}
