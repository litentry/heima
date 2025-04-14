mod types;

use types::{CoinInfo, Deposit, DepositAddress};

use crate::{error::Error, BinanceApi, Method};
use std::collections::HashMap;

/// https://developers.binance.com/docs/wallet/Introduction
const WALLET_API: &str = "/sapi/v1";

pub struct WalletApi<'a> {
	base_api: &'a BinanceApi,
}

impl<'a> WalletApi<'a> {
	pub fn new(binance_api: &BinanceApi) -> WalletApi {
		WalletApi { base_api: binance_api }
	}

	/// Get information of coins (available for deposit and withdraw).
	pub async fn get_all_coins_info(&self) -> Result<Vec<CoinInfo>, Error> {
		let endpoint = format!("{}/capital/config/getall", WALLET_API);
		self.base_api.make_signed_request(&endpoint, Method::GET, None, None).await
	}

	/// Get deposit address with network
	pub async fn get_deposit_address(&self, coin: &str, network: &str) -> Result<String, Error> {
		let endpoint = format!("{}/capital/deposit/address", WALLET_API);
		let mut params = HashMap::new();
		params.insert("coin".to_string(), coin.to_string());
		params.insert("network".to_string(), network.to_string());
		let response: DepositAddress = self
			.base_api
			.make_signed_request(&endpoint, Method::GET, Some(params), None)
			.await?;
		Ok(response.address)
	}

	/// Fetch deposit history
	pub async fn get_deposit_history(&self, coin: Option<String>) -> Result<Vec<Deposit>, Error> {
		let endpoint = format!("{}/capital/deposit/hisrec", WALLET_API);
		let mut params = HashMap::new();
		params.insert("includeSource".to_string(), "true".to_string());
		if let Some(coin) = coin {
			params.insert("coin".to_string(), coin);
		}
		self.base_api
			.make_signed_request(&endpoint, Method::GET, Some(params), None)
			.await
	}
}
