use crate::{AccountId, Address20, Address32, Address33, Balance, ChainAsset};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::H160;
use sp_runtime::{traits::ConstU32, BoundedVec};

pub const CALL_ETHEREUM_INPUT_LEN: u32 = 10 * 1024;
pub const MAX_REMARK_LEN: u32 = u32::max_value();

pub type IntentId = u128;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
pub enum Intent {
    #[codec(index = 0)]
    CrossChainSwap(SwapOrder),
    #[codec(index = 1)]
    TransferEthereum(TransferEthereum),
    #[codec(index = 2)]
    CallEthereum(CallEthereum),
    #[codec(index = 3)]
    SystemRemark(IntentId, BoundedVec<u8, ConstU32<MAX_REMARK_LEN>>),
    #[codec(index = 4)]
    TransferNative(TransferNative),
    #[codec(index = 5)]
    TransferSolana(TransferSolana),
}

impl Intent {
    pub fn intent_id(&self) -> &IntentId {
        match self {
            Intent::CrossChainSwap(so) => &so.intent_id,
            Intent::TransferEthereum(te) => &te.intent_id,
            Intent::CallEthereum(ce) => &ce.intent_id,
            Intent::SystemRemark(intent_id, ..) => &intent_id,
            Intent::TransferNative(tn) => &tn.intent_id,
            Intent::TransferSolana(ts) => &ts.intent_id,
        }
    }
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub struct TransferEthereum {
    pub intent_id: IntentId,
    pub to: H160,
    pub value: [u8; 32],
}

pub type CallEthereumInputLen = ConstU32<CALL_ETHEREUM_INPUT_LEN>;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub struct CallEthereum {
    pub intent_id: IntentId,
    pub address: H160,
    pub input: BoundedVec<u8, CallEthereumInputLen>,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub struct TransferNative {
    pub intent_id: IntentId,
    pub to: AccountId,
    pub value: Balance,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub struct TransferSolana {
    pub intent_id: IntentId,
    pub to: [u8; 32],
    pub value: u64,
}

#[derive(
    Encode, Decode, Copy, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, Ord, PartialOrd, Debug,
)]
pub enum HeimaMultiAddress {
    Address20(Address20),
    Address32(Address32),
    Address33(Address33),
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
pub struct SwapOrder {
    pub intent_id: IntentId,
    pub from_asset: ChainAsset,
    pub from_amount: u64,
    pub to_asset: ChainAsset,
    pub to_address: Option<HeimaMultiAddress>,
}
