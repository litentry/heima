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

use async_trait::async_trait;
use log::{error, info};
use parentchain_primitives::{BlockEvent, EventId};
use parity_scale_codec::Encode;
use std::marker::PhantomData;
use std::ops::Deref;
use std::thread;
use std::time::Duration;
use subxt::backend::legacy::LegacyRpcMethods;
use subxt::backend::BlockRef;
use subxt::config::Header;
use subxt::events::EventsClient;
use subxt::tx::TxClient;
use subxt::{Config, OnlineClient};
use subxt_core::utils::AccountId32;

pub struct RuntimeVersion {
	pub spec_version: u32,
	pub transaction_version: u32,
}

/// For fetching data from Substrate RPC node
#[async_trait]
pub trait SubstrateRpcClient<AccountId> {
	async fn get_last_finalized_block_num(&mut self) -> Result<u64, ()>;
	async fn get_block_events(&mut self, block_num: u64) -> Result<Vec<BlockEvent>, ()>;
	async fn get_raw_metadata(&mut self, block_num: Option<u64>) -> Result<Vec<u8>, ()>;
	async fn submit_tx(&mut self, raw_tx: &[u8]) -> Result<(), ()>;
	async fn runtime_version(&mut self) -> Result<RuntimeVersion, ()>;
	async fn get_genesis_hash(&mut self) -> Result<Vec<u8>, ()>;
	async fn get_account_nonce(&mut self, account_id: &AccountId) -> Result<u64, ()>;
}

pub struct SubxtClient<ChainConfig: Config> {
	legacy: LegacyRpcMethods<ChainConfig>,
	events: EventsClient<ChainConfig, OnlineClient<ChainConfig>>,
	tx: TxClient<ChainConfig, OnlineClient<ChainConfig>>,
}

impl<ChainConfig: Config> SubxtClient<ChainConfig> {}

#[async_trait]
impl<ChainConfig: Config<AccountId = AccountId32>> SubstrateRpcClient<ChainConfig::AccountId>
	for SubxtClient<ChainConfig>
{
	async fn get_last_finalized_block_num(&mut self) -> Result<u64, ()> {
		let finalized_header = self.legacy.chain_get_finalized_head().await.map_err(|_| ())?;
		match self.legacy.chain_get_header(Some(finalized_header)).await.map_err(|_| ())? {
			Some(header) => Ok(header.number().into()),
			None => Err(()),
		}
	}
	async fn get_block_events(&mut self, block_num: u64) -> Result<Vec<BlockEvent>, ()> {
		info!("Getting block {} events", block_num);
		match self.legacy.chain_get_block_hash(Some(block_num.into())).await.map_err(|e| {
			error!("Error getting block {} hash: {:?}", block_num, e);
		})? {
			Some(hash) => {
				let events = self.events.at(BlockRef::from_hash(hash)).await.map_err(|e| {
					error!("Error getting block {} events: {:?}", block_num, e);
				})?;
				Ok(events
					.iter()
					.enumerate()
					.map(|(i, event)| {
						let event = event.unwrap();
						BlockEvent::new(
							EventId::new(block_num, i as u64),
							event.pallet_name().to_string(),
							event.variant_name().to_string(),
							event.variant_index(),
							event.field_bytes().to_vec(),
						)
					})
					.collect())
			},
			None => Err(()),
		}
	}

	async fn get_raw_metadata(&mut self, block_num: Option<u64>) -> Result<Vec<u8>, ()> {
		let maybe_hash = self
			.legacy
			.chain_get_block_hash(block_num.map(|b| b.into()))
			.await
			.map_err(|_| ())?;
		Ok(self.legacy.state_get_metadata(maybe_hash).await.unwrap().deref().encode())
	}

	async fn submit_tx(&mut self, raw_tx: &[u8]) -> Result<(), ()> {
		self.legacy.author_submit_extrinsic(raw_tx).await.map(|_| ()).map_err(|e| {
			error!("Could not submit tx: {:?}", e);
		})
	}

	async fn runtime_version(&mut self) -> Result<RuntimeVersion, ()> {
		self.legacy
			.state_get_runtime_version(None)
			.await
			.map(|rv| RuntimeVersion {
				spec_version: rv.spec_version,
				transaction_version: rv.transaction_version,
			})
			.map_err(|_| ())
	}

	async fn get_genesis_hash(&mut self) -> Result<Vec<u8>, ()> {
		self.legacy.genesis_hash().await.map(|h| h.encode()).map_err(|_| ())
	}

	async fn get_account_nonce(&mut self, account_id: &ChainConfig::AccountId) -> Result<u64, ()> {
		self.tx.account_nonce(account_id).await.map_err(|_| ())
	}
}

pub struct MockedRpcClient {
	block_num: u64,
}

#[async_trait]
impl SubstrateRpcClient<String> for MockedRpcClient {
	async fn get_last_finalized_block_num(&mut self) -> Result<u64, ()> {
		Ok(self.block_num)
	}

	async fn get_block_events(&mut self, _block_num: u64) -> Result<Vec<BlockEvent>, ()> {
		Ok(vec![])
	}

	async fn get_raw_metadata(&mut self, _block_num: Option<u64>) -> Result<Vec<u8>, ()> {
		Ok(vec![])
	}

	async fn submit_tx(&mut self, _raw_tx: &[u8]) -> Result<(), ()> {
		Ok(())
	}

	async fn runtime_version(&mut self) -> Result<RuntimeVersion, ()> {
		Ok(RuntimeVersion { spec_version: 0, transaction_version: 0 })
	}

	async fn get_genesis_hash(&mut self) -> Result<Vec<u8>, ()> {
		Ok(vec![])
	}

	async fn get_account_nonce(&mut self, _account_id: &String) -> Result<u64, ()> {
		Ok(0)
	}
}

#[async_trait]
pub trait SubstrateRpcClientFactory<AccountId, RpcClient: SubstrateRpcClient<AccountId>> {
	async fn new_client(&self) -> Result<RpcClient, ()>;
}

pub struct SubxtClientFactory<ChainConfig: Config> {
	url: String,
	_phantom: PhantomData<ChainConfig>,
}

impl<ChainConfig: Config<AccountId = AccountId32>> SubxtClientFactory<ChainConfig> {
	pub fn new(url: &str) -> Self {
		Self { url: url.to_string(), _phantom: PhantomData }
	}
	pub async fn new_client_until_connected(&self) -> SubxtClient<ChainConfig> {
		let mut client: Option<SubxtClient<ChainConfig>> = None;
		loop {
			if client.is_some() {
				break;
			}
			match self.new_client().await {
				Ok(c) => {
					client = Some(c);
				},
				Err(e) => {
					error!("Error creating client: {:?}", e);
				},
			};
			thread::sleep(Duration::from_secs(1))
		}
		client.unwrap()
	}
}

#[async_trait]
impl<ChainConfig: Config<AccountId = AccountId32>>
	SubstrateRpcClientFactory<ChainConfig::AccountId, SubxtClient<ChainConfig>>
	for SubxtClientFactory<ChainConfig>
{
	async fn new_client(&self) -> Result<SubxtClient<ChainConfig>, ()> {
		let rpc_client = subxt::backend::rpc::RpcClient::from_insecure_url(self.url.clone())
			.await
			.map_err(|e| {
				log::error!("Could not create RpcClient: {:?}", e);
			})?;
		let legacy = LegacyRpcMethods::new(rpc_client);

		let online_client =
			OnlineClient::from_insecure_url(self.url.clone()).await.map_err(|e| {
				log::error!("Could not create OnlineClient: {:?}", e);
			})?;

		let events = online_client.events();
		let tx = online_client.tx();

		Ok(SubxtClient { legacy, events, tx })
	}
}
