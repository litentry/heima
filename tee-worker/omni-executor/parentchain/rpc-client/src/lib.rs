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
use parity_scale_codec::Encode;
use primitives::{BlockEvent, EventId};
use scale_encode::EncodeAsType;
use std::marker::PhantomData;
use std::ops::Deref;
use std::thread;
use std::time::Duration;
use std::vec::Vec;
use subxt::backend::legacy::LegacyRpcMethods;
use subxt::backend::BlockRef;
use subxt::config::signed_extensions;
use subxt::config::Header;
use subxt::events::EventsClient;
use subxt::storage::StorageClient;
use subxt::tx::TxClient;
use subxt::{Config, OnlineClient};
pub use subxt_core::utils::AccountId32;

// We don't need to construct this at runtime,
// so an empty enum is appropriate:
#[derive(EncodeAsType)]
pub enum CustomConfig {}

//todo: adjust if needed
impl Config for CustomConfig {
	type Hash = subxt::utils::H256;
	type AccountId = subxt::utils::AccountId32;
	type Address = subxt::utils::MultiAddress<Self::AccountId, u32>;
	type Signature = subxt::utils::MultiSignature;
	type Hasher = subxt::config::substrate::BlakeTwo256;
	type Header = subxt::config::substrate::SubstrateHeader<u32, Self::Hasher>;
	type ExtrinsicParams = signed_extensions::AnyOf<
		Self,
		(
			// Load in the existing signed extensions we're interested in
			// (if the extension isn't actually needed it'll just be ignored):
			signed_extensions::CheckSpecVersion,
			signed_extensions::CheckTxVersion,
			signed_extensions::CheckNonce,
			signed_extensions::CheckGenesis<Self>,
			signed_extensions::CheckMortality<Self>,
			signed_extensions::ChargeAssetTxPayment<Self>,
			signed_extensions::ChargeTransactionPayment,
			signed_extensions::CheckMetadataHash,
		),
	>;
	type AssetId = u32;
}

pub struct RuntimeVersion {
	pub spec_version: u32,
	pub transaction_version: u32,
}

/// For fetching data from Substrate RPC node
#[async_trait]
pub trait SubstrateRpcClient<AccountId, Header> {
	async fn get_last_finalized_header(&self) -> Result<Option<Header>, ()>;
	async fn get_last_finalized_block_num(&self) -> Result<u32, ()>;
	async fn get_block_events(&mut self, block_num: u64) -> Result<Vec<BlockEvent>, ()>;
	async fn get_raw_metadata(&mut self, block_num: Option<u64>) -> Result<Vec<u8>, ()>;
	async fn submit_tx(&mut self, raw_tx: &[u8]) -> Result<(), ()>;
	async fn runtime_version(&mut self) -> Result<RuntimeVersion, ()>;
	async fn get_genesis_hash(&mut self) -> Result<Vec<u8>, ()>;
	async fn get_account_nonce(&mut self, account_id: &AccountId) -> Result<u64, ()>;
	async fn get_storage_keys_paged(
		&mut self,
		key_prefix: Vec<u8>,
		count: u32,
		start_key: Option<Vec<u8>>,
	) -> Result<Vec<Vec<u8>>, ()>;
	async fn get_storage_proof_by_keys(&mut self, keys: Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>, ()>;
}

pub struct SubxtClient<ChainConfig: Config> {
	legacy: LegacyRpcMethods<ChainConfig>,
	events: EventsClient<ChainConfig, OnlineClient<ChainConfig>>,
	tx: TxClient<ChainConfig, OnlineClient<ChainConfig>>,
	storage: StorageClient<ChainConfig, OnlineClient<ChainConfig>>,
}

impl<ChainConfig: Config> SubxtClient<ChainConfig> {
	pub fn storage(&self) -> &StorageClient<ChainConfig, OnlineClient<ChainConfig>> {
		&self.storage
	}
}

#[async_trait]
impl<ChainConfig: Config<AccountId = AccountId32>>
	SubstrateRpcClient<ChainConfig::AccountId, ChainConfig::Header> for SubxtClient<ChainConfig>
{
	async fn get_last_finalized_header(&self) -> Result<Option<ChainConfig::Header>, ()> {
		let finalized_header = self.legacy.chain_get_finalized_head().await.map_err(|_| ())?;
		self.legacy.chain_get_header(Some(finalized_header)).await.map_err(|_| ())
	}
	async fn get_last_finalized_block_num(&self) -> Result<u32, ()> {
		match self.get_last_finalized_header().await {
			Ok(Some(header)) => {
				let block_num = header.number().into();
				// the parachain currently uses u32 for block numbers but subxt uses u64
				let block_num: u32 = block_num.try_into().map_err(|_| ())?;
				Ok(block_num)
			},
			_ => Err(()),
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

	async fn get_storage_keys_paged(
		&mut self,
		key_prefix: Vec<u8>,
		count: u32,
		start_key: Option<Vec<u8>>,
	) -> Result<Vec<Vec<u8>>, ()> {
		self.legacy
			.state_get_keys_paged(key_prefix.as_slice(), count, start_key.as_deref(), None)
			.await
			.map_err(|_| ())
	}

	async fn get_storage_proof_by_keys(&mut self, keys: Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>, ()> {
		let keys: Vec<&[u8]> = keys.iter().map(|k| k.as_slice()).collect();
		let storage_proof = self
			.legacy
			.state_get_read_proof(keys, None)
			.await
			.map(|read_proof| read_proof.proof.into_iter().map(|bytes| bytes.to_vec()).collect())
			.map_err(|_| ())?;

		Ok(storage_proof)
	}
}

pub struct MockedRpcClient<ChainConfig: Config> {
	block_num: u32,
	_phantom: PhantomData<ChainConfig>,
}

#[async_trait]
impl<ChainConfig: Config<AccountId = String>>
	SubstrateRpcClient<ChainConfig::AccountId, ChainConfig::Header> for MockedRpcClient<ChainConfig>
{
	async fn get_last_finalized_header(&self) -> Result<Option<ChainConfig::Header>, ()> {
		Ok(None)
	}
	async fn get_last_finalized_block_num(&self) -> Result<u32, ()> {
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

	async fn get_account_nonce(&mut self, _account_id: &ChainConfig::AccountId) -> Result<u64, ()> {
		Ok(0)
	}

	async fn get_storage_keys_paged(
		&mut self,
		_key_prefix: Vec<u8>,
		_count: u32,
		_start_key: Option<Vec<u8>>,
	) -> Result<Vec<Vec<u8>>, ()> {
		Ok(vec![])
	}

	async fn get_storage_proof_by_keys(&mut self, _keys: Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>, ()> {
		Ok(vec![])
	}
}

#[async_trait]
pub trait SubstrateRpcClientFactory<
	AccountId,
	Header,
	RpcClient: SubstrateRpcClient<AccountId, Header>,
>
{
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
	SubstrateRpcClientFactory<ChainConfig::AccountId, ChainConfig::Header, SubxtClient<ChainConfig>>
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
		let storage = online_client.storage();

		Ok(SubxtClient { legacy, events, tx, storage })
	}
}
