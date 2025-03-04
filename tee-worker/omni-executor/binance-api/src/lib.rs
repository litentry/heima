mod error;
mod types;

use error::BinanceApiError;
use log::error;
use types::TokenPair;
use url::Url;

pub struct BinanceApi {
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
		BinanceApi { base_url, api_key, api_secret }
	}

	/// List All Convert Pairs
	/// Query for all convertible token pairs and the tokens’ respective upper/lower limits
	pub async fn get_exchange_info(
		&self,
		from_asset: Option<String>,
		to_asset: Option<String>,
	) -> Result<Vec<TokenPair>, BinanceApiError> {
		let mut url = self.base_url.join("/sapi/v1/convert/exchangeInfo").unwrap();
		if let Some(from_asset) = from_asset {
			url.query_pairs_mut().append_pair("fromAsset", &from_asset);
		}
		if let Some(to_asset) = to_asset {
			url.query_pairs_mut().append_pair("toAsset", &to_asset);
		}
		let response = reqwest::get(url.as_str()).await.map_err(|e| {
			error!("Error getting exchange info: {}", e);
			BinanceApiError::RequestError
		})?;
		let token_pairs: Vec<TokenPair> = response.json().await.map_err(|e| {
			error!("Error parsing exchange info: {}", e);
			BinanceApiError::ParseResponseFailed
		})?;

		Ok(token_pairs)
	}
}
