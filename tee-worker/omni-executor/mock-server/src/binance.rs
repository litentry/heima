#![allow(opaque_hidden_inferred_bound)]

use warp::{http::Response, Filter};

const BASE_PATH: &str = "binance";

pub(crate) fn handle() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
	// Convert API
	let get_exchange_info = warp::get()
		.and(warp::path(BASE_PATH))
		.and(warp::path!("sapi" / "v1" / "convert" / "exchangeInfo"))
		.map(|| {
			// TODO implements your response logic
			Response::builder().status(200).body("").unwrap()
		});
	// mock other Convert APIs you need here

	// Spot Trading API
	let create_order = warp::get()
		.and(warp::path(BASE_PATH))
		.and(warp::path!("api" / "v3" / "order"))
		.map(|| {
			// TODO implements your response logic
			Response::builder().status(200).body("").unwrap()
		});
	// mock other Spot Trading APIs you need here

	// Wallet API
	let get_all_coins_info = warp::get()
		.and(warp::path(BASE_PATH))
		.and(warp::path!("sapi" / "v1" / "capital" / "config" / "getall"))
		.map(|| {
			// TODO implements your response logic
			Response::builder().status(200).body("").unwrap()
		});
	// mock other WalletAPIs you need here

	get_exchange_info.or(create_order).or(get_all_coins_info)
}
