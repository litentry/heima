use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::types::ApiResponse;

// /v3/account/verify_google_code
#[derive(Serialize)]
pub struct GoogleCode {
    pub google_code: String,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct VerifyGoogleCodeResponseData {
    pub result: Option<bool>,
}

pub type VerifyGoogleCodeResponse = ApiResponse<VerifyGoogleCodeResponseData>;