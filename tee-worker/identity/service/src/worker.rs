/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG

	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.

*/

///! Integritee worker. Inspiration for this design came from parity's substrate Client.
///
/// This should serve as a proof of concept for a potential refactoring design. Ultimately, everything
/// from the main.rs should be covered by the worker struct here - hidden and split across
/// multiple traits.
use crate::{config::Config, error::Error, initialized_service::TrackInitialization};
use async_trait::async_trait;
use codec::{Decode, Encode};
use futures::future::join_all;
use itc_rpc_client::direct_client::{DirectApi, DirectClient as DirectWorkerApi};
use itp_enclave_api::enclave_base::EnclaveBase;
use itp_node_api::{api_client::PalletTeebagApi, node_api_factory::CreateNodeApi};
use itp_types::ShardIdentifier;
use itp_utils::ToHexPrefixed;
use its_primitives::types::SignedBlock as SignedSidechainBlock;
use its_rpc_handler::constants::RPC_METHOD_NAME_IMPORT_BLOCKS;
use jsonrpsee::{
	types::{to_json_value, traits::Client, v2::params::JsonRpcParams, JsonValue},
	ws_client::WsClientBuilder,
};
use litentry_primitives::WorkerType;
use log::*;
use std::{
	collections::HashSet,
	sync::{Arc, RwLock},
};

pub type WorkerResult<T> = Result<T, Error>;
pub type Url = String;

#[derive(Clone, Hash, Eq, PartialEq, Encode, Decode, Debug)]
pub struct PeerUrls {
	pub trusted: Url,
	pub me: bool,
}

impl PeerUrls {
	pub fn new(trusted: Url, me: bool) -> Self {
		PeerUrls { trusted, me }
	}
}

pub struct Worker<Config, NodeApiFactory, Enclave, InitializationHandler> {
	_config: Config,
	// unused yet, but will be used when more methods are migrated to the worker
	_enclave_api: Arc<Enclave>,
	node_api_factory: Arc<NodeApiFactory>,
	initialization_handler: Arc<InitializationHandler>,
	peer_urls: RwLock<HashSet<PeerUrls>>,
}

impl<Config, NodeApiFactory, Enclave, InitializationHandler>
	Worker<Config, NodeApiFactory, Enclave, InitializationHandler>
{
	pub fn new(
		config: Config,
		enclave_api: Arc<Enclave>,
		node_api_factory: Arc<NodeApiFactory>,
		initialization_handler: Arc<InitializationHandler>,
		peer_urls: HashSet<PeerUrls>,
	) -> Self {
		Self {
			_config: config,
			_enclave_api: enclave_api,
			node_api_factory,
			initialization_handler,
			peer_urls: RwLock::new(peer_urls),
		}
	}
}

#[async_trait]
/// Broadcast Sidechain blocks to peers.
pub trait AsyncBlockBroadcaster {
	async fn broadcast_blocks(&self, blocks: Vec<SignedSidechainBlock>) -> WorkerResult<()>;
}

#[async_trait]
impl<NodeApiFactory, Enclave, InitializationHandler> AsyncBlockBroadcaster
	for Worker<Config, NodeApiFactory, Enclave, InitializationHandler>
where
	NodeApiFactory: CreateNodeApi + Send + Sync,
	Enclave: Send + Sync,
	InitializationHandler: TrackInitialization + Send + Sync,
{
	async fn broadcast_blocks(&self, blocks: Vec<SignedSidechainBlock>) -> WorkerResult<()> {
		if blocks.is_empty() {
			debug!("No blocks to broadcast, returning");
			return Ok(());
		}
		let nr_blocks = blocks.len();
		let encoded_blocks = blocks.to_hex();

		let peers = self
			.peer_urls
			.read()
			.map_err(|e| {
				Error::Custom(format!("Encountered poisoned lock for peers: {:?}", e).into())
			})
			.map(|l| l.clone())?;

		self.initialization_handler.sidechain_block_produced();

		let nr_peers = peers.len();

		let futures = peers.iter().map(|url| {
			let encoded_blocks_cloned = encoded_blocks.clone();
			let url = url.trusted.clone();
			tokio::task::spawn_blocking(move || {
				let client = DirectWorkerApi::new(url.to_string());
				if let Err(e) = client.import_sidechain_blocks(encoded_blocks_cloned) {
					error!(
						"Broadcast block request ({}) to {} failed: {:?}",
						RPC_METHOD_NAME_IMPORT_BLOCKS, url, e
					);
				}
			})
		});

		join_all(futures).await;

		info!("broadcast {} block(s) to {} peers", nr_blocks, nr_peers);
		Ok(())
	}
}

/// Looks for new peers and updates them.
pub trait UpdatePeers {
	fn search_peers(&self, shard: ShardIdentifier) -> WorkerResult<HashSet<PeerUrls>>;

	fn set_peers_urls(&self, peers: HashSet<PeerUrls>) -> WorkerResult<()>;

	fn update_peers(&self, shard: ShardIdentifier) -> WorkerResult<()> {
		let peers = self.search_peers(shard)?;
		self.set_peers_urls(peers)
	}
}

pub trait GetPeers {
	fn read_peers_urls(&self) -> WorkerResult<HashSet<PeerUrls>>;
}

impl<NodeApiFactory, Enclave, InitializationHandler> GetPeers
	for Worker<Config, NodeApiFactory, Enclave, InitializationHandler>
where
	NodeApiFactory: CreateNodeApi + Send + Sync,
	Enclave: EnclaveBase + itp_enclave_api::remote_attestation::TlsRemoteAttestation,
{
	fn read_peers_urls(&self) -> WorkerResult<HashSet<PeerUrls>> {
		if let Ok(peer_urls) = self.peer_urls.read() {
			Ok(peer_urls.clone())
		} else {
			Err(Error::Custom("Encountered poisoned lock for peers".into()))
		}
	}
}

impl<NodeApiFactory, Enclave, InitializationHandler> UpdatePeers
	for Worker<Config, NodeApiFactory, Enclave, InitializationHandler>
where
	NodeApiFactory: CreateNodeApi + Send + Sync,
	Enclave: EnclaveBase + itp_enclave_api::remote_attestation::TlsRemoteAttestation,
{
	fn search_peers(&self, shard: ShardIdentifier) -> WorkerResult<HashSet<PeerUrls>> {
		let worker_url_external = self._config.trusted_worker_url_external();
		let node_api = self
			.node_api_factory
			.create_api()
			.map_err(|e| Error::Custom(format!("Failed to create NodeApi: {:?}", e).into()))?;
		let enclaves = node_api.all_enclaves(WorkerType::Identity, None)?;
		let mut peer_urls = HashSet::<PeerUrls>::new();
		for enclave in enclaves {
			// FIXME: This is temporary only, as block broadcasting should be moved to trusted ws server.
			let enclave_url = String::from_utf8_lossy(enclave.url.as_slice()).to_string();
			trace!("found peer rpc url: {}", enclave_url);
			let is_me = enclave_url == worker_url_external;
			peer_urls.insert(PeerUrls::new(enclave_url, is_me));
		}
		debug!("found {} peers in shard state for {:?}", peer_urls.len(), shard);
		Ok(peer_urls)
	}

	fn set_peers_urls(&self, peers: HashSet<PeerUrls>) -> WorkerResult<()> {
		let peers_vec: Vec<PeerUrls> = peers.clone().into_iter().collect();
		info!("Setting peers urls: {:?}", peers_vec);

		let mut peer_urls = self.peer_urls.write().map_err(|e| {
			Error::Custom(format!("Encountered poisoned lock for peers urls: {:?}", e).into())
		})?;
		*peer_urls = peers;
		Ok(())
	}
}
#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		tests::{
			commons::local_worker_config,
			mock::{W1_URL, W2_URL},
			mocks::initialization_handler_mock::TrackInitializationMock,
		},
		worker::{AsyncBlockBroadcaster, Worker},
	};
	use frame_support::assert_ok;
	use itp_node_api::node_api_factory::NodeApiFactory;
	use its_primitives::types::block::SignedBlock as SignedSidechainBlock;
	use its_test::sidechain_block_builder::{SidechainBlockBuilder, SidechainBlockBuilderTrait};
	use jsonrpsee::{ws_server::WsServerBuilder, RpcModule};
	use log::debug;
	use sp_keyring::AccountKeyring;
	use std::{net::SocketAddr, sync::Arc};
	use tokio::net::ToSocketAddrs;

	fn init() {
		let _ = env_logger::builder().is_test(true).try_init();
	}

	async fn run_server(addr: impl ToSocketAddrs) -> anyhow::Result<SocketAddr> {
		let mut server = WsServerBuilder::default().build(addr).await?;
		let mut module = RpcModule::new(());

		module.register_method(RPC_METHOD_NAME_IMPORT_BLOCKS, |params, _| {
			debug!("{} params: {:?}", RPC_METHOD_NAME_IMPORT_BLOCKS, params);
			let _blocks: Vec<SignedSidechainBlock> = params.one()?;
			Ok("ok".as_bytes().to_vec())
		})?;

		server.register_module(module).unwrap();

		let socket_addr = server.local_addr()?;
		tokio::spawn(async move { server.start().await });
		Ok(socket_addr)
	}

	#[tokio::test]
	async fn broadcast_blocks_works() {
		init();
		run_server(W1_URL).await.unwrap();
		run_server(W2_URL).await.unwrap();
		let untrusted_worker_port = "4000".to_string();
		let mut peer_urls: HashSet<PeerUrls> = HashSet::new();

		peer_urls.insert(PeerUrls { trusted: format!("ws://{}", W1_URL), me: false });
		peer_urls.insert(PeerUrls { trusted: format!("ws://{}", W2_URL), me: false });

		let worker = Worker::new(
			local_worker_config(W1_URL.into(), untrusted_worker_port.clone(), "30".to_string()),
			Arc::new(()),
			Arc::new(NodeApiFactory::new(
				"ws://invalid.url".to_string(),
				AccountKeyring::Alice.pair(),
			)),
			Arc::new(TrackInitializationMock {}),
			peer_urls,
		);

		let resp = worker
			.broadcast_blocks(vec![SidechainBlockBuilder::default().build_signed()])
			.await;
		assert_ok!(resp);
	}
}
