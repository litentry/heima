
use serde::Serialize;

use crate::pumpx_api::PumpxApiClient;

use super::common::OrderInfoResponse;
// /v3/trade/cross_fail
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossFailBody {
	pub request_id: u32,
	pub fail_reason: String,
}

pub async fn cross_fail_impl(
	client: &PumpxApiClient,

	access_token: &str,
	data: CrossFailBody,
) -> Result<OrderInfoResponse, Error> {
	let endpoint = client.base_url.join("v3/trade/cross_fail").unwrap();
	let response = client
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
