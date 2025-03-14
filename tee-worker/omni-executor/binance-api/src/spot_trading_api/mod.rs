use crate::{error::Error, BinanceApi};

/// https://developers.binance.com/docs/binance-spot-api-docs/rest-api/general-api-information
const SPOT_TRADING_API: &str = "/api/v3";

pub struct SpotTradingApi<'a> {
	base_api: &'a BinanceApi,
}

impl<'a> SpotTradingApi<'a> {
	pub fn new(binance_api: &BinanceApi) -> SpotTradingApi {
		SpotTradingApi { base_api: binance_api }
	}
}
