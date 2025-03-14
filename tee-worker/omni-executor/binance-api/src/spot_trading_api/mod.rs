mod types;

use crate::{error::Error, BinanceApi};
use types::ServerTime;

/// https://developers.binance.com/docs/binance-spot-api-docs/rest-api/general-api-information
const SPOT_TRADING_API: &str = "/api/v3";

pub struct SpotTradingApi<'a> {
	base_api: &'a BinanceApi,
}

impl<'a> SpotTradingApi<'a> {
	pub fn new(binance_api: &BinanceApi) -> SpotTradingApi {
		SpotTradingApi { base_api: binance_api }
	}

	/// Test connectivity to the Rest API
	pub async fn test_connectivity(&self) -> Result<(), Error> {
		let endpoint = format!("{}/ping", SPOT_TRADING_API);
		self.base_api.make_public_get_request(&endpoint, None).await
	}

	/// Check server time
	pub async fn get_server_time(&self) -> Result<u64, Error> {
		let endpoint = format!("{}/time", SPOT_TRADING_API);
		let server_time: ServerTime =
			self.base_api.make_public_get_request(&endpoint, None).await?;
		Ok(server_time.server_time)
	}

}
