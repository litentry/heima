// Copyright 2020-2024 Trust Computing GmbH.
// This file is part of Litentry.
//
// Litentry is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Litentry is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Litentry.  If not, see <https://www.gnu.org/licenses/>.
#![allow(opaque_hidden_inferred_bound)]

use ita_stf::helpers::get_expected_raw_message;
use lc_data_providers::twitter_official::*;
use litentry_primitives::{Identity, IdentityString};
use sp_core::{sr25519::Pair as Sr25519Pair, Pair};
use std::collections::HashMap;
use warp::{http::Response, Filter};

pub(crate) fn query_tweet(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	warp::get()
		.and(warp::path!("2" / "tweets" / u32))
		.and(warp::query::<HashMap<String, String>>())
		.map(move |tweet_id: u32, p: HashMap<String, String>| {
			println!("query_tweet, tweet_id: {}", tweet_id);
			let default = String::default();
			let id = tweet_id.to_string();
			let expansions = p.get("expansions").unwrap_or(&default);

			let tweet_author_name = "mock_user";
			let tweet_author_id = "mock_user_id";

			if expansions.as_str() != "author_id" {
				Response::builder().status(400).body(String::from("Error query"))
			} else {
				let alice = Sr25519Pair::from_string("//Alice", None).unwrap();
				let twitter_identity =
					Identity::Twitter(IdentityString::new(tweet_author_name.as_bytes().to_vec()));
				let payload = hex::encode(get_expected_raw_message(
					&alice.public().into(),
					&twitter_identity,
					// the tweet_id is used as sidechain_nonce
					// it's a bit tricky to get the nonce from the getter: you need to know
					// the enclave signer account when launching the mock-server thread
					// the enclaveApi doesn't provide such interface
					tweet_id,
				));

				println!("query_tweet, payload: {}", payload);

				let tweet = Tweet { author_id: tweet_author_id.into(), id, text: payload };
				let twitter_users = TwitterUsers {
					users: vec![TwitterUser {
						id: tweet_author_id.to_string(),
						name: tweet_author_name.to_string(),
						// intentionally return username with a different case, which shouldn't fail the verification
						// see https://github.com/litentry/heima/issues/1680
						username: tweet_author_name.to_string().to_uppercase(),
						public_metrics: None,
					}],
				};
				let body = TwitterAPIV2Response {
					data: Some(tweet),
					meta: None,
					includes: Some(twitter_users),
				};

				Response::builder().body(serde_json::to_string(&body).unwrap())
			}
		})
}

pub(crate) fn query_retweeted_by(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	warp::get()
		.and(warp::path!("2" / "tweets" / String / "retweeted_by"))
		.and(warp::query::<HashMap<String, String>>())
		.map(move |original_tweet_id, _p: HashMap<String, String>| {
			log::info!("query_retweeted_by");
			if original_tweet_id != "100" {
				Response::builder().status(400).body(String::from("Error query"))
			} else {
				let author = "litentry";
				let user_id = "100";

				let tweets = vec![TwitterUser {
					id: user_id.into(),
					name: author.into(),
					username: author.into(),
					public_metrics: None,
				}];
				let body = Retweeted { data: tweets };
				Response::builder().body(serde_json::to_string(&body).unwrap())
			}
		})
}

pub(crate) fn query_user_by_name(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	warp::get()
		.and(warp::path!("2" / "users" / "by" / "username" / String))
		.and(warp::query::<HashMap<String, String>>())
		.map(move |user_name: String, p: HashMap<String, String>| {
			let default = String::default();
			let user_fields = p.get("user.fields").unwrap_or(&default);

			if user_fields.as_str() != "public_metrics" {
				return Response::builder().status(400).body(String::from("Error query"))
			}

			let user: Option<TwitterUser> = match user_name.as_str() {
				"twitterdev" => Some(TwitterUser {
					id: "2244994945".into(),
					name: "TwitterDev".to_string(),
					username: "TwitterDev".to_string(),
					public_metrics: Some(TwitterUserPublicMetrics {
						followers_count: 100_u32,
						following_count: 99_u32,
					}),
				}),
				"mock_user" => Some(TwitterUser {
					id: "mock_user_id".into(),
					name: "MockUser".to_string(),
					username: "MockUser".to_string(),
					public_metrics: Some(TwitterUserPublicMetrics {
						followers_count: 100_u32,
						following_count: 99_u32,
					}),
				}),
				_ => None,
			};
			let body: TwitterAPIV2Response<TwitterUser> =
				TwitterAPIV2Response { data: user, meta: None, includes: None };
			Response::builder().body(serde_json::to_string(&body).unwrap())
		})
}

pub(crate) fn query_user_by_id(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	warp::get()
		.and(warp::path!("2" / "users" / String))
		.and(warp::query::<HashMap<String, String>>())
		.map(move |id, p: HashMap<String, String>| {
			let expected_user_ids =
				vec!["2244994945".to_string(), "mock_user_id".to_string(), "me".to_string()];

			let mut user_names = HashMap::new();
			user_names.insert("2244994945", "TwitterDev".to_string());
			user_names.insert("mock_user_id", "mock_user".to_string());
			user_names.insert("me", "mock_user_me".to_string());

			let default = String::default();
			let user_fields = p.get("user.fields").unwrap_or(&default);

			if user_fields.as_str() != "public_metrics" || !expected_user_ids.contains(&id) {
				Response::builder().status(400).body(String::from("Error query"))
			} else {
				let user_name = user_names.get(id.as_str()).unwrap().to_string();

				let twitter_user_data = TwitterUser {
					id,
					name: user_name.clone(),
					username: user_name,
					public_metrics: Some(TwitterUserPublicMetrics {
						followers_count: 100_u32,
						following_count: 99_u32,
					}),
				};
				let body = TwitterAPIV2Response {
					data: Some(twitter_user_data),
					meta: None,
					includes: None,
				};
				Response::builder().body(serde_json::to_string(&body).unwrap())
			}
		})
}

pub(crate) fn request_user_access_token(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	warp::post()
		.and(warp::path!("2" / "oauth2" / "token"))
		.and(warp::body::form())
		.map(|_: HashMap<String, String>| {
			let user_access_token = TwitterUserAccessToken {
				token_type: "bearer".to_string(),
				expires_in: 7200,
				access_token: "dGFxeU1MbWRlSVhxSUgxX3VUdUJrM1FTRUtaMmFPdFM0XzMzcVlFSi0xM1dyOjE3MTMzNDEwODQ5NTg6MToxOmF0OjE".to_string(),
				scope: "users.read tweet.read".to_string(),
			};
			let body = TwitterAPIV2Response {
				data: Some(user_access_token),
				meta: None,
				includes: None,
			};

			Response::builder()
				.header("Content-Type", "application/json")
				.body(serde_json::to_string(&body).unwrap())
		})
}
