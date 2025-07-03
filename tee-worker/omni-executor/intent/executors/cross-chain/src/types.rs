use base58::ToBase58;
use executor_primitives::{Chain, ChainAsset, EthereumToken, SolanaToken};
use std::collections::HashMap;

use rust_decimal::Decimal;
use tracing::error;

pub type RpcEndpointRegistry = HashMap<Chain, String>;

pub struct CrossChainOrderParams {
	pub to_token_ca: String,
	pub from_token_ca: String,
	pub from_amount: String,
	pub usd_worth: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BinanceNetwork {
	Bsc,
	Sol,
}

impl BinanceNetwork {
	const BSC: &'static str = "BSC";
	const SOL: &'static str = "SOL";

	pub fn name(&self) -> &'static str {
		match self {
			Self::Bsc => Self::BSC,
			Self::Sol => Self::SOL,
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum BinanceCoin {
	Bnb,
	Sol,
	Fc,
	Usdc,
	Usdt,
}

impl BinanceCoin {
	// Biannce coin names
	const BNB: &'static str = "BNB";
	const SOL: &'static str = "SOL";
	const FC: &'static str = "FC";
	const USDC: &'static str = "USDC";
	const USDT: &'static str = "USDT";

	pub fn name(&self) -> &'static str {
		match self {
			Self::Bnb => Self::BNB,
			Self::Sol => Self::SOL,
			Self::Fc => Self::FC,
			Self::Usdc => Self::USDC,
			Self::Usdt => Self::USDT,
		}
	}

	pub fn decimals(&self) -> u32 {
		match self {
			BinanceCoin::Fc | BinanceCoin::Bnb => 18,
			BinanceCoin::Sol => 9,
			BinanceCoin::Usdc | BinanceCoin::Usdt => 6,
		}
	}

	pub fn decimal_value(&self) -> Decimal {
		Decimal::from(10u64.pow(self.decimals()))
	}
}

pub trait FromBscTokenAddress {
	// Bsc token addresss
	const BSC_TOKEN_ADDRESS_FC: &'static str = "4C3617A56a5E0Ca1D8CDd629695b0a8BE669971A";
	const BSC_TOKEN_ADDRESS_USDC: &'static str = "8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d";
	const BSC_TOKEN_ADDRESS_USDT: &'static str = "55d398326f99059fF775485246999027B3197955";

	fn from_bsc_token_address(address: &str) -> Result<BinanceCoin, ()>;
}

impl FromBscTokenAddress for BinanceCoin {
	fn from_bsc_token_address(address: &str) -> Result<BinanceCoin, ()> {
		match address {
			Self::BSC_TOKEN_ADDRESS_FC => Ok(Self::Fc),
			Self::BSC_TOKEN_ADDRESS_USDC => Ok(Self::Usdc),
			Self::BSC_TOKEN_ADDRESS_USDT => Ok(Self::Usdt),
			_ => {
				error!("Unsupported BSC token: {:?}", address);
				Err(())
			},
		}
	}
}

pub trait FromSolanaMintAddress {
	// Solana mint adresses
	const SOLANA_MINT_ADDRESS_USDC: &'static str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
	const SOLANA_MINT_ADDRESS_USDT: &'static str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";

	fn from_solana_mint_address(address: &str) -> Result<BinanceCoin, ()>;
}

impl FromSolanaMintAddress for BinanceCoin {
	fn from_solana_mint_address(address: &str) -> Result<BinanceCoin, ()> {
		match address {
			Self::SOLANA_MINT_ADDRESS_USDC => Ok(Self::Usdc),
			Self::SOLANA_MINT_ADDRESS_USDT => Ok(Self::Usdt),
			_ => {
				error!("Unsupported SPL token: {:?}", address);
				Err(())
			},
		}
	}
}

pub struct BinanceAsset {
	pub network: BinanceNetwork,
	pub coin: BinanceCoin,
	pub address: String,
}

pub trait FromChainAsset {
	fn from_chain_asset(asset: &ChainAsset) -> Result<BinanceAsset, ()>;
}

impl FromChainAsset for BinanceAsset {
	fn from_chain_asset(asset: &ChainAsset) -> Result<BinanceAsset, ()> {
		match asset {
			ChainAsset::Ethereum(pumpx::constants::BSC_CHAIN_ID, token) => {
				let (coin, address) = match token {
					EthereumToken::Native => (BinanceCoin::Bnb, String::new()),
					EthereumToken::ERC20(address) => {
						let address_hex = hex::encode(address.as_ref());
						let coin = BinanceCoin::from_bsc_token_address(address_hex.as_str())?;
						(coin, address_hex)
					},
				};
				Ok(BinanceAsset { network: BinanceNetwork::Bsc, coin, address })
			},
			ChainAsset::Solana(token) => {
				let (coin, address) = match token {
					SolanaToken::Native => (BinanceCoin::Sol, String::new()),
					SolanaToken::SPL(mint_address) => {
						let mint_address_string = mint_address.as_ref().to_base58();
						let coin =
							BinanceCoin::from_solana_mint_address(mint_address_string.as_str())?;
						(coin, mint_address_string)
					},
				};
				Ok(BinanceAsset { network: BinanceNetwork::Sol, coin, address })
			},
			_ => {
				error!("Unsupported asset: {:?}", asset);
				Err(())
			},
		}
	}
}
