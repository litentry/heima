use parity_scale_codec::{Decode, Encode};
use reqwest::Error;
use serde::{Deserialize, Serialize};

use crate::pumpx_api::PumpxApiClient;

use super::common::ApiResponse;

// /v3/account/get_account_user_id
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountUserIdParams {
	pub email: String,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountUserIdResponseData {
	pub user_id: Option<String>,
}
pub type GetAccountUserIdResponse = ApiResponse<GetAccountUserIdResponseData>;

pub async fn get_account_user_id_impl(
	client: &PumpxApiClient,
	email: String,
) -> Result<GetAccountUserIdResponse, Error> {
	let endpoint = client.base_url.join("v3/account/get_account_user_id").unwrap();
	let params = GetAccountUserIdParams { email };
	client.http_client.get(endpoint).query(&params).send().await?.json().await
}
