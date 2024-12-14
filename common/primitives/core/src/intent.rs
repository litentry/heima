use crate::{AccountId, Address32, Balance};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::H160;
use sp_runtime::{traits::ConstU32, BoundedVec};

pub const CALL_ETHEREUM_INPUT_LEN: u32 = 10 * 1024;

pub const MAX_REMARK_LEN: u32 = u32::max_value();

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub enum Intent {
    #[codec(index = 0)]
    TransferEthereum(TransferEthereum),
    #[codec(index = 1)]
    CallEthereum(CallEthereum),
    #[codec(index = 2)]
    SystemRemark(BoundedVec<u8, ConstU32<MAX_REMARK_LEN>>),
    #[codec(index = 3)]
    TransferNative(TransferNative),
    #[codec(index = 4)]
    TransferSolana(TransferSolana),
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub struct TransferEthereum {
    pub to: H160,
    pub value: [u8; 32],
}

pub type CallEthereumInputLen = ConstU32<CALL_ETHEREUM_INPUT_LEN>;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub struct CallEthereum {
    pub address: H160,
    pub input: BoundedVec<u8, CallEthereumInputLen>,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub struct TransferNative {
    pub to: AccountId,
    pub value: Balance,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub struct TransferSolana {
    pub to: Address32,
    pub value: u64,
}
