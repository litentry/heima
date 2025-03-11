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

// For maintaining a node code without implementing Frontier EVM.
// This File should be safe to delete once All parachain matrix are EVM impl.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use crate::{
	evm_tracing_types::{EthApi as EthApiCmd, EvmTracingConfig},
	fake_runtime_api::RuntimeApi as FakeRuntimeApi,
	standalone_block_import::StandaloneBlockImport,
	tracing::{self, RpcRequesters},
};
pub use core_primitives::{AuraId, Block, Hash};
use cumulus_client_cli::CollatorOptions;
use cumulus_client_collator::service::CollatorService;
use cumulus_client_consensus_aura::collators::lookahead::{self as aura, Params as AuraParams};
#[allow(deprecated)]
use cumulus_client_consensus_aura::SlotProportion;
use cumulus_client_consensus_common::ParachainBlockImport as TParachainBlockImport;
use cumulus_client_consensus_proposer::Proposer;
use cumulus_client_parachain_inherent::{MockValidationDataInherentDataProvider, MockXcmConfig};
use cumulus_client_service::{
	build_network, build_relay_chain_interface, prepare_node_config, start_relay_chain_tasks,
	BuildNetworkParams, CollatorSybilResistance, DARecoveryProfile, StartRelayChainTasksParams,
};
use cumulus_primitives_core::{
	relay_chain::{CollatorPair, ValidationCode},
	ParaId,
};
use cumulus_relay_chain_interface::{OverseerHandle, RelayChainInterface};
use fc_consensus::FrontierBlockImport;
use fc_rpc::EthBlockDataCacheTask;
use fc_rpc_core::types::{FeeHistoryCache, FilterPool};
use fc_storage::{StorageOverride, StorageOverrideHandler};
use futures::StreamExt;
use jsonrpsee::RpcModule;
use parity_scale_codec::Encode;
use sc_client_api::BlockchainEvents;
use sc_consensus::{ImportQueue, LongestChain};
use sc_consensus_aura::StartAuraParams;
use sc_executor::{HeapAllocStrategy, WasmExecutor, DEFAULT_HEAP_ALLOC_STRATEGY};
use sc_network::{config::FullNetworkConfiguration, service::traits::NetworkBackend, NetworkBlock};
use sc_network_sync::SyncingService;
use sc_service::{Configuration, PartialComponents, TFullBackend, TFullClient, TaskManager};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use sp_keystore::KeystorePtr;
use sp_runtime::app_crypto::AppCrypto;
use sp_runtime::traits::Header;
use sp_std::{collections::btree_map::BTreeMap, sync::Arc, time::Duration};
use substrate_prometheus_endpoint::Registry;

#[cfg(not(feature = "runtime-benchmarks"))]
pub type HostFunctions = (
	cumulus_client_service::ParachainHostFunctions,
	moonbeam_primitives_ext::moonbeam_ext::HostFunctions,
);

#[cfg(feature = "runtime-benchmarks")]
pub type HostFunctions = (
	cumulus_client_service::ParachainHostFunctions,
	frame_benchmarking::benchmarking::HostFunctions,
	moonbeam_primitives_ext::moonbeam_ext::HostFunctions,
);

type ParachainClient = TFullClient<Block, FakeRuntimeApi, WasmExecutor<HostFunctions>>;

type ParachainBackend = TFullBackend<Block>;

type ParachainBlockImport = TParachainBlockImport<
	Block,
	FrontierBlockImport<Block, Arc<ParachainClient>, ParachainClient>,
	ParachainBackend,
>;

type MaybeSelectChain = Option<LongestChain<ParachainBackend, Block>>;

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
pub fn new_partial<BIQ>(
	config: &Configuration,
	build_import_queue: BIQ,
	is_standalone: bool,
	delayed_best_block: bool,
) -> Result<
	PartialComponents<
		ParachainClient,
		ParachainBackend,
		MaybeSelectChain,
		sc_consensus::DefaultImportQueue<Block>,
		sc_transaction_pool::FullPool<Block, ParachainClient>,
		(
			ParachainBlockImport,
			Option<Telemetry>,
			Option<TelemetryWorkerHandle>,
			Arc<fc_db::kv::Backend<Block, ParachainClient>>,
		),
	>,
	sc_service::Error,
>
where
	BIQ: FnOnce(
		Arc<ParachainClient>,
		ParachainBlockImport,
		&Configuration,
		Option<TelemetryHandle>,
		&TaskManager,
		bool,
	) -> Result<sc_consensus::DefaultImportQueue<Block>, sc_service::Error>,
{
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	let heap_pages = config.default_heap_pages.map_or(DEFAULT_HEAP_ALLOC_STRATEGY, |h: u64| {
		HeapAllocStrategy::Static { extra_pages: h as _ }
	});

	let executor = sc_executor::WasmExecutor::<HostFunctions>::builder()
		.with_execution_method(config.wasm_method)
		.with_max_runtime_instances(config.max_runtime_instances)
		.with_runtime_cache_size(config.runtime_cache_size)
		.with_onchain_heap_alloc_strategy(heap_pages)
		.with_offchain_heap_alloc_strategy(heap_pages)
		.build();

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts_record_import::<Block, _, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
			true,
		)?;
	let client = Arc::new(client);

	let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);

	let select_chain = if is_standalone { Some(LongestChain::new(backend.clone())) } else { None };
	let frontier_backend = crate::rpc::open_frontier_backend(client.clone(), config)?;
	let frontier_block_import = FrontierBlockImport::new(client.clone(), client.clone());

	// Note: `new_with_delayed_best_block` will cause less `Retracted`/`Invalid` tx,
	//       especially for rococo-local where the epoch duration is 1m. However, it
	//       also means the imported block will not be notified as best blocks, instead,
	//       it waits for the best block from relay chain
	// see https://github.com/paritytech/polkadot-sdk/issues/1202
	//     https://github.com/paritytech/polkadot-sdk/pull/2001
	//     https://github.com/moonbeam-foundation/moonbeam/issues/3040
	//
	// my suggestion:
	//     use `new_with_delayed_best_block` in CI, use `new` in prod
	//
	// TODO: re-investigate this after async backing is supported
	let block_import = if delayed_best_block {
		ParachainBlockImport::new_with_delayed_best_block(frontier_block_import, backend.clone())
	} else {
		ParachainBlockImport::new(frontier_block_import, backend.clone())
	};

	let import_queue = build_import_queue(
		client.clone(),
		block_import.clone(),
		config,
		telemetry.as_ref().map(|telemetry| telemetry.handle()),
		&task_manager,
		is_standalone,
	)?;

	Ok(PartialComponents {
		backend,
		client,
		import_queue,
		keystore_container,
		task_manager,
		transaction_pool,
		select_chain,
		other: (block_import, telemetry, telemetry_worker_handle, frontier_backend),
	})
}

/// To add additional config to start_xyz_node functions
#[derive(Clone)]
pub struct AdditionalConfig {
	/// EVM tracing configuration
	pub evm_tracing_config: EvmTracingConfig,

	/// Whether EVM RPC be enabled
	pub enable_evm_rpc: bool,
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[sc_tracing::logging::prefix_logs_with("Parachain")]
async fn start_node_impl<RB, BIQ, SC, Net>(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	sybil_resistance_level: CollatorSybilResistance,
	para_id: ParaId,
	_rpc_ext_builder: RB,
	build_import_queue: BIQ,
	start_consensus: SC,
	hwbench: Option<sc_sysinfo::HwBench>,
	additional_config: AdditionalConfig,
	delayed_best_block: bool,
) -> sc_service::error::Result<(TaskManager, Arc<ParachainClient>)>
where
	RB: Fn(
			sc_rpc::DenyUnsafe,
			Arc<ParachainClient>,
			Arc<ParachainBackend>,
			Arc<sc_transaction_pool::FullPool<Block, ParachainClient>>,
		) -> Result<jsonrpsee::RpcModule<()>, sc_service::Error>
		+ 'static,
	BIQ: FnOnce(
		Arc<ParachainClient>,
		ParachainBlockImport,
		&Configuration,
		Option<TelemetryHandle>,
		&TaskManager,
		bool,
	) -> Result<sc_consensus::DefaultImportQueue<Block>, sc_service::Error>,
	SC: FnOnce(
		Arc<ParachainClient>,
		ParachainBlockImport,
		Option<&Registry>,
		Option<TelemetryHandle>,
		&TaskManager,
		Arc<dyn RelayChainInterface>,
		Arc<sc_transaction_pool::FullPool<Block, ParachainClient>>,
		KeystorePtr,
		Duration,
		ParaId,
		CollatorPair,
		OverseerHandle,
		Arc<dyn Fn(Hash, Option<Vec<u8>>) + Send + Sync>,
		Arc<ParachainBackend>,
	) -> Result<(), sc_service::Error>,
	Net: NetworkBackend<Block, Hash>,
{
	let parachain_config = prepare_node_config(parachain_config);

	let params =
		new_partial::<BIQ>(&parachain_config, build_import_queue, false, delayed_best_block)?;
	let (block_import, mut telemetry, telemetry_worker_handle, frontier_backend) = params.other;

	let client = params.client.clone();
	let backend = params.backend.clone();

	let mut task_manager = params.task_manager;
	let (relay_chain_interface, collator_key) = build_relay_chain_interface(
		polkadot_config,
		&parachain_config,
		telemetry_worker_handle,
		&mut task_manager,
		collator_options.clone(),
		hwbench.clone(),
	)
	.await
	.map_err(|e| sc_service::Error::Application(Box::new(e) as Box<_>))?;

	let validator = parachain_config.role.is_authority();
	let prometheus_registry = parachain_config.prometheus_registry().cloned();
	let transaction_pool = params.transaction_pool.clone();
	let import_queue_service = params.import_queue.service();
	let net_config = FullNetworkConfiguration::<_, _, Net>::new(&parachain_config.network);

	let (network, system_rpc_tx, tx_handler_controller, start_network, sync_service) =
		build_network(BuildNetworkParams {
			parachain_config: &parachain_config,
			net_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			para_id,
			spawn_handle: task_manager.spawn_handle(),
			relay_chain_interface: relay_chain_interface.clone(),
			import_queue: params.import_queue,
			sybil_resistance_level,
		})
		.await?;

	// Sinks for pubsub notifications.
	// Everytime a new subscription is created, a new mpsc channel is added to the sink pool.
	// The MappingSyncWorker sends through the channel on block import and the subscription
	// emits a notification to the subscriber on receiving a message through this channel. This
	// way we avoid race conditions when using native substrate block import notification
	// stream.
	let pubsub_notification_sinks: fc_mapping_sync::EthereumBlockNotificationSinks<
		fc_mapping_sync::EthereumBlockNotification<Block>,
	> = Default::default();
	let pubsub_notification_sinks = Arc::new(pubsub_notification_sinks);

	let (
		filter_pool,
		fee_history_limit,
		fee_history_cache,
		block_data_cache,
		storage_override,
		tracing_requesters,
		ethapi_cmd,
	) = start_node_evm_impl(
		client.clone(),
		backend.clone(),
		frontier_backend.clone(),
		&mut task_manager,
		&parachain_config,
		additional_config.evm_tracing_config.clone(),
		sync_service.clone(),
		pubsub_notification_sinks.clone(),
	);

	let rpc_builder = {
		let client = client.clone();
		let transaction_pool = transaction_pool.clone();

		let network = network.clone();
		let rpc_config = crate::rpc::EvmTracingConfig {
			tracing_requesters,
			trace_filter_max_count: additional_config.evm_tracing_config.ethapi_trace_max_count,
			enable_txpool: ethapi_cmd.contains(&EthApiCmd::TxPool),
		};
		let sync = sync_service.clone();
		let pubsub_notification_sinks = pubsub_notification_sinks.clone();

		let result = move |deny_unsafe, subscription| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: transaction_pool.clone(),
				graph: transaction_pool.pool().clone(),
				network: network.clone(),
				sync: sync.clone(),
				is_authority: validator,
				deny_unsafe,
				frontier_backend: frontier_backend.clone(),
				filter_pool: filter_pool.clone(),
				fee_history_limit,
				fee_history_cache: fee_history_cache.clone(),
				block_data_cache: block_data_cache.clone(),
				storage_override: storage_override.clone(),
				enable_evm_rpc: additional_config.enable_evm_rpc,
			};

			let pending_consensus_data_provider =
				Box::new(fc_rpc::pending::AuraConsensusDataProvider::new(client.clone()));

			crate::rpc::create_full(
				deps,
				subscription,
				pubsub_notification_sinks.clone(),
				pending_consensus_data_provider,
				rpc_config.clone(),
			)
			.map_err(Into::into)
		};
		Box::new(result)
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		rpc_builder,
		client: client.clone(),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		config: parachain_config,
		keystore: params.keystore_container.keystore(),
		backend: backend.clone(),
		network: network.clone(),
		sync_service: sync_service.clone(),
		system_rpc_tx,
		tx_handler_controller,
		telemetry: telemetry.as_mut(),
	})?;

	if let Some(hwbench) = hwbench {
		sc_sysinfo::print_hwbench(&hwbench);
		if validator {
			warn_if_slow_hardware(&hwbench);
		}

		if let Some(ref mut telemetry) = telemetry {
			let telemetry_handle = telemetry.handle();
			task_manager.spawn_handle().spawn(
				"telemetry_hwbench",
				None,
				sc_sysinfo::initialize_hwbench_telemetry(telemetry_handle, hwbench),
			);
		}
	}

	let announce_block = {
		let sync_service = sync_service.clone();
		Arc::new(move |hash, data| sync_service.announce_block(hash, data))
	};

	let relay_chain_slot_duration = Duration::from_secs(6);

	let overseer_handle = relay_chain_interface
		.overseer_handle()
		.map_err(|e| sc_service::Error::Application(Box::new(e)))?;

	start_relay_chain_tasks(StartRelayChainTasksParams {
		client: client.clone(),
		announce_block: announce_block.clone(),
		para_id,
		relay_chain_interface: relay_chain_interface.clone(),
		task_manager: &mut task_manager,
		da_recovery_profile: if validator {
			DARecoveryProfile::Collator
		} else {
			DARecoveryProfile::FullNode
		},
		import_queue: import_queue_service,
		relay_chain_slot_duration,
		recovery_handle: Box::new(overseer_handle.clone()),
		sync_service: sync_service.clone(),
	})?;

	if validator {
		start_consensus(
			client.clone(),
			block_import,
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|t| t.handle()),
			&task_manager,
			relay_chain_interface.clone(),
			transaction_pool,
			params.keystore_container.keystore(),
			relay_chain_slot_duration,
			para_id,
			collator_key.expect("Command line arguments do not allow this. qed"),
			overseer_handle,
			announce_block,
			backend.clone(),
		)?;
	}

	start_network.start_network();

	Ok((task_manager, client))
}

/// Start a litentry/rococo node.
pub async fn start_node(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	para_id: ParaId,
	hwbench: Option<sc_sysinfo::HwBench>,
	additional_config: AdditionalConfig,
	delayed_best_block: bool,
) -> sc_service::error::Result<(TaskManager, Arc<ParachainClient>)> {
	start_node_impl::<_, _, _, sc_network::NetworkWorker<_, _>>(
		parachain_config,
		polkadot_config,
		collator_options,
		CollatorSybilResistance::Resistant, // Aura
		para_id,
		|_, _, _, _| Ok(RpcModule::new(())),
		build_import_queue,
		start_lookahead_aura_consensus,
		hwbench,
		additional_config.clone(),
		delayed_best_block,
	)
	.await
}

/// Build the import queue for the litentry/rococo runtime.
pub fn build_import_queue(
	client: Arc<ParachainClient>,
	block_import: ParachainBlockImport,
	config: &Configuration,
	telemetry: Option<TelemetryHandle>,
	task_manager: &TaskManager,
	is_standalone: bool,
) -> Result<sc_consensus::DefaultImportQueue<Block>, sc_service::Error> {
	if is_standalone {
		// aura import queue
		let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
		let client_for_cidp = client.clone();

		sc_consensus_aura::import_queue::<sp_consensus_aura::sr25519::AuthorityPair, _, _, _, _, _>(
			sc_consensus_aura::ImportQueueParams {
				block_import,
				justification_import: None,
				client,
				create_inherent_data_providers: move |block: Hash, ()| {
					let current_para_head = client_for_cidp
					.header(block)
					.expect("Header lookup should succeed")
					.expect("Header passed in as parent should be present in backend.");
				let current_para_block_head =
					Some(polkadot_primitives::HeadData(current_para_head.encode()));
				let client_for_xcm = client_for_cidp.clone();

					async move {
						use sp_runtime::traits::UniqueSaturatedInto;
						let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

						let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
								*timestamp,
								slot_duration,
							);

						let mocked_parachain = MockValidationDataInherentDataProvider {
						// When using manual seal we start from block 0, and it's very unlikely to
						// reach a block number > u32::MAX.
						current_para_block: UniqueSaturatedInto::<u32>::unique_saturated_into(
							*current_para_head.number(),
						),
						para_id: 2013.into(),
						current_para_block_head,
							relay_offset: 1000,
							relay_blocks_per_para_block: 2,
							para_blocks_per_relay_epoch: 0,
							relay_randomness_config: (),
							xcm_config: MockXcmConfig::new(
								&*client_for_xcm,
								block,
								Default::default(),
							),
							raw_downward_messages: vec![],
							raw_horizontal_messages: vec![],
							additional_key_values: None,
						};

						Ok((slot, timestamp, mocked_parachain))
					}
				},
				spawner: &task_manager.spawn_essential_handle(),
				registry: config.prometheus_registry(),
				check_for_equivocation: Default::default(),
				telemetry,
				compatibility_mode: Default::default(),
			},
		)
		.map_err(Into::into)
	} else {
		let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;

		cumulus_client_consensus_aura::import_queue::<
			sp_consensus_aura::sr25519::AuthorityPair,
			_,
			_,
			_,
			_,
			_,
		>(cumulus_client_consensus_aura::ImportQueueParams {
			block_import,
			client: client.clone(),
			create_inherent_data_providers: move |_, _| async move {
				let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

				let slot =
					sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
						*timestamp,
						slot_duration,
					);

				Ok((slot, timestamp))
			},
			registry: config.prometheus_registry(),
			spawner: &task_manager.spawn_essential_handle(),
			telemetry,
		})
		.map_err(Into::into)
	}
}

// start a standalone node which doesn't need to connect to relaychain
pub async fn start_standalone_node(
	config: Configuration,
	evm_tracing_config: crate::evm_tracing_types::EvmTracingConfig,
) -> Result<TaskManager, sc_service::Error> {
	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain: maybe_select_chain,
		transaction_pool,
		other: (_, _, _, frontier_backend),
	} = new_partial::<_>(&config, build_import_queue, true, true)?;

	// Sinks for pubsub notifications.
	// Everytime a new subscription is created, a new mpsc channel is added to the sink pool.
	// The MappingSyncWorker sends through the channel on block import and the subscription
	// emits a notification to the subscriber on receiving a message through this channel. This
	// way we avoid race conditions when using native substrate block import notification
	// stream.
	let pubsub_notification_sinks: fc_mapping_sync::EthereumBlockNotificationSinks<
		fc_mapping_sync::EthereumBlockNotification<Block>,
	> = Default::default();
	let pubsub_notification_sinks = Arc::new(pubsub_notification_sinks);

	let net_config =
		FullNetworkConfiguration::<_, _, sc_network::NetworkWorker<_, _>>::new(&config.network);

	let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			net_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync_params: None,
			block_relay: None,
			metrics: sc_network::NotificationMetrics::new(None),
		})?;

	let role = config.role.clone();
	let force_authoring = config.force_authoring;
	let backoff_authoring_blocks: Option<()> = None;

	let (
		filter_pool,
		fee_history_limit,
		fee_history_cache,
		block_data_cache,
		storage_override,
		tracing_requesters,
		ethapi_cmd,
	) = start_node_evm_impl(
		client.clone(),
		backend.clone(),
		frontier_backend.clone(),
		&mut task_manager,
		&config,
		evm_tracing_config.clone(),
		sync_service.clone(),
		pubsub_notification_sinks.clone(),
	);

	let select_chain = maybe_select_chain
		.expect("In `standalone` mode, `new_partial` will return some `select_chain`; qed");

	// TODO: use fork-aware txpool when paritytech/polkadot-sdk#4639 is included in a stable release
	//       presumably in stable2412
	//       This is a workaround mentioned in https://github.com/paritytech/polkadot-sdk/issues/1202
	let custom_txpool =
		std::sync::Arc::new(crate::custom_txpool::CustomPool::new(transaction_pool.clone()));

	if role.is_authority() {
		let proposer_factory = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			custom_txpool,
			None,
			None,
		);
		// aura
		let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
		let client_for_cidp = client.clone();

		let aura = sc_consensus_aura::start_aura::<
			sp_consensus_aura::sr25519::AuthorityPair,
			_,
			_,
			_,
			_,
			_,
			_,
			_,
			_,
			_,
			_,
		>(StartAuraParams {
			slot_duration: sc_consensus_aura::slot_duration(&*client)?,
			client: client.clone(),
			select_chain,
			block_import: StandaloneBlockImport::new(client.clone()),
			proposer_factory,
			create_inherent_data_providers: move |block: Hash, ()| {
				let current_para_head = client_for_cidp
					.header(block)
					.expect("Header lookup should succeed")
					.expect("Header passed in as parent should be present in backend.");
				let current_para_block_head =
					Some(polkadot_primitives::HeadData(current_para_head.encode()));
				let client_for_xcm = client_for_cidp.clone();

				async move {
					use sp_runtime::traits::UniqueSaturatedInto;
					let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

					let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
							*timestamp,
							slot_duration,
						);

					let mocked_parachain = MockValidationDataInherentDataProvider {
						// When using manual seal we start from block 0, and it's very unlikely to
						// reach a block number > u32::MAX.
						current_para_block: UniqueSaturatedInto::<u32>::unique_saturated_into(
							*current_para_head.number(),
						),
						para_id: 2013.into(),
						current_para_block_head,
						relay_offset: 1000,
						relay_blocks_per_para_block: 2,
						para_blocks_per_relay_epoch: 0,
						relay_randomness_config: (),
						xcm_config: MockXcmConfig::new(&*client_for_xcm, block, Default::default()),
						raw_downward_messages: vec![],
						raw_horizontal_messages: vec![],
						additional_key_values: None,
					};

					Ok((slot, timestamp, mocked_parachain))
				}
			},
			force_authoring,
			backoff_authoring_blocks,
			keystore: keystore_container.keystore(),
			sync_oracle: sync_service.clone(),
			justification_sync_link: sync_service.clone(),
			// We got around 500ms for proposing
			block_proposal_slot_portion: SlotProportion::new(1f32 / 24f32),
			// And a maximum of 750ms if slots are skipped
			max_block_proposal_slot_portion: Some(SlotProportion::new(1f32 / 16f32)),
			telemetry: None,
			compatibility_mode: Default::default(),
		})?;

		// the AURA authoring task is considered essential, i.e. if it
		// fails we take down the service with it.
		task_manager
			.spawn_essential_handle()
			.spawn_blocking("aura", Some("block-authoring"), aura);
	}

	let rpc_builder = {
		let client = client.clone();
		let network = network.clone();
		let transaction_pool = transaction_pool.clone();
		let rpc_config = crate::rpc::EvmTracingConfig {
			tracing_requesters,
			trace_filter_max_count: evm_tracing_config.ethapi_trace_max_count,
			enable_txpool: ethapi_cmd.contains(&EthApiCmd::TxPool),
		};
		let sync = sync_service.clone();
		let pubsub_notification_sinks = pubsub_notification_sinks;

		Box::new(move |deny_unsafe, subscription| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: transaction_pool.clone(),
				graph: transaction_pool.pool().clone(),
				network: network.clone(),
				sync: sync.clone(),
				is_authority: role.is_authority(),
				deny_unsafe,
				frontier_backend: frontier_backend.clone(),
				filter_pool: filter_pool.clone(),
				fee_history_limit,
				fee_history_cache: fee_history_cache.clone(),
				block_data_cache: block_data_cache.clone(),
				storage_override: storage_override.clone(),
				// enable EVM RPC for dev node by default
				enable_evm_rpc: true,
			};

			let pending_consensus_data_provider =
				Box::new(fc_rpc::pending::AuraConsensusDataProvider::new(client.clone()));

			crate::rpc::create_full(
				deps,
				subscription,
				pubsub_notification_sinks.clone(),
				pending_consensus_data_provider,
				rpc_config.clone(),
			)
			.map_err(Into::into)
		})
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		rpc_builder,
		client,
		transaction_pool,
		task_manager: &mut task_manager,
		config,
		keystore: keystore_container.keystore(),
		backend,
		network,
		system_rpc_tx,
		sync_service,
		tx_handler_controller,
		telemetry: None,
	})?;

	network_starter.start_network();
	Ok(task_manager)
}

pub fn start_node_evm_impl(
	client: Arc<ParachainClient>,
	backend: Arc<ParachainBackend>,
	frontier_backend: Arc<fc_db::kv::Backend<Block, ParachainClient>>,
	task_manager: &mut TaskManager,
	config: &Configuration,
	evm_tracing_config: crate::evm_tracing_types::EvmTracingConfig,
	sync_service: Arc<SyncingService<Block>>,
	pubsub_notification_sinks: Arc<
		fc_mapping_sync::EthereumBlockNotificationSinks<
			fc_mapping_sync::EthereumBlockNotification<Block>,
		>,
	>,
) -> (
	FilterPool,
	u64,
	FeeHistoryCache,
	Arc<EthBlockDataCacheTask<Block>>,
	Arc<dyn StorageOverride<Block>>,
	RpcRequesters,
	Vec<EthApiCmd>,
) {
	let prometheus_registry = config.prometheus_registry().cloned();
	let filter_pool: FilterPool = Arc::new(std::sync::Mutex::new(BTreeMap::new()));
	let fee_history_cache: FeeHistoryCache = Arc::new(std::sync::Mutex::new(BTreeMap::new()));
	let storage_override = Arc::new(StorageOverrideHandler::new(client.clone()));

	let ethapi_cmd = evm_tracing_config.ethapi.clone();
	let tracing_requesters =
		if ethapi_cmd.contains(&EthApiCmd::Debug) || ethapi_cmd.contains(&EthApiCmd::Trace) {
			tracing::spawn_tracing_tasks(
				&evm_tracing_config,
				prometheus_registry.clone(),
				tracing::SpawnTasksParams {
					task_manager,
					client: client.clone(),
					substrate_backend: backend.clone(),
					frontier_backend: frontier_backend.clone(),
					filter_pool: Some(filter_pool.clone()),
					storage_override: storage_override.clone(),
				},
			)
		} else {
			tracing::RpcRequesters { debug: None, trace: None }
		};

	// Frontier offchain DB task. Essential.
	// Maps emulated ethereum data to substrate native data.
	task_manager.spawn_essential_handle().spawn(
		"frontier-mapping-sync-worker",
		Some("frontier"),
		fc_mapping_sync::kv::MappingSyncWorker::new(
			client.import_notification_stream(),
			Duration::new(6, 0),
			client.clone(),
			backend,
			storage_override.clone(),
			frontier_backend,
			3,
			0,
			fc_mapping_sync::SyncStrategy::Parachain,
			sync_service,
			pubsub_notification_sinks,
		)
		.for_each(|()| futures::future::ready(())),
	);

	// Frontier `EthFilterApi` maintenance. Manages the pool of user-created Filters.
	// Each filter is allowed to stay in the pool for 100 blocks.
	const FILTER_RETAIN_THRESHOLD: u64 = 100;
	task_manager.spawn_essential_handle().spawn(
		"frontier-filter-pool",
		Some("frontier"),
		fc_rpc::EthTask::filter_pool_task(
			client.clone(),
			filter_pool.clone(),
			FILTER_RETAIN_THRESHOLD,
		),
	);

	const FEE_HISTORY_LIMIT: u64 = 2048;
	task_manager.spawn_essential_handle().spawn(
		"frontier-fee-history",
		Some("frontier"),
		fc_rpc::EthTask::fee_history_task(
			client,
			storage_override.clone(),
			fee_history_cache.clone(),
			FEE_HISTORY_LIMIT,
		),
	);

	let block_data_cache = Arc::new(fc_rpc::EthBlockDataCacheTask::new(
		task_manager.spawn_handle(),
		storage_override.clone(),
		50,
		50,
		prometheus_registry,
	));

	(
		filter_pool,
		FEE_HISTORY_LIMIT,
		fee_history_cache,
		block_data_cache,
		storage_override,
		tracing_requesters,
		ethapi_cmd,
	)
}

/// Start consensus using the lookahead aura collator.
fn start_lookahead_aura_consensus(
	client: Arc<ParachainClient>,
	block_import: ParachainBlockImport,
	prometheus_registry: Option<&Registry>,
	telemetry: Option<TelemetryHandle>,
	task_manager: &TaskManager,
	relay_chain_interface: Arc<dyn RelayChainInterface>,
	transaction_pool: Arc<sc_transaction_pool::FullPool<Block, ParachainClient>>,
	keystore: KeystorePtr,
	relay_chain_slot_duration: Duration,
	para_id: ParaId,
	collator_key: CollatorPair,
	overseer_handle: OverseerHandle,
	announce_block: Arc<dyn Fn(Hash, Option<Vec<u8>>) + Send + Sync>,
	backend: Arc<ParachainBackend>,
) -> Result<(), sc_service::Error> {
	// TODO: use fork-aware txpool when paritytech/polkadot-sdk#4639 is included in a stable release
	//       presumably in stable2412
	//       This is a workaround mentioned in https://github.com/paritytech/polkadot-sdk/issues/1202
	let custom_txpool =
		std::sync::Arc::new(crate::custom_txpool::CustomPool::new(transaction_pool.clone()));

	let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
		task_manager.spawn_handle(),
		client.clone(),
		custom_txpool,
		prometheus_registry,
		telemetry.clone(),
	);

	let collator_service = CollatorService::new(
		client.clone(),
		Arc::new(task_manager.spawn_handle()),
		announce_block,
		client.clone(),
	);

	let params = AuraParams {
		create_inherent_data_providers: move |_, ()| async move { Ok(()) },
		block_import,
		para_client: client.clone(),
		para_backend: backend,
		relay_client: relay_chain_interface,
		code_hash_provider: move |block_hash| {
			client.code_at(block_hash).ok().map(|c| ValidationCode::from(c).hash())
		},
		keystore,
		collator_key,
		para_id,
		overseer_handle,
		relay_chain_slot_duration,
		proposer: Proposer::new(proposer_factory),
		collator_service,
		authoring_duration: Duration::from_millis(1500),
		reinitialize: false,
	};

	let fut = aura::run::<Block, <AuraId as AppCrypto>::Pair, _, _, _, _, _, _, _, _>(params);
	task_manager.spawn_essential_handle().spawn("aura", None, fut);

	Ok(())
}

/// Checks that the hardware meets the requirements and print a warning otherwise.
fn warn_if_slow_hardware(hwbench: &sc_sysinfo::HwBench) {
	// Polkadot para-chains should generally use these requirements to ensure that the relay-chain
	// will not take longer than expected to import its blocks.
	if let Err(err) = frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE.check_hardware(hwbench) {
		log::warn!(
			"⚠️  The hardware does not meet the minimal requirements {} for role 'Authority' find out more at:\n\
			https://wiki.polkadot.network/docs/maintain-guides-how-to-validate-polkadot#reference-hardware",
			err
		);
	}
}
