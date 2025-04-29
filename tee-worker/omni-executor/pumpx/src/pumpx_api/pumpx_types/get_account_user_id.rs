use crate::pumpx_types::common::ApiResponse;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

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
