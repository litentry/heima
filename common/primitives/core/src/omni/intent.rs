use crate::{AccountId, Address20, Address32, Address33, Balance, BoundedVec, ChainAsset};
use alloc::string::String;
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
        OnChainSingleChainSwapProvider,
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

#[derive(Deserialize, Debug, Clone)]
pub struct PumpxConfig {
    pub order_type: PumpxOrderType,
    pub swap_type: u32, // 1：buy 2：sell
    pub from_chain_id: u32,
    pub from_token_ca: String,
    pub to_chain_id: u32,
    pub to_token_ca: String,
    pub from_amount: String,
    pub double_out: bool,
    pub is_one_click: bool,
    pub is_anti_mev: bool,
    pub is_auto_slippage: bool,
    pub gas_type: u32, // 1: slow, 2: medium, 3: fast
    pub slippage: u32,
    pub wallet_index: u32,

    // below is only relevant to limit order, thus `Option<>`
    pub token_cap: Option<String>,
    pub price_usd: Option<String>,
    pub usd_worth: String,
    pub trailing_percent: Option<u32>,
}

impl PumpxConfig {
    pub fn is_cross_chain(&self) -> bool {
        self.from_chain_id != self.to_chain_id
    }
}

impl TryFrom<OnChainPumpxConfig> for PumpxConfig {
    type Error = ();

    fn try_from(config: OnChainPumpxConfig) -> Result<Self, Self::Error> {
        Ok(PumpxConfig {
            order_type: config.order_type,
            swap_type: config.swap_type,
            from_chain_id: config.from_chain_id,
            from_token_ca: String::from_utf8(config.from_token_ca.to_vec()).map_err(|_| ())?,
            to_chain_id: config.to_chain_id,
            to_token_ca: String::from_utf8(config.to_token_ca.to_vec()).map_err(|_| ())?,
            from_amount: String::from_utf8(config.from_amount.to_vec()).map_err(|_| ())?,
            double_out: config.double_out,
            is_one_click: config.is_one_click,
            is_anti_mev: config.is_anti_mev,
            is_auto_slippage: config.is_auto_slippage,
            gas_type: config.gas_type,
            slippage: config.slippage,
            wallet_index: config.wallet_index,
            token_cap: config
                .token_cap
                .map(|v| String::from_utf8(v.to_vec()).map_err(|_| ()))
                .transpose()?,
            price_usd: config
                .price_usd
                .map(|v| String::from_utf8(v.to_vec()).map_err(|_| ()))
                .transpose()?,
            usd_worth: String::from_utf8(config.usd_worth.to_vec()).map_err(|_| ())?,
            trailing_percent: config.trailing_percent,
        })
    }
}

// basically copied from pumpx API
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct OnChainPumpxConfig {
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

impl TryFrom<PumpxConfig> for OnChainPumpxConfig {
    type Error = ();

    fn try_from(config: PumpxConfig) -> Result<Self, Self::Error> {
        Ok(OnChainPumpxConfig {
            order_type: config.order_type,
            swap_type: config.swap_type,
            from_chain_id: config.from_chain_id,
            from_token_ca: BoundedVec::try_from(config.from_token_ca.as_bytes().to_vec())
                .map_err(|_| ())?,
            to_chain_id: config.to_chain_id,
            to_token_ca: BoundedVec::try_from(config.to_token_ca.as_bytes().to_vec())
                .map_err(|_| ())?,
            from_amount: BoundedVec::try_from(config.from_amount.as_bytes().to_vec())
                .map_err(|_| ())?,
            double_out: config.double_out,
            is_one_click: config.is_one_click,
            is_anti_mev: config.is_anti_mev,
            is_auto_slippage: config.is_auto_slippage,
            gas_type: config.gas_type,
            slippage: config.slippage,
            wallet_index: config.wallet_index,
            token_cap: config
                .token_cap
                .map(|v| BoundedVec::try_from(v.as_bytes().to_vec()).map_err(|_| ()))
                .transpose()?,
            price_usd: config
                .price_usd
                .map(|v| BoundedVec::try_from(v.as_bytes().to_vec()).map_err(|_| ()))
                .transpose()?,
            usd_worth: BoundedVec::try_from(config.usd_worth.as_bytes().to_vec())
                .map_err(|_| ())?,
            trailing_percent: config.trailing_percent,
        })
    }
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PumpxOrderType {
    Market,
    Limit,
}

#[derive(Deserialize, Debug, Clone)]
pub enum SingleChainSwapProvider {
    Pumpx(PumpxConfig),
}

impl TryFrom<OnChainSingleChainSwapProvider> for SingleChainSwapProvider {
    type Error = ();

    fn try_from(scsp: OnChainSingleChainSwapProvider) -> Result<Self, Self::Error> {
        match scsp {
            OnChainSingleChainSwapProvider::Pumpx(pumpx_config) => {
                let pumpx_config = PumpxConfig::try_from(pumpx_config)?;
                Ok(SingleChainSwapProvider::Pumpx(pumpx_config))
            }
        }
    }
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum OnChainSingleChainSwapProvider {
    Pumpx(OnChainPumpxConfig),
}

impl TryFrom<SingleChainSwapProvider> for OnChainSingleChainSwapProvider {
    type Error = ();

    fn try_from(scsp: SingleChainSwapProvider) -> Result<Self, Self::Error> {
        match scsp {
            SingleChainSwapProvider::Pumpx(pumpx_config) => {
                let on_chain_config = OnChainPumpxConfig::try_from(pumpx_config)?;
                Ok(OnChainSingleChainSwapProvider::Pumpx(on_chain_config))
            }
        }
    }
}
