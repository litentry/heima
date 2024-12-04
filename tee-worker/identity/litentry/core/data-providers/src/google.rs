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

use crate::{build_client_with_cert, format, Error, Headers, HttpError, String, ToString};
use itc_rest_client::{
	http_client::{HttpClient, SendWithCertificateVerification},
	rest_client::RestClient,
	RestPath,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const OAUTH2_BASE_URL: &str = "https://oauth2.googleapis.com";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoogleOAuth2TokenResponse {
	access_token: String,
	expires_in: u64,
	id_token: String,
	scope: String,
	token_type: String,
}

impl RestPath<String> for GoogleOAuth2TokenResponse {
	fn get_path(path: String) -> Result<String, HttpError> {
		Ok(path)
	}
}

pub struct GoogleOAuth2Client {
	client: RestClient<HttpClient<SendWithCertificateVerification>>,
	client_id: String,
	client_secret: String,
}

impl GoogleOAuth2Client {
	pub fn new(client_id: String, client_secret: String) -> Self {
		let client = build_client_with_cert(OAUTH2_BASE_URL, Headers::new());
		Self { client, client_id, client_secret }
	}

	pub fn exchange_code_for_token(
		&mut self,
		code: String,
		redirect_uri: String,
	) -> Result<String, Error> {
		let path = String::from("/token");

		let mut body = HashMap::new();
		body.insert("code".to_string(), code);
		body.insert("client_id".to_string(), self.client_id.clone());
		body.insert("client_secret".to_string(), self.client_secret.clone());
		body.insert("redirect_uri".to_string(), redirect_uri);
		body.insert("grant_type".to_string(), "authorization_code".to_string());

		let response = self
			.client
			.post_form_urlencoded_capture::<String, GoogleOAuth2TokenResponse>(path, body)
			.map_err(|e| Error::RequestError(format!("{:?}", e)))?;

		Ok(response.id_token)
	}
}
