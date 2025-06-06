use async_trait::async_trait;
use reqwest::{Client, Error};
use url::Url;
pub mod methods;
use methods::add_wallet::{add_wallet_impl, AddWalletResponse};
use methods::create_cross_order::{create_cross_order_impl, CreateCrossOrderBody};
use methods::create_limit_order::{create_limit_order_impl, CreateLimitOrderBody};
use methods::create_market_order_unsigned_tx::{
	create_market_order_unsigned_tx_impl, CreateMarketOrderUnsignedTxBody,
	CreateMarketOrderUnsignedTxResponse,
};
use methods::create_transfer_unsigned_tx::{
	create_transfer_unsigned_tx_impl, CreateTransferUnsignedTxBody,
	CreateTransferUnsignedTxResponse,
};
use methods::cross_fail::{cross_fail_impl, CrossFailBody};
use methods::get_user_trade_info::{get_user_trade_info_impl, UserTradeInfoResponse};
use methods::heima_post_login::{heima_post_login_impl, PostHeimaLoginResponse};
use methods::send_order_tx::{send_order_tx_impl, SendOrderTxBody, SendOrderTxResponse};
use methods::user_connect::{user_connect_impl, UserConnectResponse};
use methods::verify_google_code::{verify_google_code_impl, VerifyGoogleCodeResponse};

use methods::send_transfer_tx::{
	send_transfer_tx_impl, SendTransferTxBody, SendTransferTxResponse,
};

use methods::create_market_order_tx::{
	create_market_order_tx_impl, CreateMarketOrderTxBody, CreateMarketOrderTxResponse,
};

use methods::create_transfer_tx::{
	create_transfer_tx_impl, CreateTransferTxBody, CreateTransferTxResponse,
};

use methods::get_gas_info::{get_gas_info_impl, GetGasInfoResponse};

use methods::common::OrderInfoResponse;
use methods::get_account_user_id::{get_account_user_id_impl, GetAccountUserIdResponse};

use crate::methods::heima_post_login::PostHeimaLoginBody;

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

	async fn heima_post_login(
		&self,
		access_token: &str,
		body: PostHeimaLoginBody,
	) -> Result<PostHeimaLoginResponse, Error>;
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
		verify_google_code_impl(self, access_token, google_code, language).await
	}

	async fn add_wallet(
		&self,
		access_token: &str,
		language: Option<String>,
	) -> Result<AddWalletResponse, Error> {
		add_wallet_impl(self, access_token, language).await
	}

	async fn get_user_trade_info(
		&self,
		access_token: &str,
	) -> Result<UserTradeInfoResponse, Error> {
		get_user_trade_info_impl(self, access_token).await
	}

	async fn create_market_order_unsigned_tx(
		&self,
		access_token: &str,
		body: CreateMarketOrderUnsignedTxBody,
	) -> Result<CreateMarketOrderUnsignedTxResponse, Error> {
		create_market_order_unsigned_tx_impl(self, access_token, body).await
	}

	async fn send_order_tx(
		&self,
		access_token: &str,
		body: SendOrderTxBody,
	) -> Result<SendOrderTxResponse, Error> {
		send_order_tx_impl(self, access_token, body).await
	}

	async fn create_limit_order(
		&self,
		access_token: &str,
		body: CreateLimitOrderBody,
	) -> Result<OrderInfoResponse, Error> {
		create_limit_order_impl(self, access_token, body).await
	}

	async fn create_cross_order(
		&self,
		access_token: &str,
		data: CreateCrossOrderBody,
	) -> Result<OrderInfoResponse, Error> {
		create_cross_order_impl(self, access_token, data).await
	}

	async fn cross_fail(
		&self,
		access_token: &str,
		data: CrossFailBody,
	) -> Result<OrderInfoResponse, Error> {
		cross_fail_impl(self, access_token, data).await
	}

	#[allow(clippy::too_many_arguments)]
	async fn create_transfer_unsigned_tx(
		&self,
		access_token: &str,
		body: CreateTransferUnsignedTxBody,
		language: Option<String>,
	) -> Result<CreateTransferUnsignedTxResponse, Error> {
		create_transfer_unsigned_tx_impl(self, access_token, body, language).await
	}

	async fn send_transfer_tx(
		&self,
		access_token: &str,
		body: SendTransferTxBody,
		language: Option<String>,
	) -> Result<SendTransferTxResponse, Error> {
		send_transfer_tx_impl(self, access_token, body, language).await
	}

	async fn create_market_order_tx(
		&self,
		access_token: &str,
		body: CreateMarketOrderTxBody,
	) -> Result<CreateMarketOrderTxResponse, Error> {
		create_market_order_tx_impl(self, access_token, body).await
	}

	#[allow(clippy::too_many_arguments)]
	async fn create_transfer_tx(
		&self,
		access_token: &str,
		body: CreateTransferTxBody,
		language: Option<String>,
	) -> Result<CreateTransferTxResponse, Error> {
		create_transfer_tx_impl(self, access_token, body, language).await
	}

	async fn get_gas_info(
		&self,
		access_token: &str,
		chain_id: u32,
	) -> Result<GetGasInfoResponse, Error> {
		get_gas_info_impl(self, access_token, chain_id).await
	}

	async fn get_account_user_id(&self, email: String) -> Result<GetAccountUserIdResponse, Error> {
		get_account_user_id_impl(self, email).await
	}

	async fn heima_post_login(
		&self,
		access_token: &str,
		body: PostHeimaLoginBody,
	) -> Result<PostHeimaLoginResponse, Error> {
		heima_post_login_impl(self, access_token, body).await
	}
}

#[cfg(feature = "mocks")]
pub mod mocks {

	use crate::methods::heima_post_login::PostHeimaLoginBody;
	use crate::methods::heima_post_login::PostHeimaLoginResponse;
	use crate::pumpx_api::AddWalletResponse;
	use crate::pumpx_api::CreateCrossOrderBody;
	use crate::pumpx_api::CreateLimitOrderBody;
	use crate::pumpx_api::CreateMarketOrderTxBody;
	use crate::pumpx_api::CreateMarketOrderTxResponse;
	use crate::pumpx_api::CreateMarketOrderUnsignedTxBody;
	use crate::pumpx_api::CreateMarketOrderUnsignedTxResponse;
	use crate::pumpx_api::CreateTransferTxBody;
	use crate::pumpx_api::CreateTransferTxResponse;
	use crate::pumpx_api::CreateTransferUnsignedTxBody;
	use crate::pumpx_api::CreateTransferUnsignedTxResponse;
	use crate::pumpx_api::CrossFailBody;
	use crate::pumpx_api::GetAccountUserIdResponse;
	use crate::pumpx_api::GetGasInfoResponse;
	use crate::pumpx_api::OrderInfoResponse;
	use crate::pumpx_api::SendOrderTxBody;
	use crate::pumpx_api::SendOrderTxResponse;
	use crate::pumpx_api::SendTransferTxBody;
	use crate::pumpx_api::SendTransferTxResponse;
	use crate::pumpx_api::UserConnectResponse;
	use crate::pumpx_api::UserTradeInfoResponse;
	use crate::pumpx_api::VerifyGoogleCodeResponse;
	use crate::PumpxApi;
	use async_trait::async_trait;
	use mockall::mock;
	use reqwest::Error;

	mock! {
		pub PumpxApiClient {}

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

			async fn heima_post_login(
				&self,
				access_token: &str,
				body: PostHeimaLoginBody,
			) -> Result<PostHeimaLoginResponse, Error>;
		}
	}
}
