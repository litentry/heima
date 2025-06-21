use hex::{FromHex, ToHex};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rust_decimal::Decimal;

pub const WRAPPED_MAINNET_ETH: &[u8] = b"0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
pub const WRAPPPED_BASE_ETH: &[u8] = b"0x4200000000000000000000000000000000000006";
pub const GOERLI_WETH: &[u8] = b"0xB4FBF271143F4FBf7B91A5ded31805e42b2208d6";
pub const WRAPPED_BNB: &[u8] = b"0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c";

pub const NATIVE_ADDRESS: &str = "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

pub const SERVICE_FEE_PERCENT: &str = "1";
pub const CROSS_SERVICE_FEE_PERCENT: &str = "1.1";
pub const SERVICE_FEE_BPS: &str = "100";
pub const CROSS_SERVICE_FEE_BPS: &str = "110";

lazy_static! {
    pub static ref GAS_LIMIT: u64 = 450_000u64;
    pub static ref GWEI_DECIMAL: Decimal = Decimal::from(1_000_000_000u64);
    pub static ref WEI_DECIMAL: Decimal = Decimal::from(1_000_000_000_000_000_000u128);
}

lazy_static! {
    pub static ref KYBER_SWAP_DEX_ID_MAP: HashMap<String, String> = {
        let mut map = HashMap::new();
        map.insert(SWAP_NAME_UNIV2.to_string(), "uniswap".to_string());
        map.insert(SWAP_NAME_UNIV3.to_string(), "uniswapv3".to_string());
        map.insert(SWAP_NAME_PANCAKE_V2.to_string(), "pancake".to_string());
        map.insert(SWAP_NAME_PANCAKE_V3.to_string(), "pancake-v3".to_string());
        map
    };
	pub static ref INCH_DEX_IDS_MAP: HashMap<u64, HashMap<String, String>> = {
		let mut map = HashMap::new();

		let mut eth_map = HashMap::new();
		eth_map.insert("UniswapV2".to_string(), "UNISWAP_V2".to_string());
		eth_map.insert("UniswapV3".to_string(), "UNISWAP_V3".to_string());
		eth_map.insert("PancakeV2".to_string(), "ETHEREUM_PANCAKESWAP_V2".to_string());
		eth_map.insert("PancakeV3".to_string(), "PANCAKESWAP_V3".to_string());
		map.insert(1, eth_map); // ETH Chain ID

		let mut bsc_map = HashMap::new();
		bsc_map.insert("UniswapV2".to_string(), "".to_string());
		bsc_map.insert("UniswapV3".to_string(), "BSC_UNISWAP_V3".to_string());
		bsc_map.insert("PancakeV2".to_string(), "PANCAKESWAP_V2".to_string());
		bsc_map.insert("PancakeV3".to_string(), "BSC_PANCAKESWAP_V3".to_string());
		map.insert(56, bsc_map); // BSC Chain ID

		let mut base_map = HashMap::new();
		base_map.insert("UniswapV2".to_string(), "BASE_UNISWAP_V2".to_string());
		base_map.insert("UniswapV3".to_string(), "BASE_UNISWAP_V3".to_string());
		base_map.insert("PancakeV2".to_string(), "BASE_PANCAKESWAP_V2".to_string());
		base_map.insert("PancakeV3".to_string(), "BASE_PANCAKESWAP_V3".to_string());
		map.insert(8453, base_map); // BASE Chain ID

		map
	};
}

pub const SWAP_NAME_UNIV2: &str = "Uniswap V2";
pub const SWAP_NAME_UNIV3: &str = "Uniswap V3";
pub const SWAP_NAME_PANCAKE_V2: &str = "PancakeSwap V2";
pub const SWAP_NAME_PANCAKE_V3: &str = "PancakeSwap V3";
pub const SWAP_NAME_FOUR_MEME_V2: &str = "FourMeme V2";

lazy_static! {
	pub static ref OKX_DEX_IDS_MAP: HashMap<String, String> = {
		let mut map = HashMap::new();
		map.insert(SWAP_NAME_UNIV2.to_string(), "OKX_UNIV2".to_string());
		map.insert(SWAP_NAME_UNIV3.to_string(), "OKX_UNIV3".to_string());
		map.insert(SWAP_NAME_PANCAKE_V2.to_string(), "OKX_PANCAKE_V2".to_string());
		map.insert(SWAP_NAME_PANCAKE_V3.to_string(), "OKX_PANCAKE_V3".to_string());
		map
	};
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMarketTx {
	pub omni_account: String,
	pub user_id: u64,
	pub chain_id: u64,
	pub user_wallet_id: u32,
	pub user_wallet_address: String,
	pub amount_in: String,
	pub is_anti_mev: bool,
	pub is_auto_slippage: bool,
	pub slippage: u32,
	pub gas_type: i32,
	pub trade_pool_name: String,
	pub in_decimal: u8,
	pub out_decimal: u8,
	pub in_token_ca: String,
	pub out_token_ca: String,
	pub pair_addr: String,
	pub price: String,
	pub use_price_limit: bool,
	pub in_token_program: String,
	pub out_token_program: String,
	pub is_pre_cross: bool,
}

// TODO: Need to check for Okx native address too
pub fn is_native_token(token: &[u8]) -> bool {
	if token == WRAPPED_MAINNET_ETH
		|| token == WRAPPED_BNB
		|| token == WRAPPPED_BASE_ETH
		|| token == GOERLI_WETH
	{
		return true;
	}
	false
}
