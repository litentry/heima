#![allow(opaque_hidden_inferred_bound)]

use serde_json::json;
use warp::{http::Response, Filter};

const BASE_PATH: &str = "pumpx";

fn build_success_response(response: serde_json::Value) -> Response<String> {
	Response::builder()
		.status(200)
		.header("content-type", "application/json")
		.body(response.to_string())
		.unwrap()
}

pub(crate) fn handle() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
	let user_connect = warp::post()
		.and(warp::path!("v3" / "account" / "user_connect"))
		.map(|| {
			// TODO implements your response logic
			Response::builder().status(200).body("").unwrap()
		});

	let verify_google_code = warp::post()
		.and(warp::path!("v3" / "account" / "verify_google_code"))
		.and(warp::body::json())
		.map(|body: serde_json::Value| {
			let response = json!({
				"code": 10000,
				"message": "OK",
				"data": {}
			});

			build_success_response(response)
		});

	let post_heima_login = warp::post()
		.and(warp::path!("v3" / "account" / "post_heima_login"))
		.and(warp::body::json())
		.map(|body: serde_json::Value| {
			log::info!("Received post_heima_login request: {:?}", body);

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
		.and(warp::path!("v3" / "account" / "add_wallet"))
		.and(warp::header::optional::<String>("authorization"))
		.map(|auth: Option<String>| {
			log::info!("Received add_wallet request with auth: {:?}", auth);

			let response = json!({
				"code": 10000,
				"message": "OK",
				"data": {}
			});

			build_success_response(response)
		});

	let create_transfer_tx = warp::post()
		.and(warp::path!("v3" / "trade" / "create_transfer_tx"))
		.and(warp::header::optional::<String>("authorization"))
		.and(warp::body::json())
		.map(|auth: Option<String>, body: serde_json::Value| {
			log::info!(
				"Received create_transfer_tx request with auth: {:?}, body: {:?}",
				auth,
				body
			);

			let response = json!({
				"code": 10000,
				"message": "OK",
				"data": {
					"tx_hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
					"transfer_id": 12345,
					"chain_id": body.get("chainId").unwrap_or(&json!(1))
				}
			});

			build_success_response(response)
		});

	let get_user_trade_info = warp::get()
		.and(warp::path!("v3" / "trade" / "get_user_trade_info"))
		.and(warp::header::optional::<String>("authorization"))
		.and(warp::query::raw())
		.map(|auth: Option<String>, _query: String| {
			log::info!("Received get_user_trade_info request with auth: {:?}", auth);

			let response = json!({
				"code": 10000,
				"message": "OK",
				"data": {
					"user_id": "test_user",
					"trade_enabled": true,
					"balance": "1000000000000000000"
				}
			});

			build_success_response(response)
		});

	let create_limit_order = warp::post()
		.and(warp::path!("v3" / "trade" / "create_limit_order"))
		.and(warp::header::optional::<String>("authorization"))
		.and(warp::body::json())
		.map(|auth: Option<String>, body: serde_json::Value| {
			log::info!(
				"Received create_limit_order request with auth: {:?}, body: {:?}",
				auth,
				body
			);

			let response = json!({
				"code": 10000,
				"message": "OK",
				"data": {
					"order_id": 123456
				}
			});

			build_success_response(response)
		});

	let create_market_order_unsigned_tx = warp::post()
		.and(warp::path!("v3" / "trade" / "create_market_order_unsigned_tx"))
		.and(warp::header::optional::<String>("authorization"))
		.and(warp::body::json())
		.map(|auth: Option<String>, body: serde_json::Value| {
			log::info!(
				"Received create_market_order_unsigned_tx request with auth: {:?}, body: {:?}",
				auth,
				body
			);

			let response = json!({
				"code": 10000,
				"message": "OK",
				"data": {
					"unsigned_tx": ["0x1234567890abcdef"],
					"tx_hash": ["0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"]
				}
			});

			build_success_response(response)
		});

	let send_order_tx = warp::post()
		.and(warp::path!("v3" / "trade" / "send_order_tx"))
		.and(warp::header::optional::<String>("authorization"))
		.and(warp::body::json())
		.map(|auth: Option<String>, body: serde_json::Value| {
			log::info!("Received send_order_tx request with auth: {:?}, body: {:?}", auth, body);

			let response = json!({
				"code": 10000,
				"message": "OK",
				"data": {
					"tx_hash": ["0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"]
				}
			});

			build_success_response(response)
		});

	// mock other APIs you need here

	user_connect
		.or(verify_google_code)
		.or(post_heima_login)
		.or(add_wallet)
		.or(create_transfer_tx)
		.or(get_user_trade_info)
		.or(create_limit_order)
		.or(create_market_order_unsigned_tx)
		.or(send_order_tx)
}
