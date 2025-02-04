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
mod xt_status;
pub use xt_status::{TransactionStatus, XtStatus};

mod type_converters;
pub use type_converters::*;

pub mod metadata;
pub use subxt_core::utils::AccountId32;

use async_trait::async_trait;
use executor_primitives::{AccountId, BlockEvent, BlockNumber, EventId, Hash};
use log::{error, info};
use parity_scale_codec::{Decode, Encode};
use scale_encode::EncodeAsType;
use std::marker::PhantomData;
use std::ops::Deref;
use std::vec::Vec;
use subxt::{
	backend::{legacy::LegacyRpcMethods, BlockRef},
	blocks::BlocksClient,
	config::{
		signed_extensions,
		substrate::{BlakeTwo256, SubstrateHeader},
		Hasher,
	},
	events::EventsClient,
	storage::StorageClient,
	tx::TxClient,
	Config, OnlineClient,
};
use tokio::time::{sleep, Duration};

pub type RpcClientHeader = SubstrateHeader<BlockNumber, BlakeTwo256>;

// We don't need to construct this at runtime,
// so an empty enum is appropriate:
#[derive(EncodeAsType, Clone)]
pub enum CustomConfig {}

//todo: adjust if needed
impl Config for CustomConfig {
	type Hash = subxt::utils::H256;
	type AccountId = subxt::utils::AccountId32;
	type Address = subxt::utils::MultiAddress<Self::AccountId, u32>;
	type Signature = subxt::utils::MultiSignature;
	type Hasher = BlakeTwo256;
	type Header = SubstrateHeader<BlockNumber, Self::Hasher>;
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
pub trait SubstrateRpcClient<Header> {
	async fn get_last_finalized_header(&self) -> Result<Header, ()>;
	async fn get_last_finalized_block_num(&self) -> Result<BlockNumber, ()>;
	async fn get_block_events(&mut self, block_num: u64) -> Result<Vec<BlockEvent>, ()>;
	async fn get_raw_metadata(&mut self, block_num: Option<u64>) -> Result<Vec<u8>, ()>;
	async fn submit_tx(&mut self, raw_tx: &[u8]) -> Result<(), ()>;
	async fn submit_and_watch_tx_until(
		&mut self,
		extrinsic: &[u8],
		until_status: XtStatus,
	) -> Result<ExtrinsicReport<Hash>, ()>;
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
	blocks: BlocksClient<ChainConfig, OnlineClient<ChainConfig>>,
}

#[derive(Decode, Encode)]
pub struct ExtrinsicReport<Hash> {
	pub extrinsic_hash: Hash,
	pub block_hash: Option<Hash>,
	pub status: TransactionStatus<Hash>,
}

impl<Hash> ExtrinsicReport<Hash> {
	pub fn new(
		extrinsic_hash: Hash,
		block_hash: Option<Hash>,
		status: TransactionStatus<Hash>,
	) -> Self {
		Self { extrinsic_hash, block_hash, status }
	}
}

impl<ChainConfig: Config> SubxtClient<ChainConfig> {
	pub fn storage(&self) -> &StorageClient<ChainConfig, OnlineClient<ChainConfig>> {
		&self.storage
	}
}

#[async_trait]
impl<ChainConfig: Config<AccountId = AccountId32, Header = RpcClientHeader>>
	SubstrateRpcClient<ChainConfig::Header> for SubxtClient<ChainConfig>
{
	async fn get_last_finalized_header(&self) -> Result<ChainConfig::Header, ()> {
		let latest_block = self.blocks.at_latest().await.map_err(|e| {
			error!("Error getting latest block: {:?}", e);
		})?;
		Ok(latest_block.header().clone())
	}
	async fn get_last_finalized_block_num(&self) -> Result<BlockNumber, ()> {
		let latest_block = self.blocks.at_latest().await.map_err(|e| {
			error!("Error getting latest block: {:?}", e);
		})?;
		Ok(latest_block.number())
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

	async fn submit_and_watch_tx_until(
		&mut self,
		extrinsic: &[u8],
		until_status: XtStatus,
	) -> Result<ExtrinsicReport<Hash>, ()> {
		let tx_hash_bytes = ChainConfig::Hasher::hash(extrinsic).encode();
		let result = self.legacy.author_submit_and_watch_extrinsic(extrinsic).await;
		match result {
			Ok(mut subscription) => {
				while let Some(Ok(tx_status)) = subscription.next().await {
					let transaction_status: TransactionStatus<Hash> = tx_status.into();
					match transaction_status.is_expected() {
						Ok(_) => {
							if transaction_status.reached_status(until_status) {
								let block_hash = transaction_status.get_maybe_block_hash();
								return Ok(ExtrinsicReport::new(
									Hash::decode(&mut tx_hash_bytes.as_slice()).unwrap(),
									block_hash.cloned(),
									transaction_status,
								));
							}
						},
						Err(e) => {
							error!("Unexpected transaction status: {:?}", e);
							return Err(());
						},
					}
				}
			},
			Err(e) => {
				error!("Could not submit tx: {:?}", e);
			},
		};

		Err(())
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

	async fn get_account_nonce(&mut self, account_id: &AccountId) -> Result<u64, ()> {
		self.tx.account_nonce(&account_id.to_subxt_type()).await.map_err(|_| ())
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
impl<ChainConfig: Config<AccountId = String, Header = RpcClientHeader>>
	SubstrateRpcClient<ChainConfig::Header> for MockedRpcClient<ChainConfig>
{
	async fn get_last_finalized_header(&self) -> Result<ChainConfig::Header, ()> {
		let numeric_block_number_json = r#"
            {
                "digest": {
                    "logs": []
                },
                "extrinsicsRoot": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "number": 4,
                "parentHash": "0xcb2690b2c85ceab55be03fc7f7f5f3857e7efeb7a020600ebd4331e10be2f7a5",
                "stateRoot": "0x0000000000000000000000000000000000000000000000000000000000000000"
            }
        "#;

		let header: SubstrateHeader<u32, BlakeTwo256> =
			serde_json::from_str(numeric_block_number_json).expect("valid block header");

		Ok(header)
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

	async fn get_account_nonce(&mut self, _account_id: &AccountId) -> Result<u64, ()> {
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

	async fn submit_and_watch_tx_until(
		&mut self,
		_extrinsic: &[u8],
		_until_status: XtStatus,
	) -> Result<ExtrinsicReport<Hash>, ()> {
		todo!()
	}
}

#[async_trait]
pub trait SubstrateRpcClientFactory<Header, RpcClient: SubstrateRpcClient<Header>> {
	async fn new_client(&self) -> Result<RpcClient, ()>;
}

#[derive(Clone)]
pub struct SubxtClientFactory<ChainConfig: Config> {
	url: String,
	_phantom: PhantomData<ChainConfig>,
}

impl<ChainConfig: Config<AccountId = AccountId32, Header = RpcClientHeader>>
	SubxtClientFactory<ChainConfig>
{
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
			sleep(Duration::from_secs(1)).await;
		}
		client.unwrap()
	}
}

#[async_trait]
impl<ChainConfig: Config<AccountId = AccountId32, Header = RpcClientHeader>>
	SubstrateRpcClientFactory<ChainConfig::Header, SubxtClient<ChainConfig>>
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
		let blocks = online_client.blocks();

		Ok(SubxtClient { legacy, events, tx, storage, blocks })
	}
}
