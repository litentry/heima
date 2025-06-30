// Copyright 2020-2025 Trust Computing GmbH.
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

use crate::inch_client::types::{SwapRequest, SwapResponse};
use async_trait::async_trait;
use log::error;
use reqwest::{Client, Error};
use std::sync::Arc;

pub struct InchClient {
	client: Client,
	access_token: Arc<str>,
	base_url: Arc<str>,
}

impl InchClient {
	pub fn new(access_token: impl Into<String>, base_url: impl Into<String>) -> Self {
		InchClient {
			client: Client::new(),
			access_token: Arc::from(access_token.into()),
			base_url: Arc::from(base_url.into()),
		}
	}
}

#[async_trait]
pub trait InchSwap: Send + Sync {
	async fn swap(&self, chain_id: u64, swap_request: SwapRequest) -> Result<SwapResponse, Error>;
}

#[async_trait]
impl InchSwap for InchClient {
	async fn swap(&self, chain_id: u64, swap_request: SwapRequest) -> Result<SwapResponse, Error> {
		let query_params = swap_request.convert_to_query_params();

		let path = format!("{}/swap/v6.0/{}/swap", self.base_url, chain_id);

		let response = self
			.client
			.get(&path)
			.query(&query_params)
			.header("Content-Length", "0")
			.header("accept", "application/json")
			.header("content-type", "application/json")
			.bearer_auth(self.access_token.clone())
			.send()
			.await
			.map_err(|e| {
				error!("Failed to send 1inch swap request: {:?}", e);
				e
			})?;

		let status = response.status();
		let response = response.error_for_status().map_err(|e| {
			error!("swap request failed with status: {}, error: {:?}", status, e);
			e
		})?;
		response.json().await.map_err(|e| {
			error!("Failed to parse 1inch swap response: {:?}", e);
			e
		})
	}
}
