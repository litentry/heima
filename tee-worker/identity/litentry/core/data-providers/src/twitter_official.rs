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

#[cfg(all(not(feature = "std"), feature = "sgx"))]
use crate::sgx_reexport_prelude::*;

use crate::{build_client_with_cert, vec_to_string, Error, HttpError, UserInfo};
use http::header::{AUTHORIZATION, CONNECTION};
use http_req::response::Headers;
use itc_rest_client::{
	http_client::{HttpClient, SendWithCertificateVerification},
	rest_client::RestClient,
	RestGet, RestPath,
};
use log::*;
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
	format,
	string::{String, ToString},
	vec,
	vec::Vec,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct TwitterAPIV2Response<T> {
	pub data: Option<T>,
	pub meta: Option<ResponseMeta>,
	pub includes: Option<TwitterUsers>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseMeta {
	pub result_count: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tweet {
	pub author_id: String,
	pub id: String,
	pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Retweeted {
	pub data: Vec<TwitterUser>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TwitterUser {
	pub id: String,
	pub name: String,
	pub username: String,
	pub public_metrics: Option<TwitterUserPublicMetrics>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TwitterUserPublicMetrics {
	pub followers_count: u32,
	pub following_count: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TwitterUsers {
	pub users: Vec<TwitterUser>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Relationship {
	pub source: SourceTwitterUser,
	pub target: TargetTwitterUser,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SourceTwitterUser {
	pub id_str: String,
	pub screen_name: String,
	pub following: bool,
	pub followed_by: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TargetTwitterUser {
	pub id_str: String,
	pub screen_name: String,
	pub following: bool,
	pub followed_by: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TwitterUserAccessTokenData {
	pub client_id: String,
	pub code: String,
	pub code_verifier: String,
	pub redirect_uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TwitterUserAccessToken {
	pub access_token: String,
	pub token_type: String,
	pub expires_in: u32,
	pub scope: String,
}

impl RestPath<String> for Tweet {
	fn get_path(path: String) -> Result<String, HttpError> {
		Ok(path)
	}
}

impl<T> RestPath<String> for TwitterAPIV2Response<T> {
	fn get_path(path: String) -> Result<String, HttpError> {
		Ok(path)
	}
}

impl RestPath<String> for Relationship {
	fn get_path(path: String) -> Result<String, HttpError> {
		Ok(path)
	}
}

impl RestPath<String> for Retweeted {
	fn get_path(path: String) -> Result<String, HttpError> {
		Ok(path)
	}
}

impl UserInfo for Tweet {
	fn get_user_id(&self) -> Option<String> {
		Some(self.author_id.clone())
	}
}

pub struct TwitterOfficialClient {
	client: RestClient<HttpClient<SendWithCertificateVerification>>,
}

pub enum TargetUser {
	Name(Vec<u8>),
	Id(Vec<u8>),
}

impl TargetUser {
	pub fn to_query_param(self) -> Result<(String, String), Error> {
		match self {
			Self::Id(v) => {
				let id_as_string = vec_to_string(v)?;
				Ok(("target_id".to_string(), id_as_string))
			},
			Self::Name(v) => {
				let name_as_string = vec_to_string(v)?;
				Ok(("target_screen_name".to_string(), name_as_string))
			},
		}
	}
}

/// rate limit: https://developer.twitter.com/en/docs/twitter-api/rate-limits
impl TwitterOfficialClient {
	pub fn v2(url: &str, token: &str) -> Self {
		let mut headers = Headers::new();
		headers.insert(CONNECTION.as_str(), "close");
		headers.insert(AUTHORIZATION.as_str(), token);
		let client = build_client_with_cert(url, headers.clone());

		TwitterOfficialClient { client }
	}

	pub fn oauth2_authorization(client_id: &str, client_secret: &str) -> String {
		format!("Basic {}", base64::encode(format!("{}:{}", client_id, client_secret)))
	}

	/// V2, rate limit: 300/15min(per App) 900/15min(per User)
	pub fn query_tweet(&mut self, tweet_id: Vec<u8>) -> Result<Tweet, Error> {
		let tweet_id = vec_to_string(tweet_id)?;
		debug!("Twitter query tweet, id: {}", tweet_id);

		let path = format!("/2/tweets/{}", tweet_id);
		let query: Vec<(&str, &str)> = vec![("expansions", "author_id")];

		let resp = self
			.client
			.get_with::<String, TwitterAPIV2Response<Tweet>>(path, query.as_slice())
			.map_err(|e| Error::RequestError(format!("{:?}", e)))?;

		let mut tweet = resp.data.ok_or(Error::RequestError("tweet not found".into()))?;

		// have to replace user_id with includes -> users -> username, otherwise the handle verification would fail
		if let Some(tweet_users) = resp.includes {
			if tweet_users.users.is_empty() {
				return Err(Error::RequestError("user not found from tweet".to_string()))
			}
			tweet.author_id = tweet_users.users[0].id.clone();
		}

		Ok(tweet)
	}

	/// V2, https://developer.twitter.com/en/docs/twitter-api/tweets/retweets/api-reference/get-tweets-id-retweeted_by
	/// rate limit: 75/15min(per App) 75/15min(per User)
	/// Note: The maximum result is 100 latest, when a user requests too late (after 100 retweets by others),
	/// the verification will fail.
	pub fn query_retweeted_by(&mut self, original_tweet_id: Vec<u8>) -> Result<Retweeted, Error> {
		let original_tweet_id = vec_to_string(original_tweet_id)?;
		debug!("Twitter original tweet id: {}", original_tweet_id);

		let path = format!("/2/tweets/{}/retweeted_by", original_tweet_id);
		let query: Vec<(&str, &str)> = vec![("max_results", "100")];

		let resp = self
			.client
			.get_with::<String, Retweeted>(path, query.as_slice())
			.map_err(|e| Error::RequestError(format!("{:?}", e)))?;

		Ok(resp)
	}

	/// V2, rate limit: 300/15min(per App) 900/15min(per User)
	pub fn query_user_by_name(&mut self, user_name: Vec<u8>) -> Result<TwitterUser, Error> {
		let user = vec_to_string(user_name)?;
		debug!("Twitter query user by name, name: {}", user);

		let query = vec![("user.fields", "public_metrics")];
		let resp = self
			.client
			.get_with::<String, TwitterAPIV2Response<TwitterUser>>(
				format!("/2/users/by/username/{}", user),
				query.as_slice(),
			)
			.map_err(|e| Error::RequestError(format!("{:?}", e)))?;

		let user = resp.data.ok_or_else(|| Error::RequestError("user not found".to_string()))?;
		Ok(user)
	}

	/// V2, rate limit: 300/15min(per App) 900/15min(per User)
	pub fn query_user_by_id(&mut self, id: Vec<u8>) -> Result<TwitterUser, Error> {
		let id = vec_to_string(id)?;
		debug!("Twitter query user by id, id: {}", id);

		let query = vec![("user.fields", "public_metrics")];
		let resp = self
			.client
			.get_with::<String, TwitterAPIV2Response<TwitterUser>>(
				format!("/2/users/{}", id),
				query.as_slice(),
			)
			.map_err(|e| Error::RequestError(format!("{:?}", e)))?;

		let user = resp.data.ok_or_else(|| Error::RequestError("user not found".to_string()))?;
		Ok(user)
	}

	pub fn request_user_access_token(
		&mut self,
		data: TwitterUserAccessTokenData,
	) -> Result<TwitterUserAccessToken, Error> {
		debug!("Twitter create access token");

		let path = String::from("/2/oauth2/token");

		let mut body = HashMap::new();
		body.insert("client_id".to_string(), data.client_id);
		body.insert("code".to_string(), data.code);
		body.insert("code_verifier".to_string(), data.code_verifier);
		body.insert("redirect_uri".to_string(), data.redirect_uri);
		body.insert("grant_type".to_string(), "authorization_code".to_string());

		let resp = self
			.client
			.post_form_urlencoded_capture::<String, TwitterAPIV2Response<TwitterUserAccessToken>>(
				path, body,
			)
			.map_err(|e| Error::RequestError(format!("{:?}", e)))?;

		let token = resp
			.data
			.ok_or_else(|| Error::RequestError("could not get token from twitter".to_string()))?;

		Ok(token)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::DataProviderConfig;
	use lc_mock_server::run;

	fn init() -> DataProviderConfig {
		let _ = env_logger::builder().is_test(true).try_init();
		let url = run(0).unwrap();
		let mut data_provider_config = DataProviderConfig::new().unwrap();
		data_provider_config.set_twitter_official_url(url).unwrap();
		data_provider_config
	}

	#[test]
	fn test_oauth2_authorization() {
		let client_id = "Z24wcG85SXVJUy1ldE1wdVl3MlA6MTpjaY";
		let client_secret = "lYq3l-sMbGVk94iaze3j8G4ne1MBWAQ8pH4-L58yQ7y4mHOCgp";
		let token = TwitterOfficialClient::oauth2_authorization(client_id, client_secret);

		assert_eq!(token, "Basic WjI0d2NHODVTWFZKVXkxbGRFMXdkVmwzTWxBNk1UcGphWTpsWXEzbC1zTWJHVms5NGlhemUzajhHNG5lMU1CV0FROHBINC1MNTh5UTd5NG1IT0NncA==".to_string());
	}

	#[test]
	fn query_tweet_work() {
		let data_provider_config = init();
		let tweet_id = "100";

		let mut client =
			TwitterOfficialClient::v2(&data_provider_config.twitter_official_url, "token");
		let result = client.query_tweet(tweet_id.as_bytes().to_vec());
		assert!(result.is_ok(), "error: {:?}", result);
		let tweet = result.unwrap();

		assert_eq!(tweet.id, tweet_id);
		assert_eq!(tweet.author_id, "mock_user_id");
		assert_eq!(tweet.text, "74983bb19b0e9f7cc8446b0fc16d50855521add6c9574cdf7559830de4054e75")
	}

	#[test]
	fn query_retweeted_work() {
		let data_provider_config = init();

		let mut client =
			TwitterOfficialClient::v2(&data_provider_config.twitter_official_url, "token");
		let original_tweet_id = "100".as_bytes().to_vec();
		let response = client.query_retweeted_by(original_tweet_id);

		assert!(response.is_ok(), "error: {:?}", response);
	}

	#[test]
	fn query_user_by_name_work() {
		let data_provider_config = init();

		let user = "twitterdev";
		let mut client =
			TwitterOfficialClient::v2(&data_provider_config.twitter_official_url, "token");
		let result = client.query_user_by_name(user.as_bytes().to_vec());
		assert!(result.is_ok(), "error: {:?}", result);
	}

	#[test]
	fn query_user_by_id_work() {
		let data_provider_config = init();

		let user_id = "2244994945";
		let mut client =
			TwitterOfficialClient::v2(&data_provider_config.twitter_official_url, "token");
		let result = client.query_user_by_id(user_id.as_bytes().to_vec());
		assert!(result.is_ok(), "error: {:?}", result);
	}

	#[test]
	fn request_user_access_token_work() {
		let data_provider_config = init();

		let data = TwitterUserAccessTokenData {
			client_id: data_provider_config.twitter_client_id.clone(),
			code: "code".to_string(),
			code_verifier: "code_verifier".to_string(),
			redirect_uri: "redirect_uri".to_string(),
		};
		let authorization = TwitterOfficialClient::oauth2_authorization(
			&data_provider_config.twitter_client_id,
			&data_provider_config.twitter_client_secret,
		);
		let mut client = TwitterOfficialClient::v2(
			&data_provider_config.twitter_official_url,
			authorization.as_str(),
		);
		let result = client.request_user_access_token(data);

		assert!(result.is_ok(), "error: {:?}", result);
	}
}
