use crate::pumpx_api::pumpx_types::get_user_trade_info::UserTradeInfoResponse;
use reqwest::Error;

pub async fn get_user_trade_info(
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