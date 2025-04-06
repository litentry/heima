pub mod types;

use reqwest::{Client, Error};
use types::{
	AddWalletResponse, ConnectUser, GoogleCode, LimitOrderResponse, MarketOrderTx,
	MarketOrderTxResponse, MarketOrderUnsignedTxResponse, NewLimitOrder, NewMarketOrder,
	UserConnectResponse, UserTradeInfoResponse, VerifyGoogleCodeResponse,
};
use url::Url;

const DEFAULT_BASE_URL: &str = "https://api.pumpx.ai";

pub struct PumpxApi {
	http_client: Client,
	base_url: Url,
}

impl PumpxApi {
	pub fn new(base_url: Option<String>) -> Self {
		let base_url = match base_url {
			Some(url) => Url::parse(&url).expect("Invalid base URL"),
			None => Url::parse(DEFAULT_BASE_URL).unwrap(),
		};
		let mut default_headers = reqwest::header::HeaderMap::new();
		default_headers.insert("X-Language", "en".parse().unwrap());
		let http_client = Client::builder()
			.default_headers(default_headers)
			.build()
			.expect("Failed to build HTTP client");
		PumpxApi { http_client, base_url }
	}

	pub async fn user_connect(
		&self,
		access_token: &str,
		email: String,
		invite_code: Option<String>,
		google_code: Option<String>,
		language: Option<String>,
	) -> Result<UserConnectResponse, Error> {
		let endpoint = format!("{}/v3/account/user_connect", self.base_url);
		let user_connect = ConnectUser { email: email.clone(), invite_code, google_code };

		let response = self
			.http_client
			.post(&endpoint)
			.header("X-Language", language.unwrap_or("en".to_string()))
			.bearer_auth(access_token)
			.json(&user_connect)
			.send()
			.await
			.map_err(|e| {
				log::error!("Failed to send user connect request: {:?}", e);
				e
			})?;

		let status = response.status();
		let response = response.error_for_status().map_err(|e| {
			log::error!("User connect request failed with status: {}, error: {:?}", status, e);
			e
		})?;

		response.json().await.map_err(|e| {
			log::error!("Failed to parse user connect response: {:?}", e);
			e
		})
	}

	pub async fn verify_google_code(
		&self,
		access_token: &str,
		google_code: String,
		language: Option<String>,
	) -> Result<VerifyGoogleCodeResponse, Error> {
		let endpoint = format!("{}/v3/account/verify_google_code", self.base_url);
		let response = self
			.http_client
			.post(&endpoint)
			.header("X-Language", language.unwrap_or("en".to_string()))
			.bearer_auth(access_token)
			.json(&GoogleCode { google_code })
			.send()
			.await
			.map_err(|e| {
				log::error!("Failed to send Google code verification request: {:?}", e);
				e
			})?;
		let status = response.status();
		let response = response.error_for_status().map_err(|e| {
			log::error!("Google code verification failed with status: {}, error: {:?}", status, e);
			e
		})?;
		response.json().await.map_err(|e| {
			log::error!("Failed to parse Google code verification response: {:?}", e);
			e
		})
	}

	pub async fn add_wallet(
		&self,
		access_token: &str,
		language: Option<String>,
	) -> Result<AddWalletResponse, Error> {
		let endpoint = format!("{}/v3/account/add_wallet", self.base_url);
		let response = self
			.http_client
			.post(&endpoint)
			.header("X-Language", language.unwrap_or("en".to_string()))
			.bearer_auth(access_token)
			.send()
			.await
			.map_err(|e| {
				log::error!("Failed to send add_wallet request: {:?}", e);
				e
			})?;
		let status = response.status();
		let response = response.error_for_status().map_err(|e| {
			log::error!("add_wallet request failed with status: {}, error: {:?}", status, e);
			e
		})?;
		response.json().await.map_err(|e| {
			log::error!("Failed to parse add_wallet response: {:?}", e);
			e
		})
	}

	pub async fn get_user_trade_info(
		&self,
		access_token: &str,
	) -> Result<UserTradeInfoResponse, Error> {
		let endpoint = format!("{}/v3/account/get_user_trade_info", self.base_url);
		self.http_client
			.get(&endpoint)
			.bearer_auth(access_token)
			.send()
			.await?
			.json()
			.await
	}

	pub async fn create_market_order_unsigned_tx(
		&self,
		access_token: &str,
		new_market_order: NewMarketOrder,
	) -> Result<MarketOrderUnsignedTxResponse, Error> {
		let endpoint = format!("{}/v3/trade/create_market_order_unsigned_tx", self.base_url);
		let response = self
			.http_client
			.post(&endpoint)
			.bearer_auth(access_token)
			.json(&new_market_order)
			.send()
			.await
			.map_err(|e| {
				log::error!("Failed to send market order creation request: {:?}", e);
				e
			})?;

		let status = response.status();
		let response = response.error_for_status().map_err(|e| {
			log::error!("Market order creation failed with status: {}, error: {:?}", status, e);
			e
		})?;

		response.json().await.map_err(|e| {
			log::error!("Failed to parse market order creation response: {:?}", e);
			e
		})
	}

	pub async fn send_market_order_tx(
		&self,
		access_token: &str,
		market_order_tx: MarketOrderTx,
	) -> Result<MarketOrderTxResponse, Error> {
		let endpoint = format!("{}/v3/trade/send_tx", self.base_url);
		let response = self
			.http_client
			.post(&endpoint)
			.bearer_auth(access_token)
			.json(&market_order_tx)
			.send()
			.await
			.map_err(|e| {
				log::error!("Failed to send market order transaction: {:?}", e);
				e
			})?;

		let status = response.status();
		let response = response.error_for_status().map_err(|e| {
			log::error!("Market order transaction failed with status: {}, error: {:?}", status, e);
			e
		})?;

		response.json().await.map_err(|e| {
			log::error!("Failed to parse market order transaction response: {:?}", e);
			e
		})
	}

	pub async fn create_limit_order(
		&self,
		access_token: &str,
		new_limit_order: NewLimitOrder,
	) -> Result<LimitOrderResponse, Error> {
		let endpoint = format!("{}/v3/trade/create_limit_order", self.base_url);
		self.http_client
			.post(&endpoint)
			.bearer_auth(access_token)
			.json(&new_limit_order)
			.send()
			.await?
			.json()
			.await
	}
}
