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

pub type MaybeGoogleCode = Option<String>;
pub type PumxWalletIndex = u32;
pub type PumpxChainId = u32;

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
	PumpxRequestJwt(Identity, Option<String>, MaybeGoogleCode, Option<String>),
	#[codec(index = 21)]
	PumpxExportWallet(Identity, MaybeGoogleCode, PumpxChainId, PumxWalletIndex, String),
	#[codec(index = 22)]
	PumpxAddWallet(Identity),
	#[codec(index = 23)]
	PumpxTransferWidthdraw(
		Identity,
		u64,             // request_id
		u32,             // chain_id
		u32,             // wallet_index
		String,          // recipient_address
		String,          // token_ca
		String,          // amount
		MaybeGoogleCode, // google_code
		Option<String>,  // language
	),
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
			Self::PumpxRequestJwt(sender, ..) => sender,
			Self::PumpxExportWallet(sender, ..) => sender,
			Self::PumpxAddWallet(sender, ..) => sender,
			Self::PumpxTransferWidthdraw(sender, ..) => sender,
		}
	}

	fn require_auth(&self) -> bool {
		// currently all tasks require auth
		true
	}

	fn require_encrypt(&self) -> bool {
		matches!(self, Self::PumpxExportWallet(..))
	}
}
