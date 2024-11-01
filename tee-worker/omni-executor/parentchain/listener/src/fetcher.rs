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

use crate::primitives::{BlockEvent, EventId};
use crate::rpc_client::SubstrateRpcClient;
use crate::rpc_client::SubstrateRpcClientFactory;
use async_trait::async_trait;
use executor_core::fetcher::{EventsFetcher, LastFinalizedBlockNumFetcher};
use log::error;
use std::sync::Arc;

/// Used for fetching data from parentchain
pub struct Fetcher<
	RpcClient: SubstrateRpcClient,
	RpcClientFactory: SubstrateRpcClientFactory<RpcClient>,
> {
	client_factory: Arc<RpcClientFactory>,
	client: Option<RpcClient>,
}

impl<RpcClient: SubstrateRpcClient, RpcClientFactory: SubstrateRpcClientFactory<RpcClient>>
	Fetcher<RpcClient, RpcClientFactory>
{
	pub fn new(client_factory: Arc<RpcClientFactory>) -> Self {
		Self { client: None, client_factory }
	}

	async fn connect_if_needed(&mut self) {
		if self.client.is_none() {
			match self.client_factory.new_client().await {
				Ok(client) => self.client = Some(client),
				Err(e) => error!("Could not create client: {:?}", e),
			}
		}
	}
}

#[async_trait]
impl<
		RpcClient: SubstrateRpcClient + Sync + Send,
		RpcClientFactory: SubstrateRpcClientFactory<RpcClient> + Sync + Send,
	> LastFinalizedBlockNumFetcher for Fetcher<RpcClient, RpcClientFactory>
{
	async fn get_last_finalized_block_num(&mut self) -> Result<Option<u64>, ()> {
		self.connect_if_needed().await;

		if let Some(ref mut client) = self.client {
			let block_num = client.get_last_finalized_block_num().await?;
			Ok(Some(block_num))
		} else {
			Err(())
		}
	}
}

#[async_trait]
impl<
		RpcClient: SubstrateRpcClient + Sync + Send,
		RpcClientFactory: SubstrateRpcClientFactory<RpcClient> + Sync + Send,
	> EventsFetcher<EventId, BlockEvent> for Fetcher<RpcClient, RpcClientFactory>
{
	async fn get_block_events(&mut self, block_num: u64) -> Result<Vec<BlockEvent>, ()> {
		self.connect_if_needed().await;

		if let Some(ref mut client) = self.client {
			client.get_block_events(block_num).await
		} else {
			Err(())
		}
	}
}
