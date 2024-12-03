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
extern crate sgx_tstd as std;

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

mod discord;
pub mod email;
pub mod google;
pub mod twitter;

use crate::{ensure, Error, Result};
use email::VerificationCodeStore;
use itp_sgx_crypto::ShieldingCryptoDecrypt;
use itp_utils::stringify::account_id_to_string;
use lc_data_providers::{
	discord_official::{DiscordMessage, DiscordOfficialClient, DiscordUserAccessTokenData},
	twitter_official::{Tweet, TwitterOfficialClient, TwitterUserAccessTokenData},
	vec_to_string, DataProviderConfig, UserInfo,
};
use lc_omni_account::InMemoryStore as OmniAccountStore;
use litentry_primitives::{
	DiscordValidationData, ErrorDetail, ErrorString, Identity, IntoErrorDetail,
	TwitterValidationData, Web2ValidationData,
};
use log::*;
use std::{string::ToString, vec::Vec};

pub trait DecryptionVerificationPayload<K: ShieldingCryptoDecrypt> {
	fn decrypt_ciphertext(&self, key: K) -> Result<Vec<u8>>;
}

pub fn verify(
	who: &Identity,
	identity: &Identity,
	raw_msg: &[u8],
	validation_data: &Web2ValidationData,
	config: &DataProviderConfig,
) -> Result<()> {
	debug!("verify web2 identity, who: {:?}", who);

	let username = match validation_data {
		Web2ValidationData::Twitter(data) => match data {
			TwitterValidationData::PublicTweet { ref tweet_id } => {
				let mut client = TwitterOfficialClient::v2(
					config.twitter_official_url.as_str(),
					config.twitter_auth_token_v2.as_str(),
				);
				let tweet: Tweet = client
					.query_tweet(tweet_id.to_vec())
					.map_err(|e| Error::LinkIdentityFailed(e.into_error_detail()))?;

				let user_id = tweet
					.get_user_id()
					.ok_or(Error::LinkIdentityFailed(ErrorDetail::WrongWeb2Handle))?;
				let user = client
					.query_user_by_id(user_id.into_bytes())
					.map_err(|e| Error::LinkIdentityFailed(e.into_error_detail()))?;

				let payload = twitter::payload_from_tweet(&tweet)?;
				ensure!(
					payload.as_slice() == raw_msg,
					Error::LinkIdentityFailed(ErrorDetail::UnexpectedMessage)
				);

				Ok(user.username)
			},
			TwitterValidationData::OAuth2 { code, state, redirect_uri } => {
				let authorization = TwitterOfficialClient::oauth2_authorization(
					&config.twitter_client_id,
					&config.twitter_client_secret,
				);
				let mut oauth2_client =
					TwitterOfficialClient::v2(&config.twitter_official_url, &authorization);
				let redirect_uri = vec_to_string(redirect_uri.to_vec())
					.map_err(|e| Error::LinkIdentityFailed(e.into_error_detail()))?;
				let code = vec_to_string(code.to_vec())
					.map_err(|e| Error::LinkIdentityFailed(e.into_error_detail()))?;
				let state = vec_to_string(state.to_vec())
					.map_err(|e| Error::LinkIdentityFailed(e.into_error_detail()))?;
				let Some(account_id) = who.to_native_account() else {
					return Err(Error::LinkIdentityFailed(ErrorDetail::ParseError));
				};
				let (code_verifier, state_verifier) =
					match twitter::OAuthStore::get_data(&account_id) {
						Ok(data) => data.ok_or_else(|| {
							Error::LinkIdentityFailed(ErrorDetail::StfError(
								ErrorString::truncate_from(
									std::format!(
										"no oauth data found for {}",
										account_id_to_string(&account_id)
									)
									.as_bytes()
									.to_vec(),
								),
							))
						})?,
						Err(e) =>
							return Err(Error::LinkIdentityFailed(ErrorDetail::StfError(
								ErrorString::truncate_from(
									std::format!("failed to get oauth data: {}", e)
										.as_bytes()
										.to_vec(),
								),
							))),
					};

				ensure!(
					state == state_verifier,
					Error::LinkIdentityFailed(ErrorDetail::StfError(ErrorString::truncate_from(
						"stored state mismatch".as_bytes().to_vec()
					)))
				);

				let data = TwitterUserAccessTokenData {
					client_id: config.twitter_client_id.clone(),
					code,
					code_verifier,
					redirect_uri,
				};
				let user_token = oauth2_client.request_user_access_token(data).map_err(|e| {
					Error::LinkIdentityFailed(ErrorDetail::StfError(ErrorString::truncate_from(
						e.to_string().as_bytes().to_vec(),
					)))
				})?;

				let user_authorization = std::format!("Bearer {}", user_token.access_token);
				let mut user_client =
					TwitterOfficialClient::v2(&config.twitter_official_url, &user_authorization);

				let user = user_client
					.query_user_by_id("me".to_string().into_bytes())
					.map_err(|e| Error::LinkIdentityFailed(e.into_error_detail()))?;

				Ok(user.username)
			},
		},
		Web2ValidationData::Discord(data) => match data {
			DiscordValidationData::PublicMessage { ref channel_id, ref message_id, .. } => {
				let mut client = DiscordOfficialClient::new(config);
				let message: DiscordMessage = client
					.query_message(channel_id.to_vec(), message_id.to_vec())
					.map_err(|e| Error::LinkIdentityFailed(e.into_error_detail()))?;

				let user = client
					.get_user_info(message.author.id.clone())
					.map_err(|e| Error::LinkIdentityFailed(e.into_error_detail()))?;

				let mut username = message.author.username.clone();
				// if discord user's username is upgraded complete, the discriminator value from api will be "0".
				if user.discriminator != "0" {
					username.push_str(&'#'.to_string());
					username.push_str(&user.discriminator);
				}
				let payload = discord::payload_from_discord(&message)?;
				ensure!(
					payload.as_slice() == raw_msg,
					Error::LinkIdentityFailed(ErrorDetail::UnexpectedMessage)
				);

				Ok(username)
			},
			DiscordValidationData::OAuth2 { code, redirect_uri } => {
				let mut client = DiscordOfficialClient::new(config);

				let redirect_uri = vec_to_string(redirect_uri.to_vec())
					.map_err(|e| Error::LinkIdentityFailed(e.into_error_detail()))?;
				let code = vec_to_string(code.to_vec())
					.map_err(|e| Error::LinkIdentityFailed(e.into_error_detail()))?;
				let data = DiscordUserAccessTokenData {
					client_id: config.discord_client_id.clone(),
					client_secret: config.discord_client_secret.clone(),
					code,
					redirect_uri,
				};
				let user_token = client
					.request_user_access_token(data)
					.map_err(|e| Error::LinkIdentityFailed(e.into_error_detail()))?;
				let mut user_client = DiscordOfficialClient::with_access_token(
					&config.discord_official_url,
					&user_token.token_type,
					&user_token.access_token,
				);
				let user = user_client
					.get_user_info("@me".to_string())
					.map_err(|e| Error::LinkIdentityFailed(e.into_error_detail()))?;

				Ok(user.username)
			},
		},
		Web2ValidationData::Email(data) => {
			let email = vec_to_string(data.email.to_vec())
				.map_err(|e| Error::LinkIdentityFailed(e.into_error_detail()))?;
			let verification_code = vec_to_string(data.verification_code.to_vec())
				.map_err(|e| Error::LinkIdentityFailed(e.into_error_detail()))?;
			let account_id = match OmniAccountStore::get_omni_account(who.hash()) {
				Ok(Some(account)) => account,
				_ => return Err(Error::LinkIdentityFailed(ErrorDetail::InvalidIdentity)),
			};

			let email_identity =
				Identity::from_web2_account(&email, litentry_primitives::Web2IdentityType::Email);
			let stored_verification_code =
				match VerificationCodeStore::get(&account_id, email_identity.hash()) {
					Ok(data) => data.ok_or_else(|| {
						Error::LinkIdentityFailed(ErrorDetail::StfError(
							ErrorString::truncate_from(
								std::format!(
									"no verification code found for {}:{}",
									account_id_to_string(&account_id),
									&email
								)
								.as_bytes()
								.to_vec(),
							),
						))
					})?,
					Err(e) => return Err(Error::LinkIdentityFailed(e.into_error_detail())),
				};

			ensure!(
				verification_code == stored_verification_code,
				Error::LinkIdentityFailed(ErrorDetail::StfError(ErrorString::truncate_from(
					"verification code mismatch".as_bytes().to_vec()
				)))
			);

			Ok(email)
		},
	}?;

	// compare the username:
	// - twitter's username is case insensitive
	// - discord's username is case sensitive
	match identity {
		Identity::Twitter(address) => {
			let handle = std::str::from_utf8(address.inner_ref())
				.map_err(|_| Error::LinkIdentityFailed(ErrorDetail::ParseError))?;
			ensure!(
				username.to_ascii_lowercase().eq(&handle.to_string().to_ascii_lowercase()),
				Error::LinkIdentityFailed(ErrorDetail::WrongWeb2Handle)
			);
		},
		Identity::Discord(address) => {
			let handle = std::str::from_utf8(address.inner_ref())
				.map_err(|_| Error::LinkIdentityFailed(ErrorDetail::ParseError))?;
			ensure!(username.eq(handle), Error::LinkIdentityFailed(ErrorDetail::WrongWeb2Handle));
		},
		Identity::Email(address) => {
			let handle = std::str::from_utf8(address.inner_ref())
				.map_err(|_| Error::LinkIdentityFailed(ErrorDetail::ParseError))?;
			ensure!(username.eq(handle), Error::LinkIdentityFailed(ErrorDetail::WrongWeb2Handle));
		},
		_ => return Err(Error::LinkIdentityFailed(ErrorDetail::InvalidIdentity)),
	}

	Ok(())
}
