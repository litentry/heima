mod types;

use reqwest::{
	header::{HeaderMap, HeaderValue},
	Client, Error,
};
use types::{
	ApiResponse, EmptyData, MarketOrderTx, MarketOrderUnsignedTx, NewMarketOrder, NewUser, TxData,
	User, Wallet,
};
use url::Url;

const DEFAULT_BASE_URL: &str = "https://api.pumpx.ai";

pub struct PumpxApi {
	http_client: Client,
	base_url: Url,
}

impl PumpxApi {
	pub fn new(base_url: Option<&str>) -> Self {
		let base_url = match base_url {
			Some(url) => Url::parse(url).expect("Invalid base URL"),
			None => Url::parse(DEFAULT_BASE_URL).unwrap(),
		};
		let mut headers = HeaderMap::new();
		headers.insert("X-Language", HeaderValue::from_static("en"));
		let http_client = Client::builder()
			.default_headers(headers)
			.build()
			.expect("Failed to build HTTP client");
		PumpxApi { http_client, base_url }
	}

	pub async fn register_user(
		&self,
		invite_code: String,
		wallet_list: Vec<Wallet>,
		email: Option<String>,
	) -> Result<ApiResponse<EmptyData>, Error> {
		let endpoint = format!("{}/v3/account/user_register", self.base_url);
		let new_user = NewUser { invite_code, wallet_list, email };
		self.http_client.post(&endpoint).json(&new_user).send().await?.json().await
	}

	// TODO: double check how this works
	pub async fn login_user(&self) -> Result<ApiResponse<User>, Error> {
		let endpoint = format!("{}/v3/account/user_login", self.base_url);
		self.http_client.post(&endpoint).send().await?.json().await
	}

	pub async fn create_market_order_unsigned_tx(
		&self,
		new_market_order: NewMarketOrder,
	) -> Result<ApiResponse<MarketOrderUnsignedTx>, Error> {
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
	) -> Result<ApiResponse<TxData>, Error> {
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
