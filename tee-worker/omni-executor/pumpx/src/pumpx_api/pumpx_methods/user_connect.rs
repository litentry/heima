use crate::pumpx_api::pumpx_types::user_connect::{UserConnectBody, UserConnectResponse};
use reqwest::Error;
use crate::pumpx_api::PumpxApiClient;

pub async fn user_connect_impl(
    client: &PumpxApiClient,
    access_token: &str,
    user_id: String,
    email: String,
    invite_code: Option<String>,
    google_code: String,
    language: Option<String>,
) -> Result<UserConnectResponse, Error> {
    let endpoint = client.base_url.join("v3/account/user_connect").unwrap();
    let body = UserConnectBody { email, invite_code, google_code, user_id };
    let response = client
        .http_client
        .post(endpoint)
        .header("X-Language", language.unwrap_or("en".to_string()))
        .bearer_auth(access_token)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to send user connect request: {:?}", e);
            e
        })?;
    let status = response.status();
    let response = response.error_for_status().map_err(|e| {
        log::error!("User connect request failed with status: {}, error: {:?}", status, e);
        e
    })?;
    response.json().await.map_err(|e| {
        log::error!("Failed to parse user connect response: {:?}", e);
        e
    })
}