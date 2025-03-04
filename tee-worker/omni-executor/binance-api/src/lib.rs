mod types;
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
}
