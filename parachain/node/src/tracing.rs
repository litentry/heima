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

use crate::evm_tracing_types::{EthApi as EthApiCmd, EvmTracingConfig};

use fc_rpc_core::types::FilterPool;
use fc_storage::StorageOverride;
use fp_rpc::EthereumRuntimeRPCApi;
use sc_client_api::{
	Backend, BlockOf, BlockchainEvents, HeaderBackend, StateBackend, StorageProvider,
};
use sc_service::TaskManager;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderMetadata};
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT, Header as HeaderT};
use std::sync::Arc;
use substrate_prometheus_endpoint::Registry as PrometheusRegistry;
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct RpcRequesters {
	// Tracing support removed - using standard frontier without debug/trace APIs
}

#[allow(dead_code)]
pub struct SpawnTasksParams<'a, B: BlockT, C, BE> {
	pub task_manager: &'a TaskManager,
	pub client: Arc<C>,
	pub substrate_backend: Arc<BE>,
	pub frontier_backend: Arc<dyn fc_api::Backend<B> + Send + Sync>,
	pub filter_pool: Option<FilterPool>,
	pub storage_override: Arc<dyn StorageOverride<B>>,
}

pub fn spawn_tracing_tasks<B, C, BE>(
	rpc_config: &EvmTracingConfig,
	prometheus: Option<PrometheusRegistry>,
	params: SpawnTasksParams<B, C, BE>,
) -> RpcRequesters
where
	C: ProvideRuntimeApi<B> + BlockOf,
	C: StorageProvider<B, BE>,
	C: HeaderBackend<B> + HeaderMetadata<B, Error = BlockChainError> + 'static,
	C: BlockchainEvents<B>,
	C: Send + Sync + 'static,
	C::Api: EthereumRuntimeRPCApi<B>,
	C::Api: BlockBuilder<B>,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	B::Header: HeaderT<Number = u32>,
	BE: Backend<B> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
{
	// Tracing support removed - using standard frontier without debug/trace APIs
	RpcRequesters {}
}
