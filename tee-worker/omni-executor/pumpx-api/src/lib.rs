mod types;

use reqwest::{
	header::{HeaderMap, HeaderValue},
};
use url::Url;

const DEFAULT_BASE_URL: &str = "https://api.pumpx.ai";

pub struct PumpxApi {
	http_client: Client,
	base_url: Url,
}

impl PumpxApi {
	pub fn new(api_key: String, base_url: Option<&str>) -> Self {
		let base_url = match base_url {
			Some(url) => Url::parse(url).expect("Invalid base URL"),
			None => Url::parse(DEFAULT_BASE_URL).unwrap(),
		};
		let mut headers = HeaderMap::new();
		headers.insert("Authorization", HeaderValue::from_str(&api_key).unwrap());
		headers.insert("X-Language", HeaderValue::from_static("en"));
		let http_client = Client::builder()
			.default_headers(headers)
			.build()
			.expect("Failed to build HTTP client");
		PumpxApi { http_client, base_url }
	}
}
