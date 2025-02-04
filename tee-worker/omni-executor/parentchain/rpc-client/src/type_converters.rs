use executor_primitives::{intent::Intent, AccountId, Hash, MemberAccount, OmniAccountAuthType};
use parentchain_api_interface::{
	omni_account::calls::types::dispatch_as_omni_account::AuthType as SubxtOmniAccountAuthType,
	runtime_types::core_primitives::{
		intent::Intent as SubxtIntent, omni_account::MemberAccount as SubxtMemberAccount,
	},
};
use parity_scale_codec::{Decode, Encode};
use subxt_core::utils::{AccountId32 as SubxtAccountId, H256 as SubxtHash};

pub trait ToSubxtType<T: Decode>: Encode {
	fn to_subxt_type(&self) -> T;
}

pub trait ToPrimitiveType<T: Decode>: Encode {
	fn to_primitive_type(&self) -> T;
}

impl ToSubxtType<SubxtAccountId> for AccountId {
	fn to_subxt_type(&self) -> SubxtAccountId {
		let bytes = self.encode();
		Decode::decode(&mut &bytes[..]).expect("Failed to decode SubxtAccountId")
	}
}

impl ToPrimitiveType<AccountId> for SubxtAccountId {
	fn to_primitive_type(&self) -> AccountId {
		let bytes = self.encode();
		Decode::decode(&mut &bytes[..]).expect("Failed to decode AccountId")
	}
}

impl ToSubxtType<SubxtHash> for Hash {
	fn to_subxt_type(&self) -> SubxtHash {
		let bytes = self.encode();
		Decode::decode(&mut &bytes[..]).expect("Failed to decode SubxtHash")
	}
}

impl ToPrimitiveType<Hash> for SubxtHash {
	fn to_primitive_type(&self) -> Hash {
		let bytes = self.encode();
		Decode::decode(&mut &bytes[..]).expect("Failed to decode Hash")
	}
}

impl ToSubxtType<SubxtOmniAccountAuthType> for OmniAccountAuthType {
	fn to_subxt_type(&self) -> SubxtOmniAccountAuthType {
		let bytes = self.encode();
		Decode::decode(&mut &bytes[..]).expect("Failed to decode SubxtOmniAccountAuthType")
	}
}

impl ToSubxtType<SubxtIntent> for Intent {
	fn to_subxt_type(&self) -> SubxtIntent {
		let bytes = self.encode();
		Decode::decode(&mut &bytes[..]).expect("Failed to decode SubxtIntent")
	}
}

impl ToPrimitiveType<MemberAccount> for SubxtMemberAccount {
	fn to_primitive_type(&self) -> MemberAccount {
		let bytes = self.encode();
		Decode::decode(&mut &bytes[..]).expect("Failed to decode MemberAccount")
	}
}
