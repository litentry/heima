use crate::{AccountId, Address20, Address32, Address33, AssetId, Balance};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::H160;
use sp_runtime::{traits::ConstU32, BoundedVec};

pub const CALL_ETHEREUM_INPUT_LEN: u32 = 10 * 1024;

pub const MAX_REMARK_LEN: u32 = u32::max_value();

pub type ChainId = u8;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
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
    #[codec(index = 5)]
    CrossChainSwap(SwapOrder),
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
    pub to: [u8; 32],
    pub value: u64,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
pub struct Asset {
    // `name` is intentionally defined as string-like type to:
    // - allow easier integration with external service. E.g. APIs of binance/pumpx definitely expect strings
    // -
    pub name: Vec<u8>,
    pub chain: ChainId,
}

#[derive(
    Encode, Decode, Copy, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, Ord, PartialOrd, Debug,
)]
pub enum MultiAddress {
    Address32(Address32),
    Address20(Address20),
    Address33(Address33),
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
pub struct SwapOrder {
    pub from_asset: Asset,
    pub from_amount: u64,
    pub to_asset: Asset,
    pub to_min_amount: u64,
    pub to_address: Option<MultiAddress>,
}
