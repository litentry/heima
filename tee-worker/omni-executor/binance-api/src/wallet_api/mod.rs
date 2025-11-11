pub mod types;

use types::{CoinInfo, Deposit, DepositAddress, WithdrawOrder};

use crate::BinanceApi;
use crate::{error::Error, Method};
use std::collections::HashMap;

/// https://developers.binance.com/docs/wallet/Introduction
const WALLET_API: &str = "/sapi/v1";

pub struct WalletApi<'a, BinanceClient: BinanceApi> {
	base_api: &'a BinanceClient,
}

impl<'a, BinanceClient: BinanceApi> WalletApi<'a, BinanceClient> {
	pub fn new(binance_api: &BinanceClient) -> WalletApi<'_, BinanceClient> {
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
	pub async fn get_deposit_history(
		&self,
		coin: Option<String>,
		tx_id: Option<String>,
	) -> Result<Vec<Deposit>, Error> {
		let endpoint = format!("{}/capital/deposit/hisrec", WALLET_API);
		let mut params = HashMap::new();
		params.insert(
			"startTime".to_string(),
			(chrono::Utc::now() - chrono::Duration::hours(1)).timestamp_millis().to_string(),
		);
		params.insert("includeSource".to_string(), "true".to_string());
		if let Some(coin) = coin {
			params.insert("coin".to_string(), coin);
		}
		if let Some(tx_id) = tx_id {
			params.insert("txId".to_string(), tx_id);
		}
		self.base_api
			.make_signed_request(&endpoint, Method::GET, Some(params), None)
			.await
	}

	/// Submit a withdraw request.
	pub async fn withdraw(
		&self,
		coin: &str,
		address: &str,
		amount: String, // Decimal amount
		network: Option<&str>,
	) -> Result<WithdrawOrder, Error> {
		let endpoint = format!("{}/capital/withdraw/apply", WALLET_API);
		let mut params = HashMap::new();
		params.insert("coin".to_string(), coin.to_string());
		params.insert("address".to_string(), address.to_string());
		params.insert("amount".to_string(), amount);
		if let Some(network) = network {
			params.insert("network".to_string(), network.to_string());
		}
		self.base_api
			.make_signed_request(&endpoint, Method::POST, Some(params), None)
			.await
	}
}
