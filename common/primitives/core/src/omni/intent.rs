use crate::{AccountId, Address20, Address32, Address33, Balance, BoundedVec, ChainAsset};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::Deserialize;
use sp_core::H160;
use sp_runtime::traits::ConstU32;

pub type CallEthereumInputLen = ConstU32<{ 10 * 1024 }>;
pub type RemarkLen = ConstU32<{ u32::max_value() }>;
pub type IntentStringLen = ConstU32<128>;
pub type AmountLen = ConstU32<32>;

pub type IntentId = u32;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum Intent {
    #[codec(index = 0)]
    TransferEthereum(TransferEthereum),
    #[codec(index = 1)]
    CallEthereum(CallEthereum),
    #[codec(index = 2)]
    SystemRemark(BoundedVec<u8, RemarkLen>),
    #[codec(index = 3)]
    TransferNative(TransferNative),
    #[codec(index = 4)]
    TransferSolana(TransferSolana),
    #[codec(index = 5)]
    Swap(
        SwapOrder,
        Option<CrossChainSwapProvider>,
        SingleChainSwapProvider,
    ),
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, MaxEncodedLen, TypeInfo)]
pub struct TransferEthereum {
    pub to: H160,
    pub value: [u8; 32],
}

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

#[derive(
    Encode, Decode, Copy, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, Ord, PartialOrd, Debug,
)]
pub enum HeimaMultiAddress {
    Address20(Address20),
    Address32(Address32),
    Address33(Address33),
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct SwapOrder {
    pub from_asset: ChainAsset,
    pub from_amount: BoundedVec<u8, AmountLen>,
    pub to_asset: ChainAsset,
    pub to_address: Option<HeimaMultiAddress>,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum CrossChainSwapProvider {
    Binance(BinanceConfig),
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct BinanceConfig {
    // placeholder
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum SingleChainSwapProvider {
    Pumpx(PumpxConfig),
}

// basically copied from pumpx API
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct PumpxConfig {
    pub order_type: PumpxOrderType,
    pub swap_type: u32, // 1：buy 2：sell
    pub from_chain_id: u32,
    pub from_token_ca: BoundedVec<u8, IntentStringLen>,
    pub to_chain_id: u32,
    pub to_token_ca: BoundedVec<u8, IntentStringLen>,
    pub from_amount: BoundedVec<u8, IntentStringLen>,
    pub double_out: bool,
    pub is_one_click: bool,
    pub is_anti_mev: bool,
    pub is_auto_slippage: bool,
    pub gas_type: u32, // 1: slow, 2: medium, 3: fast
    pub slippage: u32,
    pub wallet_index: u32,

    // below is only relevant to limit order, thus `Option<>`
    pub token_cap: Option<BoundedVec<u8, IntentStringLen>>,
    pub price_usd: Option<BoundedVec<u8, IntentStringLen>>,
    pub usd_worth: BoundedVec<u8, IntentStringLen>,
    pub trailing_percent: Option<u32>,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PumpxOrderType {
    Market,
    Limit,
}
