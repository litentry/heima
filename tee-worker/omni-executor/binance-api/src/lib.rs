pub mod convert_api;
mod error;
pub mod spot_trading_api;
mod traits;
mod types;
pub mod wallet_api;

use async_trait::async_trait;
use error::Error;
use hmac::{Hmac, Mac};
use reqwest::{Client, Method};
use sha2::Sha256;
use std::{
	collections::HashMap,
	time::{SystemTime, UNIX_EPOCH},
};
use tracing::log::{debug, error};
use url::Url;

const MAX_RECV_WINDOW: u32 = 60000;

#[async_trait]
pub trait BinanceApi: Send + Sync {
	fn sign_request(&self, query_string: &str) -> String;
	async fn make_public_get_request<T>(
		&self,
		endpoint: &str,
		parameters: Option<HashMap<String, String>>,
	) -> Result<T, Error>
	where
		T: 'static + serde::de::DeserializeOwned;

	async fn make_signed_request<T>(
		&self,
		endpoint: &str,
		method: Method,
		parameters: Option<HashMap<String, String>>,
		recv_window: Option<u32>,
	) -> Result<T, Error>
	where
		T: 'static + serde::de::DeserializeOwned;
}

#[derive(Debug, Clone)]
pub struct BinanceApiClient {
	client: Client,
	base_url: Url,
	api_key: String,
	api_secret: String,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
pub enum BinanceApiResponse<T> {
	Success(T),
	Error { code: i32, msg: String },
}

impl BinanceApiClient {
	pub fn new(api_key: String, api_secret: String, base_url: String) -> BinanceApiClient {
		let base_url = Url::parse(&base_url).expect("Invalid base URL");
		let client = Client::new();
		BinanceApiClient { client, base_url, api_key, api_secret }
	}
}

#[async_trait]
impl BinanceApi for BinanceApiClient {
	/// Create HMAC SHA256 signature for request parameters
	fn sign_request(&self, query_string: &str) -> String {
		let mut mac = Hmac::<Sha256>::new_from_slice(self.api_secret.as_bytes())
			.expect("HMAC can take key of any size");
		mac.update(query_string.as_bytes());
		let result = mac.finalize();
		let code_bytes = result.into_bytes();
		hex::encode(code_bytes)
	}

	/// Helper for making public API requests
	async fn make_public_get_request<T>(
		&self,
		endpoint: &str,
		parameters: Option<HashMap<String, String>>,
	) -> Result<T, Error>
	where
		T: 'static + serde::de::DeserializeOwned,
	{
		let mut url = self.base_url.join(endpoint).unwrap();
		if let Some(p) = parameters {
			url.query_pairs_mut().extend_pairs(p.iter());
		}

		let response = self.client.get(url.as_str()).send().await.map_err(|e| {
			error!("API request failed: {}", e);
			Error::RequestFailed
		})?;

		debug!("Binance-api make_public_get_request response: {:?}", response);

		response
			.json::<BinanceApiResponse<T>>()
			.await
			.map_err(|e| {
				error!("Error parsing response: {}", e);
				Error::ParseResponseFailed
			})
			.and_then(|res| match res {
				BinanceApiResponse::Success(data) => Ok(data),
				BinanceApiResponse::Error { code, msg } => {
					error!("Binance API Error: {} - {}", code, msg);
					Err(Error::RequestFailed)
				},
			})
	}

	/// Helper for making signed API requests
	async fn make_signed_request<T>(
		&self,
		endpoint: &str,
		method: Method,
		parameters: Option<HashMap<String, String>>,
		recv_window: Option<u32>,
	) -> Result<T, Error>
	where
		T: 'static + serde::de::DeserializeOwned,
	{
		let mut url = self.base_url.join(endpoint).unwrap();
		let mut params = HashMap::new();

		let timestamp = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.expect("Time went backwards")
			.as_millis() as u64;
		params.insert("timestamp".to_string(), timestamp.to_string());

		if let Some(window) = recv_window {
			if window > MAX_RECV_WINDOW {
				return Err(Error::MaxRecvWindowExceeded);
			}
			params.insert("recvWindow".to_string(), window.to_string());
		}

		if let Some(p) = parameters {
			params.extend(p);
		}

		let signature = {
			let mut tmp_url = url.clone();
			tmp_url.query_pairs_mut().extend_pairs(params.iter());
			let query_string = tmp_url.query().unwrap().to_string();
			self.sign_request(&query_string)
		};

		let request = match method {
			Method::GET => {
				url.query_pairs_mut().extend_pairs(params.iter());
				url.query_pairs_mut().append_pair("signature", &signature);
				self.client.request(method, url.as_str()).header("X-MBX-APIKEY", &self.api_key)
			},
			Method::POST => {
				params.insert("signature".to_string(), signature);
				self.client
					.request(method, url.as_str())
					.header("X-MBX-APIKEY", &self.api_key)
					.form(&params)
			},
			_ => {
				return Err(Error::MethodNotSupported);
			},
		};

		debug!("Binance-api make_signed_request: {:?}", request);
		let response = request.send().await.map_err(|e| {
			error!("API request failed: {}", e);
			Error::RequestFailed
		})?;

		debug!("Binance-api make_signed_request response: {:?}", response);

		response
			.json::<BinanceApiResponse<T>>()
			.await
			.map_err(|e| {
				error!("Error parsing response: {}", e);
				Error::ParseResponseFailed
			})
			.and_then(|res| match res {
				BinanceApiResponse::Success(data) => Ok(data),
				BinanceApiResponse::Error { code, msg } => {
					error!("Binance API Error: {} - {}", code, msg);
					Err(Error::RequestFailed)
				},
			})
	}
}

#[cfg(feature = "mocks")]
pub mod mocks {
	use crate::BinanceApi;
	use crate::Error;
	use crate::HashMap;
	use crate::Method;
	use async_trait::async_trait;
	use mockall::mock;

	mock! {

		pub BinanceApiClient {}

		#[async_trait]
		impl BinanceApi for BinanceApiClient {

			fn sign_request(&self, query_string: &str) -> String;
			async fn make_public_get_request<T>(
				&self,
				endpoint: &str,
				parameters: Option<HashMap<String, String>>,
			) -> Result<T, Error>
			where
				T: 'static + serde::de::DeserializeOwned;

			async fn make_signed_request<T>(
				&self,
				endpoint: &str,
				method: Method,
				parameters: Option<HashMap<String, String>>,
				recv_window: Option<u32>,
			) -> Result<T, Error>
			where
				T: 'static + serde::de::DeserializeOwned;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_sign_request() {
		let api_key =
			"vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A".to_string();
		let api_secret =
			"NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j".to_string();
		let query_string = "symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559".to_string();
		let binance_api =
			BinanceApiClient::new(api_key, api_secret, "https://api.binance.com".to_string());
		let signature = binance_api.sign_request(&query_string);

		assert_eq!(signature, "c8db56825ae71d6d79447849e617115f4a920fa2acdcab2b053c4b2838bd6b71");
	}
}
