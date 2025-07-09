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
		.and(warp::path(BASE_PATH))
		.and(warp::path!("v3" / "account" / "user_connect"))
		.map(|| {
			// TODO implements your response logic
			Response::builder().status(200).body("").unwrap()
		});

	let verify_google_code = warp::post()
		.and(warp::path(BASE_PATH))
		.and(warp::path!("v3" / "account" / "verify_google_code"))
		.map(|| {
			// TODO implements your response logic
			Response::builder().status(200).body("").unwrap()
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

	// mock other APIs you need here

	user_connect.or(verify_google_code).or(post_heima_login).or(add_wallet)
}
