// Copyright 2020-2023 Litentry Technologies GmbH.
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

use crate::{UserShieldingKeyType, MOCK_VERIFICATION_NONCE};
use ita_stf::helpers::get_expected_raw_message;
use lc_data_providers::twitter_official::*;
use litentry_primitives::{Identity, IdentityString, Web2Network};
use sp_core::{sr25519::Pair as Sr25519Pair, Pair};
use std::{collections::HashMap, sync::Arc};
use warp::{http::Response, Filter};

pub(crate) fn query_tweet<F>(
	func: Arc<F>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
where
	F: Fn(&Sr25519Pair) -> UserShieldingKeyType + Send + Sync + 'static,
{
	warp::get()
		.and(warp::path!("2" / "tweets" / u32))
		.and(warp::query::<HashMap<String, String>>())
		.map(move |tweet_id: u32, p: HashMap<String, String>| {
			println!("query_tweet, tweet_id: {}", tweet_id);
			let default = String::default();
			let ids = tweet_id.to_string();
			let expansions = p.get("expansions").unwrap_or(&default);

			if expansions.as_str() != "author_id" {
				Response::builder().status(400).body(String::from("Error query"))
			} else {
				let alice = Sr25519Pair::from_string("//Alice", None).unwrap();
				let twitter_identity = Identity::Web2 {
					network: Web2Network::Twitter,
					address: IdentityString::try_from("mock_user".as_bytes().to_vec()).unwrap(),
				};
				let key = func(&alice);
				let payload = hex::encode(get_expected_raw_message(
					&alice.public(),
					&twitter_identity,
					// the tweet_id is used as sidechain_nonce
					// it's a bit tricky to get the nonce from the getter: you need to know
					// the enclave signer account when launching the mock-server thread
					// the enclaveApi doesn't provide such interface
					tweet_id,
					key,
					MOCK_VERIFICATION_NONCE,
				));

				println!("query_tweet, payload: {}", payload);

				let tweet = Tweet { author_id: "mock_user".into(), id: ids.clone(), text: payload };
				let twitter_users = TwitterUsers {
					users: vec![TwitterUser {
						id: ids,
						name: "mock_user".to_string(),
						// intentionally return username with a different case, which shouldn't fail the verification
						// see https://github.com/litentry/litentry-parachain/issues/1680
						username: "Mock_User".to_string(),
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

pub(crate) fn query_friendship(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	warp::get()
		.and(warp::path!("1.1" / "friendships" / "show.json"))
		.and(warp::query::<HashMap<String, String>>())
		.map(move |p: HashMap<String, String>| {
			log::info!("query_friendship");
			let default = String::default();
			let source = p.get("source_screen_name").unwrap_or(&default);
			let target_id = p.get("target_id").unwrap_or(&default);

			if source != "twitterdev" || target_id != "783214" {
				Response::builder().status(400).body(String::from("Error query"))
			} else {
				let source_user = SourceTwitterUser {
					id_str: "2244994945".into(),
					screen_name: "TwitterDev".into(),
					following: true,
					followed_by: false,
				};

				let target_user = TargetTwitterUser {
					id_str: "783214".into(),
					screen_name: "Twitter".into(),
					following: false,
					followed_by: true,
				};

				let body = Relationship { source: source_user, target: target_user };
				Response::builder().body(serde_json::to_string(&body).unwrap())
			}
		})
}

pub(crate) fn query_user(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	warp::get()
		.and(warp::path!("2" / "users" / "by" / "username" / String))
		.and(warp::query::<HashMap<String, String>>())
		.map(move |user_name, p: HashMap<String, String>| {
			let expected_user_name = "twitterdev".to_string();

			let default = String::default();
			let user_fields = p.get("user.fields").unwrap_or(&default);

			if user_fields.as_str() != "public_metrics" || user_name != expected_user_name {
				Response::builder().status(400).body(String::from("Error query"))
			} else {
				let twitter_user_data = TwitterUser {
					id: "2244994945".into(),
					name: "TwitterDev".to_string(),
					username: "TwitterDev".to_string(),
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
