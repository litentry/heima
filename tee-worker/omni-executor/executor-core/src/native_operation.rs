use executor_primitives::{Identity, Intent, OmniAccountPermission, ValidationData};
use parity_scale_codec::{Codec, Decode, Encode};
use std::vec::Vec;

pub trait NativeOperation: Codec {
	fn sender_identity(&self) -> &Identity;

	fn signature_message_prefix(&self) -> String {
		"Token: ".to_string()
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum NativeCall {
	request_auth_token(Identity),
	request_intent(Identity, Intent),
	create_account_store(Identity),
	add_account(Identity, Identity, ValidationData, bool, Option<Vec<OmniAccountPermission>>),
	remove_accounts(Identity, Vec<Identity>),
	publicize_account(Identity, Identity),
	set_permissions(Identity, Identity, Vec<OmniAccountPermission>),
	request_pumpx_jwt(Identity),
}

impl NativeOperation for NativeCall {
	fn sender_identity(&self) -> &Identity {
		match self {
			NativeCall::request_auth_token(sender_identity, ..) => sender_identity,
			NativeCall::request_intent(sender_identity, ..) => sender_identity,
			NativeCall::create_account_store(sender_identity) => sender_identity,
			NativeCall::add_account(sender_identity, ..) => sender_identity,
			NativeCall::remove_accounts(sender_identity, ..) => sender_identity,
			NativeCall::publicize_account(sender_identity, ..) => sender_identity,
			NativeCall::set_permissions(sender_identity, ..) => sender_identity,
			NativeCall::request_pumpx_jwt(sender_identity) => sender_identity,
		}
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum NativeQuery {
	get_account_store(Identity),
}

impl NativeOperation for NativeQuery {
	fn sender_identity(&self) -> &Identity {
		match self {
			NativeQuery::get_account_store(sender_identity) => sender_identity,
		}
	}
}
