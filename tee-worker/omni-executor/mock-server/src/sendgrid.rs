#![allow(opaque_hidden_inferred_bound)]

use warp::{http::Response, Filter};

const BASE_PATH: &str = "sendgrid";

pub(crate) fn handle() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
	warp::post()
		.and(warp::path(BASE_PATH))
		.and(warp::path!("v3" / "mail" / "send"))
		.map(|| Response::builder().status(200).body("").unwrap())
}
