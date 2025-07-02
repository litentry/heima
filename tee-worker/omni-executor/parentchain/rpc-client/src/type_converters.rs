use executor_primitives::{AccountId, Hash, Identity, Intent, OmniAccountAuthType};
use parentchain_api_interface::runtime_types::heima_primitives::{
	identity::Identity as SubxtIdentity,
	omni::account::OmniAccountAuthType as SubxtOmniAccountAuthType,
	omni::intent::Intent as SubxtIntent,
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

impl ToSubxtType<SubxtIdentity> for Identity {
	fn to_subxt_type(&self) -> SubxtIdentity {
		let bytes = self.encode();
		Decode::decode(&mut &bytes[..]).expect("Failed to decode SubxtIdentity")
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
