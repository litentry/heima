pub mod types;

use reqwest::{Client, Error};
use types::{
	ConnectUser, MarketOrderTx, MarketOrderUnsignedTxResponse, NewMarketOrder, UserConnectResponse,
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

	pub async fn connect_user(
		&self,
		session_token: &str,
		email: String,
		invite_code: Option<String>,
		google_code: Option<String>,
		lang: Option<String>,
	) -> Result<UserConnectResponse, Error> {
		let endpoint = format!("{}/v3/account/user_connect", self.base_url);
		let connect_user = ConnectUser { email, invite_code, google_code };
		self.http_client
			.post(&endpoint)
			.header("X-Language", lang.unwrap_or("en".to_string()))
			.bearer_auth(session_token)
			.json(&connect_user)
			.send()
			.await?
			.json()
			.await
	}

	pub async fn create_market_order_unsigned_tx(
		&self,
		new_market_order: NewMarketOrder,
	) -> Result<MarketOrderUnsignedTxResponse, Error> {
		let endpoint = format!("{}/v3/trade/create_market_order_unsigned_tx", self.base_url);
		self.http_client
			.post(&endpoint)
			.json(&new_market_order)
			.send()
			.await?
			.json()
			.await
	}

	pub async fn send_market_order_tx(
		&self,
		market_order_tx: MarketOrderTx,
	) -> Result<MarketOrderUnsignedTxResponse, Error> {
		let endpoint = format!("{}/v3/trade/send_tx", self.base_url);
		self.http_client
			.post(&endpoint)
			.json(&market_order_tx)
			.send()
			.await?
			.json()
			.await
	}
}
