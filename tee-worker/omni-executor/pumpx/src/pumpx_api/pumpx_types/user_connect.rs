use serde::{Deserialize, Serialize};
use crate::pumpx_api::common::ApiResponse;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserConnectBody {
    pub email: String,
    pub invite_code: Option<String>,
    pub google_code: String,
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserConnectResponseData {
    pub user_id: Option<String>,
    pub google_auth_check: Option<bool>,
}

pub type UserConnectResponse = ApiResponse<UserConnectResponseData>;