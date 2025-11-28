#![allow(opaque_hidden_inferred_bound)]

use serde::{Deserialize, Serialize};
use warp::{http::Response, Filter};

const BASE_PATH: &str = "wildmeta";

#[derive(Serialize, Deserialize)]
struct HyperliquidLinkRequest {
	main_address: String,
	agent_address: String,
	login_type: i32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HyperliquidLinkResponseData {
	is_bound: bool,
}

#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
	code: i32,
	message: String,
	data: T,
}

pub(crate) fn handle() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
	warp::post()
		.and(warp::path(BASE_PATH))
		.and(warp::path!("v1" / "account" / "check_hyper_agent_address"))
		.and(warp::body::json())
		.map(|_request: HyperliquidLinkRequest| {
			let response_data = HyperliquidLinkResponseData {
				is_bound: true, // Always return true for testing
			};

			let api_response =
				ApiResponse { code: 200, message: "Success".to_string(), data: response_data };

			Response::builder()
				.status(200)
				.header("content-type", "application/json")
				.body(serde_json::to_string(&api_response).unwrap())
				.unwrap()
		})
}
