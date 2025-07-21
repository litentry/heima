#![allow(opaque_hidden_inferred_bound)]

use crate::dex;
use serde_json::{json, Value};
use warp::{http::Response, Filter};

const BASE_PATH: &str = "pumpx";

fn build_success_response(response: serde_json::Value) -> Response<warp::hyper::Body> {
	Response::builder()
		.status(200)
		.header("content-type", "application/json")
		.body(warp::hyper::Body::from(response.to_string()))
		.unwrap()
}

pub(crate) fn handle() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
	let user_connect = warp::post()
		.and(warp::path(BASE_PATH))
		.and(warp::path!("v3" / "account" / "user_connect"))
		.and(warp::body::json())
		.map(|body: serde_json::Value| {
			tracing::info!("Received user_connect request: {:?}", body);

			let response = json!({
				"code": 10000,
				"message": "OK",
				"data": {
					"userId": "mock_user_123",
					"googleAuthCheck": true
				}
			});

			build_success_response(response)
		});

	let verify_google_code = warp::post().and(warp::body::json()).map(|body: serde_json::Value| {
		tracing::info!("Received verify_google_code request: {:?}", body);

		let response = json!({
			"code": 10000,
			"message": "OK",
			"data": {
				"result": true
			}
		});

		build_success_response(response)
	});

	let post_heima_login = warp::post()
		.and(warp::path(BASE_PATH))
		.and(warp::path!("v3" / "account" / "post_heima_login"))
		.and(warp::body::json())
		.map(|body: serde_json::Value| {
			tracing::info!("Received post_heima_login request: {:?}", body);

			let response = json!({
				"code": 10000,
				"message": "OK",
				"data": {
					"user_id": null
				}
			});

			build_success_response(response)
		});

	let add_wallet = warp::post()
		.and(warp::path(BASE_PATH))
		.and(warp::path!("v3" / "account" / "add_wallet"))
		.and(warp::header::optional::<String>("authorization"))
		.map(|auth: Option<String>| {
			tracing::info!("Received add_wallet request with auth: {:?}", auth);

			let response = json!({
				"code": 10000,
				"message": "OK",
				"data": {}
			});

			build_success_response(response)
		});

	let get_account_user_id = warp::get()
		.and(warp::path(BASE_PATH))
		.and(warp::path!("v3" / "account" / "get_account_user_id"))
		.and(warp::query::<std::collections::HashMap<String, String>>())
		.map(|query: std::collections::HashMap<String, String>| {
			tracing::info!("Received get_account_user_id request with query: {:?}", query);

			let response = json!({
				"code": 10000,
				"message": "OK",
				"data": {
					"userId": "mock_user_123"
				}
			});

			build_success_response(response)
		});

	let create_transfer_tx = warp::post()
		.and(warp::path(BASE_PATH))
		.and(warp::path!("v3" / "account" / "create_transfer_tx"))
		.and(warp::header::optional::<String>("authorization"))
		.and(warp::body::json())
		.map(|auth: Option<String>, body: serde_json::Value| {
			tracing::info!(
				"Received create_transfer_tx request with auth: {:?}, body: {:?}",
				auth,
				body
			);

			let response = json!({
				"code": 10000,
				"message": "OK",
				"data": {
					"txHash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
					"status": "success"
				}
			});

			build_success_response(response)
		});

	// JSON-RPC endpoints for dex signer
	let jsonrpc_handler = warp::post().and(warp::path("jsonrpc")).and(warp::body::json()).and_then(
		|body: Value| async move {
			tracing::info!("Received JSON-RPC request: {:?}", body);
			Ok::<_, warp::Rejection>(handle_jsonrpc_request(body).await)
		},
	);

	// mock other APIs you need here

	user_connect
		.or(verify_google_code)
		.or(post_heima_login)
		.or(add_wallet)
		.or(get_account_user_id)
		.or(create_transfer_tx)
		.or(jsonrpc_handler)
}

async fn handle_jsonrpc_request(request: Value) -> Box<dyn warp::Reply> {
	let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");

	// Check if it's a dex_* method and delegate to dex module
	if method.starts_with("dex_") {
		return match dex::handle_dex_method(request).await {
			Ok(response) => response,
			Err(_) => {
				let error_response = json!({
					"jsonrpc": "2.0",
					"error": {
						"code": -32603,
						"message": "Internal error"
					},
					"id": 1
				});
				Box::new(warp::reply::with_status(
					warp::reply::json(&error_response),
					warp::http::StatusCode::OK,
				))
			},
		};
	}

	// Handle other pumpx-specific methods here
	let id = request.get("id").cloned().unwrap_or(json!(1));
	let response = json!({
		"jsonrpc": "2.0",
		"error": {
			"code": -32601,
			"message": "Method not found"
		},
		"id": id
	});
	Box::new(warp::reply::with_status(warp::reply::json(&response), warp::http::StatusCode::OK))
}
