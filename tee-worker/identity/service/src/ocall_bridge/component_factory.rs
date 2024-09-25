/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG
	Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.

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

use crate::{
	globals::tokio_handle::GetTokioHandle,
	ocall_bridge::{
		bridge_api::{
			GetOCallBridgeComponents, IpfsBridge, MetricsBridge, RemoteAttestationBridge,
			SidechainBridge, WorkerOnChainBridge,
		},
		ipfs_ocall::IpfsOCall,
		metrics_ocall::MetricsOCall,
		remote_attestation_ocall::RemoteAttestationOCall,
		sidechain_ocall::SidechainOCall,
		worker_on_chain_ocall::WorkerOnChainOCall,
	},
	prometheus_metrics::ReceiveEnclaveMetrics,
	sync_block_broadcaster::BroadcastBlocks,
	worker_peers_updater::PeersRegistry,
};
use itp_enclave_api::{enclave_base::EnclaveBase, remote_attestation::RemoteAttestationCallBacks};
use itp_node_api::node_api_factory::CreateNodeApi;
use its_peer_fetch::FetchBlocksFromPeer;
use its_primitives::types::block::SignedBlock as SignedSidechainBlock;
use its_storage::BlockStorage;
use std::sync::Arc;

/// Concrete implementation, should be moved out of the OCall Bridge, into the worker
/// since the OCall bridge itself should not know any concrete types to ensure
/// our dependency graph is worker -> ocall bridge
pub struct OCallBridgeComponentFactory<
	NodeApi,
	Broadcaster,
	EnclaveApi,
	Storage,
	WorkerPeersRegistry,
	PeerBlockFetcher,
	TokioHandle,
	MetricsReceiver,
> {
	integritee_rpc_api_factory: Arc<NodeApi>,
	target_a_parentchain_rpc_api_factory: Option<Arc<NodeApi>>,
	target_b_parentchain_rpc_api_factory: Option<Arc<NodeApi>>,
	block_broadcaster: Arc<Broadcaster>,
	enclave_api: Arc<EnclaveApi>,
	block_storage: Arc<Storage>,
	peers_registry: Arc<WorkerPeersRegistry>,
	peer_block_fetcher: Arc<PeerBlockFetcher>,
	tokio_handle: Arc<TokioHandle>,
	metrics_receiver: Arc<MetricsReceiver>,
}

impl<
		NodeApi,
		Broadcaster,
		EnclaveApi,
		Storage,
		WorkerPeersRegistry,
		PeerBlockFetcher,
		TokioHandle,
		MetricsReceiver,
	>
	OCallBridgeComponentFactory<
		NodeApi,
		Broadcaster,
		EnclaveApi,
		Storage,
		WorkerPeersRegistry,
		PeerBlockFetcher,
		TokioHandle,
		MetricsReceiver,
	>
{
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		integritee_rpc_api_factory: Arc<NodeApi>,
		target_a_parentchain_rpc_api_factory: Option<Arc<NodeApi>>,
		target_b_parentchain_rpc_api_factory: Option<Arc<NodeApi>>,
		block_broadcaster: Arc<Broadcaster>,
		enclave_api: Arc<EnclaveApi>,
		block_storage: Arc<Storage>,
		peers_registry: Arc<WorkerPeersRegistry>,
		peer_block_fetcher: Arc<PeerBlockFetcher>,
		tokio_handle: Arc<TokioHandle>,
		metrics_receiver: Arc<MetricsReceiver>,
	) -> Self {
		OCallBridgeComponentFactory {
			integritee_rpc_api_factory,
			target_a_parentchain_rpc_api_factory,
			target_b_parentchain_rpc_api_factory,
			block_broadcaster,
			enclave_api,
			block_storage,
			peers_registry,
			peer_block_fetcher,
			tokio_handle,
			metrics_receiver,
		}
	}
}

impl<
		NodeApi,
		Broadcaster,
		EnclaveApi,
		Storage,
		WorkerPeersRegistry,
		PeerBlockFetcher,
		TokioHandle,
		MetricsReceiver,
	> GetOCallBridgeComponents
	for OCallBridgeComponentFactory<
		NodeApi,
		Broadcaster,
		EnclaveApi,
		Storage,
		WorkerPeersRegistry,
		PeerBlockFetcher,
		TokioHandle,
		MetricsReceiver,
	> where
	NodeApi: CreateNodeApi + 'static,
	Broadcaster: BroadcastBlocks + 'static,
	EnclaveApi: EnclaveBase + RemoteAttestationCallBacks + 'static,
	Storage: BlockStorage<SignedSidechainBlock> + 'static,
	WorkerPeersRegistry: PeersRegistry + 'static,
	PeerBlockFetcher: FetchBlocksFromPeer<SignedBlockType = SignedSidechainBlock> + 'static,
	TokioHandle: GetTokioHandle + 'static,
	MetricsReceiver: ReceiveEnclaveMetrics + 'static,
{
	fn get_ra_api(&self) -> Arc<dyn RemoteAttestationBridge> {
		Arc::new(RemoteAttestationOCall::new(self.enclave_api.clone()))
	}

	fn get_sidechain_api(&self) -> Arc<dyn SidechainBridge> {
		Arc::new(SidechainOCall::new(
			self.block_broadcaster.clone(),
			self.block_storage.clone(),
			self.peers_registry.clone(),
			self.peer_block_fetcher.clone(),
			self.tokio_handle.clone(),
		))
	}

	fn get_oc_api(&self) -> Arc<dyn WorkerOnChainBridge> {
		Arc::new(WorkerOnChainOCall::new(
			self.enclave_api.clone(),
			self.integritee_rpc_api_factory.clone(),
			self.target_a_parentchain_rpc_api_factory.clone(),
			self.target_b_parentchain_rpc_api_factory.clone(),
		))
	}

	fn get_ipfs_api(&self) -> Arc<dyn IpfsBridge> {
		Arc::new(IpfsOCall {})
	}

	fn get_metrics_api(&self) -> Arc<dyn MetricsBridge> {
		Arc::new(MetricsOCall::new(self.metrics_receiver.clone()))
	}
}
