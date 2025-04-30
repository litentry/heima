use async_trait::async_trait;
use reqwest::{Client, Error};
use url::Url;
pub mod pumpx_methods;
pub mod pumpx_types;
pub use pumpx_methods::user_connect::user_connect_impl;
use pumpx_types::add_wallet::AddWalletResponse;
use pumpx_types::create_cross_order::CreateCrossOrderBody;
use pumpx_types::create_limit_order::CreateLimitOrderBody;
use pumpx_types::create_market_order_tx::{CreateMarketOrderTxBody, CreateMarketOrderTxResponse};
use pumpx_types::create_market_order_unsigned_tx::{
	CreateMarketOrderUnsignedTxBody, CreateMarketOrderUnsignedTxResponse,
};
use pumpx_types::create_transfer_tx::{CreateTransferTxBody, CreateTransferTxResponse};
use pumpx_types::create_transfer_unsigned_tx::{
	CreateTransferUnsignedTxBody, CreateTransferUnsignedTxResponse,
};
use pumpx_types::cross_fail::CrossFailBody;
use pumpx_types::get_account_user_id::{GetAccountUserIdParams, GetAccountUserIdResponse};
use pumpx_types::get_gas_info::{GetGasInfoParams, GetGasInfoResponse};
use pumpx_types::order_info::OrderInfoResponse;
use pumpx_types::send_order_tx::{SendOrderTxBody, SendOrderTxResponse};
use pumpx_types::send_transfer_tx::{SendTransferTxBody, SendTransferTxResponse};
use pumpx_types::user_connect::{UserConnectBody, UserConnectResponse};
use pumpx_types::user_trade_info::UserTradeInfoResponse;
use pumpx_types::verify_google_code::{GoogleCode, VerifyGoogleCodeResponse};

const DEFAULT_BASE_URL: &str = "https://api.pumpx.ai";

#[async_trait]
pub trait PumpxApi: Send + Sync {
	async fn user_connect(
		&self,
		access_token: &str,
		user_id: String,
		email: String,
		invite_code: Option<String>,
		google_code: String,
		language: Option<String>,
	) -> Result<UserConnectResponse, Error>;

	async fn verify_google_code(
		&self,
		access_token: &str,
		google_code: String,
		language: Option<String>,
	) -> Result<VerifyGoogleCodeResponse, Error>;

	async fn add_wallet(
		&self,
		access_token: &str,
		language: Option<String>,
	) -> Result<AddWalletResponse, Error>;

	async fn get_user_trade_info(&self, access_token: &str)
		-> Result<UserTradeInfoResponse, Error>;

	async fn create_market_order_unsigned_tx(
		&self,
		access_token: &str,
		body: CreateMarketOrderUnsignedTxBody,
	) -> Result<CreateMarketOrderUnsignedTxResponse, Error>;

	async fn send_order_tx(
		&self,
		access_token: &str,
		body: SendOrderTxBody,
	) -> Result<SendOrderTxResponse, Error>;

	async fn create_limit_order(
		&self,
		access_token: &str,
		body: CreateLimitOrderBody,
	) -> Result<OrderInfoResponse, Error>;

	async fn create_cross_order(
		&self,
		access_token: &str,
		data: CreateCrossOrderBody,
	) -> Result<OrderInfoResponse, Error>;

	async fn cross_fail(
		&self,
		access_token: &str,
		data: CrossFailBody,
	) -> Result<OrderInfoResponse, Error>;

	#[allow(clippy::too_many_arguments)]
	async fn create_transfer_unsigned_tx(
		&self,
		access_token: &str,
		body: CreateTransferUnsignedTxBody,
		language: Option<String>,
	) -> Result<CreateTransferUnsignedTxResponse, Error>;

	async fn send_transfer_tx(
		&self,
		access_token: &str,
		body: SendTransferTxBody,
		language: Option<String>,
	) -> Result<SendTransferTxResponse, Error>;

	async fn create_market_order_tx(
		&self,
		access_token: &str,
		body: CreateMarketOrderTxBody,
	) -> Result<CreateMarketOrderTxResponse, Error>;

	#[allow(clippy::too_many_arguments)]
	async fn create_transfer_tx(
		&self,
		access_token: &str,
		body: CreateTransferTxBody,
		language: Option<String>,
	) -> Result<CreateTransferTxResponse, Error>;

	async fn get_gas_info(
		&self,
		access_token: &str,
		chain_id: u32,
	) -> Result<GetGasInfoResponse, Error>;

	async fn get_account_user_id(&self, email: String) -> Result<GetAccountUserIdResponse, Error>;
}

pub struct PumpxApiClient {
	http_client: Client,
	base_url: Url,
}

impl PumpxApiClient {
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
		PumpxApiClient { http_client, base_url }
	}
}

#[async_trait]
impl PumpxApi for PumpxApiClient {
	async fn user_connect(
		&self,
		access_token: &str,
		user_id: String,
		email: String,
		invite_code: Option<String>,
		google_code: String,
		language: Option<String>,
	) -> Result<UserConnectResponse, Error> {
		user_connect_impl(self, access_token, user_id, email, invite_code, google_code, language)
			.await
	}

	async fn verify_google_code(
		&self,
		access_token: &str,
		google_code: String,
		language: Option<String>,
	) -> Result<VerifyGoogleCodeResponse, Error> {
		let endpoint = self.base_url.join("v3/account/verify_google_code").unwrap();
		let response = self
			.http_client
			.post(endpoint)
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

	async fn add_wallet(
		&self,
		access_token: &str,
		language: Option<String>,
	) -> Result<AddWalletResponse, Error> {
		let endpoint = self.base_url.join("v3/account/add_wallet").unwrap();
		let response = self
			.http_client
			.post(endpoint)
			.header("Content-Length", 0)
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

	async fn get_user_trade_info(
		&self,
		access_token: &str,
	) -> Result<UserTradeInfoResponse, Error> {
		let endpoint = self.base_url.join("v3/account/get_user_trade_info").unwrap();
		self.http_client
			.get(endpoint)
			.bearer_auth(access_token)
			.send()
			.await?
			.json()
			.await
	}

	async fn create_market_order_unsigned_tx(
		&self,
		access_token: &str,
		body: CreateMarketOrderUnsignedTxBody,
	) -> Result<CreateMarketOrderUnsignedTxResponse, Error> {
		let endpoint = self.base_url.join("v3/trade/create_market_order_unsigned_tx").unwrap();
		let response = self
			.http_client
			.post(endpoint)
			.bearer_auth(access_token)
			.json(&body)
			.send()
			.await
			.map_err(|e| {
				log::error!("Failed to send create_market_order_unsigned_tx request: {:?}", e);
				e
			})?;

		let status = response.status();
		let response = response.error_for_status().map_err(|e| {
			log::error!(
				"create_market_order_unsigned_tx failed with status: {}, error: {:?}",
				status,
				e
			);
			e
		})?;

		response.json().await.map_err(|e| {
			log::error!("Failed to parse create_market_order_unsigned_tx response: {:?}", e);
			e
		})
	}

	async fn send_order_tx(
		&self,
		access_token: &str,
		body: SendOrderTxBody,
	) -> Result<SendOrderTxResponse, Error> {
		let endpoint = self.base_url.join("v3/trade/send_order_tx").unwrap();
		let response = self
			.http_client
			.post(endpoint)
			.bearer_auth(access_token)
			.json(&body)
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

	async fn create_limit_order(
		&self,
		access_token: &str,
		body: CreateLimitOrderBody,
	) -> Result<OrderInfoResponse, Error> {
		let endpoint = self.base_url.join("v3/trade/create_limit_order").unwrap();
		self.http_client
			.post(endpoint)
			.bearer_auth(access_token)
			.json(&body)
			.send()
			.await?
			.json()
			.await
	}

	async fn create_cross_order(
		&self,
		access_token: &str,
		data: CreateCrossOrderBody,
	) -> Result<OrderInfoResponse, Error> {
		let endpoint = self.base_url.join("v3/trade/create_cross_order").unwrap();
		let response = self
			.http_client
			.post(endpoint)
			.bearer_auth(access_token)
			.json(&data)
			.send()
			.await?;

		let status = response.status();
		let response = response.error_for_status().map_err(|e| {
			log::error!("Cross order creation failed with status: {}, error: {:?}", status, e);
			e
		})?;

		response.json().await.map_err(|e| {
			log::error!("Failed to parse cross order creation response: {:?}", e);
			e
		})
	}

	async fn cross_fail(
		&self,
		access_token: &str,
		data: CrossFailBody,
	) -> Result<OrderInfoResponse, Error> {
		let endpoint = self.base_url.join("v3/trade/cross_fail").unwrap();
		let response = self
			.http_client
			.post(endpoint)
			.bearer_auth(access_token)
			.json(&data)
			.send()
			.await?;

		let status = response.status();
		let response = response.error_for_status().map_err(|e| {
			log::error!("Cross order failed with status: {}, error: {:?}", status, e);
			e
		})?;

		response.json().await.map_err(|e| {
			log::error!("Failed to parse cross order failed response: {:?}", e);
			e
		})
	}

	#[allow(clippy::too_many_arguments)]
	async fn create_transfer_unsigned_tx(
		&self,
		access_token: &str,
		body: CreateTransferUnsignedTxBody,
		language: Option<String>,
	) -> Result<CreateTransferUnsignedTxResponse, Error> {
		let endpoint = self.base_url.join("v3/trade/create_transfer_unsigned_tx").unwrap();
		let response = self
			.http_client
			.post(endpoint)
			.header("X-Language", language.unwrap_or("en".to_string()))
			.bearer_auth(access_token)
			.json(&body)
			.send()
			.await
			.map_err(|e| {
				log::error!("Failed to send create_transfer_unsigned_tx request: {:?}", e);
				e
			})?;

		let status = response.status();
		let response = response.error_for_status().map_err(|e| {
			log::error!(
				"create_transfer_unsigned_tx request failed with status: {}, error: {:?}",
				status,
				e
			);
			e
		})?;

		response.json().await.map_err(|e| {
			log::error!("Failed to parse create_transfer_unsigned_tx response: {:?}", e);
			e
		})
	}

	async fn send_transfer_tx(
		&self,
		access_token: &str,
		body: SendTransferTxBody,
		language: Option<String>,
	) -> Result<SendTransferTxResponse, Error> {
		let endpoint = self.base_url.join("v3/trade/send_transfer_tx").unwrap();
		let response = self
			.http_client
			.post(endpoint)
			.header("X-Language", language.unwrap_or("en".to_string()))
			.bearer_auth(access_token)
			.json(&body)
			.send()
			.await
			.map_err(|e| {
				log::error!("Failed to send send_transfer_tx request: {:?}", e);
				e
			})?;

		let status = response.status();
		let response = response.error_for_status().map_err(|e| {
			log::error!("send_transfer_tx request failed with status: {}, error: {:?}", status, e);
			e
		})?;

		response.json().await.map_err(|e| {
			log::error!("Failed to parse send_transfer_tx response: {:?}", e);
			e
		})
	}

	async fn create_market_order_tx(
		&self,
		access_token: &str,
		body: CreateMarketOrderTxBody,
	) -> Result<CreateMarketOrderTxResponse, Error> {
		let endpoint = self.base_url.join("v3/trade/create_market_order_tx").unwrap();
		let response = self
			.http_client
			.post(endpoint)
			.bearer_auth(access_token)
			.json(&body)
			.send()
			.await
			.map_err(|e| {
				log::error!("Failed to send create_market_order_tx request: {:?}", e);
				e
			})?;

		let status = response.status();
		let response = response.error_for_status().map_err(|e| {
			log::error!("create_market_order_tx failed with status: {}, error: {:?}", status, e);
			e
		})?;

		response.json().await.map_err(|e| {
			log::error!("Failed to parse create_market_order_tx response: {:?}", e);
			e
		})
	}

	#[allow(clippy::too_many_arguments)]
	async fn create_transfer_tx(
		&self,
		access_token: &str,
		body: CreateTransferTxBody,
		language: Option<String>,
	) -> Result<CreateTransferTxResponse, Error> {
		let endpoint = self.base_url.join("v3/trade/create_transfer_tx").unwrap();
		let response = self
			.http_client
			.post(endpoint)
			.header("X-Language", language.unwrap_or("en".to_string()))
			.bearer_auth(access_token)
			.json(&body)
			.send()
			.await
			.map_err(|e| {
				log::error!("Failed to send create_transfer_tx request: {:?}", e);
				e
			})?;

		let status = response.status();
		let response = response.error_for_status().map_err(|e| {
			log::error!(
				"create_transfer_tx request failed with status: {}, error: {:?}",
				status,
				e
			);
			e
		})?;

		response.json().await.map_err(|e| {
			log::error!("Failed to parse create_transfer_tx response: {:?}", e);
			e
		})
	}

	async fn get_gas_info(
		&self,
		access_token: &str,
		chain_id: u32,
	) -> Result<GetGasInfoResponse, Error> {
		let endpoint = self.base_url.join("v1/trade/get_gas_info").unwrap();
		let params = GetGasInfoParams { chain_id };
		self.http_client
			.get(endpoint)
			.bearer_auth(access_token)
			.query(&params)
			.send()
			.await?
			.json()
			.await
	}

	async fn get_account_user_id(&self, email: String) -> Result<GetAccountUserIdResponse, Error> {
		let endpoint = self.base_url.join("v3/account/get_account_user_id").unwrap();
		let params = GetAccountUserIdParams { email };
		self.http_client.get(endpoint).query(&params).send().await?.json().await
	}
}
