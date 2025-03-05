mod error;
mod types;

use error::Error;
use hmac::{Hmac, Mac};
use log::error;
use reqwest::{Client, Method};
use sha2::Sha256;
use std::{
	collections::HashMap,
	time::{SystemTime, UNIX_EPOCH},
};
use types::{AssetInfo, AssetSymbol, ConvertOrder, Quote, RequestQuoteParams, TokenPair};
use url::Url;

const CONVERT_API: &str = "/sapi/v1/convert";
const MAX_RECV_WINDOW: u32 = 60000;

pub struct BinanceApi {
	client: Client,
	base_url: Url,
	api_key: String,
	api_secret: String,
}

impl BinanceApi {
	pub fn new(api_key: String, api_secret: String, base_url: Option<String>) -> BinanceApi {
		let base_url = match base_url {
			Some(url) => Url::parse(&url).expect("Invalid base URL"),
			None => Url::parse("https://api.binance.com").unwrap(),
		};
		let client = Client::new();
		BinanceApi { client, base_url, api_key, api_secret }
	}

	/// List all convertible token pairs and the tokens’ respective upper/lower limits
	pub async fn get_exchange_info(
		&self,
		from_asset: Option<AssetSymbol>,
		to_asset: Option<AssetSymbol>,
	) -> Result<Vec<TokenPair>, Error> {
		let endpoint = format!("{}/exchangeInfo", CONVERT_API);
		let mut url = self.base_url.join(&endpoint).unwrap();
		if let Some(from_asset) = from_asset {
			url.query_pairs_mut().append_pair("fromAsset", &from_asset);
		}
		if let Some(to_asset) = to_asset {
			url.query_pairs_mut().append_pair("toAsset", &to_asset);
		}
		let response = self.client.get(url.as_str()).send().await.map_err(|e| {
			error!("Error getting exchange info: {}", e);
			Error::RequestFailed
		})?;
		let token_pairs: Vec<TokenPair> = response.json().await.map_err(|e| {
			error!("Error parsing exchange info: {}", e);
			Error::ParseResponseFailed
		})?;

		Ok(token_pairs)
	}

	/// List all supported asset’s precision information
	pub async fn get_asset_info(&self, recv_window: Option<u32>) -> Result<Vec<AssetInfo>, Error> {
		let endpoint = format!("{}/assetInfo", CONVERT_API);
		self.make_signed_request(&endpoint, Method::GET, None, recv_window).await
	}

	/// Request a quote for the requested token pairs
	pub async fn get_quote(
		&self,
		request_quote_params: RequestQuoteParams,
		recv_window: Option<u32>,
	) -> Result<Quote, Error> {
		let endpoint = format!("{}/getQuote", CONVERT_API);
		let params = request_quote_params.try_into_params().map_err(|e| {
			error!("Error converting request quote params: {}", e);
			Error::InvalidParams
		})?;
		self.make_signed_request(&endpoint, Method::POST, Some(params), recv_window)
			.await
	}

	pub async fn accept_quote(
		&self,
		quote_id: &str,
		recv_window: Option<u32>,
	) -> Result<ConvertOrder, Error> {
		let endpoint = format!("{}/acceptQuote", CONVERT_API);
		let mut params = HashMap::new();
		params.insert("quoteId".to_string(), quote_id.to_string());
		self.make_signed_request(&endpoint, Method::POST, Some(params), recv_window)
			.await
	}
}

impl BinanceApi {
	/// Create HMAC SHA256 signature for request parameters
	fn sign_request(&self, query_string: &str) -> String {
		let mut mac = Hmac::<Sha256>::new_from_slice(self.api_secret.as_bytes())
			.expect("HMAC can take key of any size");
		mac.update(query_string.as_bytes());
		let result = mac.finalize();
		let code_bytes = result.into_bytes();
		hex::encode(code_bytes)
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
		T: serde::de::DeserializeOwned,
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

		let response = request.send().await.map_err(|e| {
			error!("API request failed: {}", e);
			Error::RequestFailed
		})?;

		response.json::<T>().await.map_err(|e| {
			error!("Error parsing response: {}", e);
			Error::ParseResponseFailed
		})
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
		let binance_api = BinanceApi::new(api_key, api_secret, None);
		let signature = binance_api.sign_request(&query_string);

		assert_eq!(signature, "c8db56825ae71d6d79447849e617115f4a920fa2acdcab2b053c4b2838bd6b71");
	}
}
