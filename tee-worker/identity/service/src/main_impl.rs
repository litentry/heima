#[cfg(not(feature = "dcap"))]
use crate::utils::check_files;
use crate::{
	account_funding::{setup_reasonable_account_funding, EnclaveAccountInfoProvider},
	config::Config,
	enclave::{
		api::enclave_init,
		tls_ra::{enclave_request_state_provisioning, enclave_run_state_provisioning_server},
	},
	error::Error,
	globals::tokio_handle::{GetTokioHandle, GlobalTokioHandle},
	initialized_service::{
		start_is_initialized_server, InitializationHandler, IsInitialized, TrackInitialization,
	},
	ocall_bridge::{
		bridge_api::Bridge as OCallBridge, component_factory::OCallBridgeComponentFactory,
	},
	parentchain_handler::{HandleParentchain, ParentchainHandler},
	prometheus_metrics::{start_metrics_server, EnclaveMetricsReceiver, MetricsHandler},
	setup,
	sidechain_setup::{sidechain_init_block_production, sidechain_start_untrusted_rpc_server},
	sync_block_broadcaster::SyncBlockBroadcaster,
	sync_state, tests,
	utils::extract_shard,
	worker::Worker,
	worker_peers_updater::WorkerPeersRegistry,
};
use base58::ToBase58;
use clap::{load_yaml, App, ArgMatches};
use codec::{Decode, Encode};
use ita_parentchain_interface::integritee::{Hash, Header};
use itp_enclave_api::{
	enclave_base::EnclaveBase,
	remote_attestation::{RemoteAttestation, TlsRemoteAttestation},
	sidechain::Sidechain,
	Enclave,
};
use itp_node_api::{
	api_client::{AccountApi, PalletTeebagApi, ParentchainApi},
	metadata::NodeMetadata,
	node_api_factory::{CreateNodeApi, NodeApiFactory},
};
use itp_settings::worker_mode::{ProvideWorkerMode, WorkerMode, WorkerModeProvider};
use its_peer_fetch::{
	block_fetch_client::BlockFetcher, untrusted_peer_fetch::UntrustedPeerFetcher,
};
use its_primitives::types::block::SignedBlock as SignedSidechainBlock;
use its_storage::{interface::FetchBlocks, BlockPruner, SidechainStorageLock};
use lc_data_providers::DataProviderConfig;
use litentry_macros::if_development_or;
use litentry_primitives::{Enclave as TeebagEnclave, ShardIdentifier, WorkerType};
use log::*;
use regex::Regex;
use serde_json::Value;
use sgx_types::*;
use sp_runtime::traits::Header as HeaderT;
use substrate_api_client::{
	api::XtStatus, rpc::HandleSubscription, GetAccountInformation, GetBalance, GetChainInfo,
	SubmitAndWatch, SubscribeChain, SubscribeEvents,
};

#[cfg(feature = "dcap")]
use litentry_primitives::extract_tcb_info_from_raw_dcap_quote;

use crate::error::ServiceResult;
use itp_types::parentchain::{AccountId, Balance, ParentchainId};
use sp_core::crypto::{AccountId32, Ss58Codec};
use sp_keyring::AccountKeyring;
use sp_runtime::MultiSigner;
use std::{
	collections::HashSet, fmt::Debug, path::PathBuf, str, str::Utf8Error, sync::Arc, thread,
	time::Duration,
};
use substrate_api_client::ac_node_api::{EventRecord, Phase::ApplyExtrinsic};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "link-binary")]
pub type EnclaveWorker =
	Worker<Config, NodeApiFactory, Enclave, InitializationHandler<WorkerModeProvider>>;

pub(crate) fn main() {
	// Setup logging
	env_logger::builder()
		.format_timestamp(Some(env_logger::TimestampPrecision::Millis))
		.init();

	let yml = load_yaml!("cli.yml");
	let matches = App::from_yaml(yml).get_matches();

	let config = Config::from(&matches);

	GlobalTokioHandle::initialize();

	// log this information, don't println because some python scripts for GA rely on the
	// stdout from the service
	#[cfg(not(feature = "development"))]
	info!("*** Starting service in SGX production mode");
	#[cfg(feature = "development")]
	info!("*** Starting service in SGX debug mode");

	info!("*** Running worker in mode: {:?} \n", WorkerModeProvider::worker_mode());

	let mut lockfile = PathBuf::from(config.data_dir());
	lockfile.push("worker.lock");
	while std::fs::metadata(lockfile.clone()).is_ok() {
		info!("lockfile is present, will wait for it to disappear {:?}", lockfile);
		thread::sleep(std::time::Duration::from_secs(5));
	}

	let clean_reset = matches.is_present("clean-reset");
	if clean_reset {
		crate::setup::purge_files_from_dir(config.data_dir()).unwrap();
	}

	// build the entire dependency tree
	let tokio_handle = Arc::new(GlobalTokioHandle {});
	let sidechain_blockstorage = Arc::new(
		SidechainStorageLock::<SignedSidechainBlock>::from_base_path(
			config.data_dir().to_path_buf(),
		)
		.unwrap(),
	);
	let node_api_factory =
		Arc::new(NodeApiFactory::new(config.litentry_rpc_endpoint(), AccountKeyring::Alice.pair()));
	let enclave = Arc::new(enclave_init(&config).unwrap());
	let initialization_handler = Arc::new(InitializationHandler::default());
	let worker = Arc::new(EnclaveWorker::new(
		config.clone(),
		enclave.clone(),
		node_api_factory.clone(),
		initialization_handler.clone(),
		HashSet::new(),
	));
	let sync_block_broadcaster =
		Arc::new(SyncBlockBroadcaster::new(tokio_handle.clone(), worker.clone()));
	let peer_updater = Arc::new(WorkerPeersRegistry::new(worker));
	let untrusted_peer_fetcher = UntrustedPeerFetcher::new(node_api_factory.clone());
	let peer_sidechain_block_fetcher =
		Arc::new(BlockFetcher::<SignedSidechainBlock, _>::new(untrusted_peer_fetcher));
	let enclave_metrics_receiver = Arc::new(EnclaveMetricsReceiver {});

	let maybe_target_a_parentchain_api_factory = config
		.target_a_parentchain_rpc_endpoint()
		.map(|url| Arc::new(NodeApiFactory::new(url, AccountKeyring::Alice.pair())));

	let maybe_target_b_parentchain_api_factory = config
		.target_b_parentchain_rpc_endpoint()
		.map(|url| Arc::new(NodeApiFactory::new(url, AccountKeyring::Alice.pair())));

	// initialize o-call bridge with a concrete factory implementation
	OCallBridge::initialize(Arc::new(OCallBridgeComponentFactory::new(
		node_api_factory.clone(),
		maybe_target_a_parentchain_api_factory,
		maybe_target_b_parentchain_api_factory,
		sync_block_broadcaster,
		enclave.clone(),
		sidechain_blockstorage.clone(),
		peer_updater,
		peer_sidechain_block_fetcher,
		tokio_handle.clone(),
		enclave_metrics_receiver,
	)));

	#[cfg(feature = "dcap")]
	let quoting_enclave_target_info = match enclave.qe_get_target_info() {
		Ok(target_info) => Some(target_info),
		Err(e) => {
			warn!("Setting up DCAP - qe_get_target_info failed with error: {:?}, continuing.", e);
			None
		},
	};
	#[cfg(feature = "dcap")]
	let quote_size = match enclave.qe_get_quote_size() {
		Ok(size) => Some(size),
		Err(e) => {
			warn!("Setting up DCAP - qe_get_quote_size failed with error: {:?}, continuing.", e);
			None
		},
	};

	#[cfg(not(feature = "dcap"))]
	let quoting_enclave_target_info = None;
	#[cfg(not(feature = "dcap"))]
	let quote_size = None;

	if let Some(run_config) = config.run_config() {
		// init in-memory store, it should be done after the o-call bridge is initialized
		if let Err(e) = enclave.init_in_memory_state() {
			error!("Failed to initialize in-memory state: {:?}", e);
		}

		let shard = extract_shard(run_config.shard(), enclave.as_ref());

		info!("Worker Config: {:?}", config);

		// litentry: start the mock-server if enabled
		if config.enable_mock_server {
			#[cfg(any(feature = "mock-server", feature = "development"))]
			{
				let mock_server_port = config
					.try_parse_mock_server_port()
					.expect("mock server port to be a valid port number");
				thread::spawn(move || {
					info!("*** Starting mock server");
					let _ = lc_mock_server::run(mock_server_port);
				});
			}
		}

		if clean_reset {
			setup::initialize_shard_and_keys(enclave.as_ref(), &shard).unwrap();
		}

		let node_api =
			node_api_factory.create_api().expect("Failed to create parentchain node API");

		start_worker::<_, _, _, _, WorkerModeProvider>(
			config,
			&shard,
			enclave,
			sidechain_blockstorage,
			node_api,
			tokio_handle,
			initialization_handler,
			quoting_enclave_target_info,
			quote_size,
		);
	} else if let Some(smatches) = matches.subcommand_matches("request-state") {
		info!("*** Requesting state from a registered worker \n");
		let node_api =
			node_api_factory.create_api().expect("Failed to create parentchain node API");
		sync_state::sync_state::<_, _, WorkerModeProvider>(
			&node_api,
			&extract_shard(smatches.value_of("shard"), enclave.as_ref()),
			enclave.as_ref(),
			smatches.is_present("skip-ra"),
		);
	} else if matches.is_present("shielding-key") {
		setup::generate_shielding_key_file(enclave.as_ref());
	} else if matches.is_present("signing-key") {
		setup::generate_signing_key_file(enclave.as_ref());
		let tee_accountid = enclave_account(enclave.as_ref());
		println!("Enclave signing account: {:}", &tee_accountid.to_ss58check());
	} else if matches.is_present("dump-ra") {
		info!("*** Perform RA and dump cert to disk");
		#[cfg(not(feature = "dcap"))]
		enclave.dump_ias_ra_cert_to_disk().unwrap();
		#[cfg(feature = "dcap")]
		{
			let skip_ra = false;
			let dcap_quote = enclave.generate_dcap_ra_quote(skip_ra).unwrap();
			let (fmspc, _tcb_info) = extract_tcb_info_from_raw_dcap_quote(&dcap_quote).unwrap();
			enclave.dump_dcap_collateral_to_disk(fmspc).unwrap();
			enclave.dump_dcap_ra_cert_to_disk().unwrap();
		}
	} else if matches.is_present("mrenclave") {
		let mrenclave = enclave.get_fingerprint().unwrap();
		let hex_value = hex::encode(mrenclave);
		println!("MRENCLAVE hex: {}", hex_value);
		println!("MRENCLAVE base58: {}", mrenclave.encode().to_base58());
	} else if let Some(sub_matches) = matches.subcommand_matches("init-shard") {
		setup::init_shard(
			enclave.as_ref(),
			&extract_shard(sub_matches.value_of("shard"), enclave.as_ref()),
		);
	} else if let Some(sub_matches) = matches.subcommand_matches("test") {
		if sub_matches.is_present("provisioning-server") {
			info!("*** Running Enclave MU-RA TLS server\n");
			enclave_run_state_provisioning_server(
				enclave.as_ref(),
				sgx_quote_sign_type_t::SGX_UNLINKABLE_SIGNATURE,
				quoting_enclave_target_info.as_ref(),
				quote_size.as_ref(),
				&config.mu_ra_url(),
				sub_matches.is_present("skip-ra"),
			);
			info!("[+] Done!");
		} else if sub_matches.is_present("provisioning-client") {
			info!("*** Running Enclave MU-RA TLS client\n");
			let shard = extract_shard(sub_matches.value_of("shard"), enclave.as_ref());
			enclave_request_state_provisioning(
				enclave.as_ref(),
				sgx_quote_sign_type_t::SGX_UNLINKABLE_SIGNATURE,
				&config.mu_ra_url_external(),
				&shard,
				sub_matches.is_present("skip-ra"),
			)
			.unwrap();
			info!("[+] Done!");
		} else {
			tests::run_enclave_tests(sub_matches);
		}
	} else if let Some(sub_matches) = matches.subcommand_matches("migrate-shard") {
		let new_shard = extract_shard(None, enclave.as_ref());
		setup::migrate_shard(enclave.as_ref(), &new_shard);
		let new_shard_name = new_shard.encode().to_base58();
		setup::remove_old_shards(config.data_dir(), &new_shard_name);
	} else if let Some(sub_matches) = matches.subcommand_matches("upload-id-graph") {
		let tee_accountid = enclave_account(enclave.as_ref());
		let shard = extract_shard(sub_matches.value_of("shard"), enclave.as_ref());
		info!("shard is {:?}", shard);
		let node_api =
			node_api_factory.create_api().expect("Failed to create parentchain node API");
		init_parentchain(&enclave, &node_api, &tee_accountid, ParentchainId::Litentry, &shard);
		enclave.upload_id_graph();
	} else {
		info!("For options: use --help");
	}
}

/// FIXME: needs some discussion (restructuring?)
#[allow(clippy::too_many_arguments)]
fn start_worker<E, T, D, InitializationHandler, WorkerModeProvider>(
	config: Config,
	shard: &ShardIdentifier,
	enclave: Arc<E>,
	sidechain_storage: Arc<D>,
	litentry_rpc_api: ParentchainApi,
	tokio_handle_getter: Arc<T>,
	initialization_handler: Arc<InitializationHandler>,
	quoting_enclave_target_info: Option<sgx_target_info_t>,
	quote_size: Option<u32>,
) where
	T: GetTokioHandle,
	E: EnclaveBase + Sidechain + RemoteAttestation + TlsRemoteAttestation + Clone,
	D: BlockPruner + FetchBlocks<SignedSidechainBlock> + Sync + Send + 'static,
	InitializationHandler: TrackInitialization + IsInitialized + Sync + Send + 'static,
	WorkerModeProvider: ProvideWorkerMode,
{
	let run_config = config.run_config().clone().expect("Run config missing");
	let skip_ra = run_config.skip_ra();

	#[cfg(feature = "sidechain")]
	let flavor_str = "sidechain";
	#[cfg(feature = "offchain-worker")]
	let flavor_str = "offchain-worker";
	#[cfg(not(any(feature = "offchain-worker", feature = "sidechain")))]
	let flavor_str = "offchain-worker";

	info!("Litentry Worker for {} v{}", flavor_str, VERSION);

	#[cfg(feature = "dcap")]
	info!("  DCAP is enabled");
	#[cfg(not(feature = "dcap"))]
	info!("  DCAP is disabled");
	#[cfg(not(feature = "development"))]
	info!("  Production Mode is enabled");
	#[cfg(feature = "development")]
	info!("  Production Mode is disabled");
	#[cfg(feature = "evm")]
	info!("  EVM is enabled");
	#[cfg(not(feature = "evm"))]
	info!("  EVM is disabled");

	info!("starting worker on shard {}", shard.encode().to_base58());
	// ------------------------------------------------------------------------
	// check for required files
	if !skip_ra {
		#[cfg(not(feature = "dcap"))]
		check_files();
	}
	// ------------------------------------------------------------------------
	// initialize the enclave
	let mrenclave = enclave.get_fingerprint().unwrap();
	info!("MRENCLAVE={}", mrenclave.0.to_base58());
	info!("MRENCLAVE in hex {:?}", hex::encode(mrenclave));

	// ------------------------------------------------------------------------
	// let new workers call us for key provisioning
	info!("MU-RA server listening on {}", config.mu_ra_url());
	let is_development_mode = run_config.dev();
	let ra_url = config.mu_ra_url();
	let enclave_api_key_prov = enclave.clone();
	thread::spawn(move || {
		enclave_run_state_provisioning_server(
			enclave_api_key_prov.as_ref(),
			sgx_quote_sign_type_t::SGX_UNLINKABLE_SIGNATURE,
			quoting_enclave_target_info.as_ref(),
			quote_size.as_ref(),
			&ra_url,
			skip_ra,
		);
		info!("State provisioning server stopped.");
	});

	let tokio_handle = tokio_handle_getter.get_handle();

	// ------------------------------------------------------------------------
	// Get the public key of our TEE.
	let tee_accountid = enclave_account(enclave.as_ref());
	info!("Enclave account {:} ", &tee_accountid.to_ss58check());

	// ------------------------------------------------------------------------
	// Start `is_initialized` server.
	let untrusted_http_server_port = config
		.try_parse_untrusted_http_server_port()
		.expect("untrusted http server port to be a valid port number");
	let initialization_handler_clone = initialization_handler.clone();
	tokio_handle.spawn(async move {
		if let Err(e) =
			start_is_initialized_server(initialization_handler_clone, untrusted_http_server_port)
				.await
		{
			error!("Unexpected error in `is_initialized` server: {:?}", e);
		}
	});

	// ------------------------------------------------------------------------
	// Start prometheus metrics server.
	if config.enable_metrics_server() {
		let enclave_wallet = Arc::new(EnclaveAccountInfoProvider::new(
			litentry_rpc_api.clone(),
			tee_accountid.clone(),
		));
		let metrics_handler = Arc::new(MetricsHandler::new(enclave_wallet));
		let metrics_server_port = config
			.try_parse_metrics_server_port()
			.expect("metrics server port to be a valid port number");
		tokio_handle.spawn(async move {
			if let Err(e) = start_metrics_server(metrics_handler, metrics_server_port).await {
				error!("Unexpected error in Prometheus metrics server: {:?}", e);
			}
		});
	}

	// ------------------------------------------------------------------------
	// Start trusted worker rpc server
	if WorkerModeProvider::worker_mode() == WorkerMode::Sidechain
		|| WorkerModeProvider::worker_mode() == WorkerMode::OffChainWorker
	{
		let direct_invocation_server_addr = config.trusted_worker_url_internal();
		let enclave_for_direct_invocation = enclave.clone();
		thread::spawn(move || {
			info!(
				"[+] Trusted RPC direct invocation server listening on {}",
				direct_invocation_server_addr
			);
			enclave_for_direct_invocation
				.init_direct_invocation_server(direct_invocation_server_addr)
				.unwrap();
			info!("[+] RPC direct invocation server shut down");
		});
	}

	// ------------------------------------------------------------------------
	// Start untrusted worker rpc server.
	if WorkerModeProvider::worker_mode() == WorkerMode::Sidechain {
		sidechain_start_untrusted_rpc_server(&config, sidechain_storage.clone(), &tokio_handle);
	}

	// ------------------------------------------------------------------------
	// Init parentchain specific stuff. Needed early for parentchain communication.
	let (integritee_parentchain_handler, integritee_last_synced_header_at_last_run) =
		init_parentchain(
			&enclave,
			&litentry_rpc_api,
			&tee_accountid,
			ParentchainId::Litentry,
			shard,
		);

	#[cfg(feature = "dcap")]
	register_collateral(&litentry_rpc_api, &*enclave, &tee_accountid, is_development_mode, skip_ra);

	let trusted_url = config.trusted_worker_url_external();

	#[cfg(feature = "attesteer")]
	fetch_marblerun_events_every_hour(
		litentry_rpc_api.clone(),
		enclave.clone(),
		tee_accountid.clone(),
		is_development_mode,
		trusted_url.clone(),
		run_config.marblerun_base_url().to_string(),
	);

	// ------------------------------------------------------------------------
	// Perform a remote attestation and get an unchecked extrinsic back.

	if skip_ra {
		info!("[!] skipping remote attestation. Registering enclave without attestation report.");
	} else {
		info!("[!] creating remote attestation report and create enclave register extrinsic.");
	};

	#[cfg(feature = "dcap")]
	enclave.set_sgx_qpl_logging().expect("QPL logging setup failed");

	// Litentry: send the registration extrinsic regardless of being registered or not,
	//           the reason is the mrenclave could change in between, so we rely on the
	//           on-chain logic to handle everything.
	//           this is the same behavior as upstream
	let register_enclave_block_hash = register_enclave(
		enclave.clone(),
		&litentry_rpc_api,
		&tee_accountid,
		&trusted_url,
		skip_ra,
		is_development_mode,
	)
	.expect("enclave RA registration must be successful to continue");

	let api_register_enclave_xt_header =
		litentry_rpc_api.get_header(Some(register_enclave_block_hash)).unwrap().unwrap();

	// TODO: #1451: Fix api-client type hacks
	let register_enclave_xt_header =
		Header::decode(&mut api_register_enclave_xt_header.encode().as_slice())
			.expect("Can decode previously encoded header; qed");

	info!(
		"[+] Enclave registered at block number: {:?}, hash: {:?}",
		register_enclave_xt_header.number(),
		register_enclave_xt_header.hash()
	);
	// double-check
	let my_enclave = litentry_rpc_api
		.enclave(&tee_accountid, None)
		.unwrap()
		.expect("our enclave should be registered at this point");
	trace!("verified that our enclave is registered: {:?}", my_enclave);

	// Litentry:
	// the logic differs from upstream a bit here (due to different impl in parachain pallet),
	// theoretically the `primary_enclave_identifier_for_shard` should never be empty, unless the previous
	// registration failed (e.g. due to unexpected mrenclave). In that case it's expected not to continue with anything.
	//
	// in case it's non-empty, it relies on the check of `enclave.get_shard_creation_info` to tell if this worker
	// has run before - this is similar to upstream.
	// There're a few cases:
	// 1. `--clean-reset` is set, then the shard should have been initalized earlier already and it's empty state anyway
	// 2. `--clean-reset` is not set:
	//    2a. `get_shard_creation_info` is empty and we are primary worker => it's never run before => init everything
	//    2b. `get_shard_creation_info` is empty and we are non-primary worker => it's never run before => request to sync state
	//    2c. `get_shard_creation_info` is non-empty it's run before => do nothing
	let (we_are_primary_validateer, re_init_parentchain_needed) =
		match litentry_rpc_api
			.primary_enclave_identifier_for_shard(WorkerType::Identity, shard, None)
			.unwrap()
		{
			Some(account) => {
				let first_run = enclave
					.get_shard_creation_info(shard)
					.unwrap()
					.for_parentchain(ParentchainId::Litentry)
					.is_none();
				if account == tee_accountid {
					info!("We are the primary worker, first_run: {}", first_run);
					if first_run {
						enclave.init_shard(shard.encode()).unwrap();
						enclave
							.init_shard_creation_parentchain_header(
								shard,
								&ParentchainId::Litentry,
								&register_enclave_xt_header,
							)
							.unwrap();
						debug!("shard config should be initialized on litentry network now");
						(true, true)
					} else {
						(true, false)
					}
				} else {
					info!("We are NOT primary worker, the primary worker is {}", account);
					if first_run {
						// obtain provisioning from last active worker as this hasn't been done before
						info!("my state doesn't know the creation header of the shard. will request provisioning");
						sync_state::sync_state::<_, _, WorkerModeProvider>(
							&litentry_rpc_api,
							&shard,
							enclave.as_ref(),
							skip_ra,
						);

						info!("re-register the enclave to update the keys after provisioning");
						register_enclave(
							enclave.clone(),
							&litentry_rpc_api,
							&tee_accountid,
							&trusted_url,
							skip_ra,
							is_development_mode,
						)
						.expect("enclave RA registration must be successful to continue");
					}
					(false, true)
				}
			},
			None => {
				panic!("No primary enclave account is found - was the enclave successfully registered?");
			},
		};
	debug!("getting shard creation: {:?}", enclave.get_shard_creation_info(shard));
	initialization_handler.registered_on_parentchain();

	let (integritee_parentchain_handler, integritee_last_synced_header_at_last_run) =
		if re_init_parentchain_needed {
			// re-initialize integritee parentchain to make sure to use creation_header for fast-sync or the provisioned light client state
			init_parentchain(
				&enclave,
				&litentry_rpc_api,
				&tee_accountid,
				ParentchainId::Litentry,
				shard,
			)
		} else {
			(integritee_parentchain_handler, integritee_last_synced_header_at_last_run)
		};

	match WorkerModeProvider::worker_mode() {
		WorkerMode::OffChainWorker => {
			info!("[Litentry:OCW] Finished initializing light client, syncing parentchain...");

			// Syncing all parentchain blocks, this might take a while..
			let last_synced_header = integritee_parentchain_handler
				.sync_parentchain_until_latest_finalized(
					integritee_last_synced_header_at_last_run,
					0,
					*shard,
					true,
				)
				.unwrap();

			start_parentchain_header_subscription_thread(
				integritee_parentchain_handler,
				last_synced_header,
				*shard,
			);

			info!("skipping shard vault check because not yet supported for offchain worker");
		},
		WorkerMode::Sidechain => {
			info!("[Litentry:SCV] Finished initializing light client, syncing litentry parentchain...");

			// Litentry: apply skipped parentchain block
			let parentchain_start_block = config
				.try_parse_parentchain_start_block()
				.expect("parentchain start block to be a valid number");

			let last_synced_header = if we_are_primary_validateer {
				info!("We're the first validateer to be registered, syncing parentchain blocks until the one we have registered ourselves on.");
				integritee_parentchain_handler
					.await_sync_and_import_parentchain_until_at_least(
						&integritee_last_synced_header_at_last_run,
						&register_enclave_xt_header,
						parentchain_start_block,
						*shard,
					)
					.unwrap()
			} else {
				integritee_last_synced_header_at_last_run
			};

			info!(
				"*** [+] last_synced_header: {}, config.parentchain_start_block: {}",
				last_synced_header.number, parentchain_start_block
			);

			start_parentchain_header_subscription_thread(
				integritee_parentchain_handler,
				last_synced_header,
				*shard,
			);

			spawn_worker_for_shard_polling(shard, litentry_rpc_api.clone(), initialization_handler);
		},
	}

	let maybe_target_a_rpc_api = if let Some(url) = config.target_a_parentchain_rpc_endpoint() {
		Some(init_target_parentchain(
			&enclave,
			&tee_accountid,
			url,
			shard,
			ParentchainId::TargetA,
			is_development_mode,
		))
	} else {
		None
	};

	let maybe_target_b_rpc_api = if let Some(url) = config.target_b_parentchain_rpc_endpoint() {
		Some(init_target_parentchain(
			&enclave,
			&tee_accountid,
			url,
			shard,
			ParentchainId::TargetB,
			is_development_mode,
		))
	} else {
		None
	};

	if WorkerModeProvider::worker_mode() == WorkerMode::Sidechain {
		info!("[Litentry:SCV] starting block production");
		let last_synced_header = sidechain_init_block_production(
			enclave.clone(),
			sidechain_storage,
			config.clone().fail_slot_mode,
			config.fail_at,
		)
		.unwrap();
	}

	ita_parentchain_interface::event_subscriber::subscribe_to_parentchain_events(
		&litentry_rpc_api,
		ParentchainId::Litentry,
	);
}

fn init_target_parentchain<E>(
	enclave: &Arc<E>,
	tee_account_id: &AccountId32,
	url: String,
	shard: &ShardIdentifier,
	parentchain_id: ParentchainId,
	is_development_mode: bool,
) -> ParentchainApi
where
	E: EnclaveBase + Sidechain,
{
	info!("Initializing parentchain {:?} with url: {}", parentchain_id, url);
	let node_api = NodeApiFactory::new(url, AccountKeyring::Alice.pair())
		.create_api()
		.unwrap_or_else(|_| panic!("[{:?}] Failed to create parentchain node API", parentchain_id));

	setup_reasonable_account_funding(
		&node_api,
		tee_account_id,
		parentchain_id,
		is_development_mode,
	)
	.unwrap_or_else(|_| {
		panic!("[{:?}] Could not fund parentchain enclave account", parentchain_id)
	});

	// we attempt to set shard creation for this parentchain in case it hasn't been done before
	let api_head = node_api.get_header(None).unwrap().unwrap();
	// TODO: #1451: Fix api-client type hacks
	let head = Header::decode(&mut api_head.encode().as_slice())
		.expect("Can decode previously encoded header; qed");
	// we ignore failure
	let _ = enclave.init_shard_creation_parentchain_header(shard, &parentchain_id, &head);

	let (parentchain_handler, last_synched_header) =
		init_parentchain(enclave, &node_api, tee_account_id, parentchain_id, shard);

	info!("[{:?}] Finished initializing light client, syncing parentchain...", parentchain_id);

	// Syncing all parentchain blocks, this might take a while..
	let last_synched_header = parentchain_handler
		.sync_parentchain_until_latest_finalized(last_synched_header, 0, *shard, true)
		.unwrap();

	start_parentchain_header_subscription_thread(
		parentchain_handler.clone(),
		last_synched_header,
		*shard,
	);

	let parentchain_init_params = parentchain_handler.parentchain_init_params.clone();

	let node_api_clone = node_api.clone();
	thread::Builder::new()
		.name(format!("{:?}_parentchain_event_subscription", parentchain_id))
		.spawn(move || {
			ita_parentchain_interface::event_subscriber::subscribe_to_parentchain_events(
				&node_api_clone,
				parentchain_id,
			)
		})
		.unwrap();
	node_api
}

fn init_parentchain<E>(
	enclave: &Arc<E>,
	node_api: &ParentchainApi,
	tee_account_id: &AccountId32,
	parentchain_id: ParentchainId,
	shard: &ShardIdentifier,
) -> (Arc<ParentchainHandler<ParentchainApi, E>>, Header)
where
	E: EnclaveBase + Sidechain,
{
	let parentchain_handler = Arc::new(
		ParentchainHandler::new_with_automatic_light_client_allocation(
			node_api.clone(),
			enclave.clone(),
			parentchain_id,
			*shard,
		)
		.unwrap(),
	);
	let last_synced_header = parentchain_handler.init_parentchain_components().unwrap();
	info!("[{:?}] last synced parentchain block: {}", parentchain_id, last_synced_header.number);

	let nonce = node_api.get_account_next_index(tee_account_id).unwrap();
	info!("[{:?}] Enclave nonce = {:?}", parentchain_id, nonce);
	enclave.set_nonce(nonce, parentchain_id).unwrap_or_else(|_| {
		panic!("[{:?}] Could not set nonce of enclave. Returning here...", parentchain_id)
	});

	let metadata = node_api.metadata().clone();
	let runtime_spec_version = node_api.runtime_version().spec_version;
	let runtime_transaction_version = node_api.runtime_version().transaction_version;
	enclave
		.set_node_metadata(
			NodeMetadata::new(metadata, runtime_spec_version, runtime_transaction_version).encode(),
			parentchain_id,
		)
		.unwrap_or_else(|_| {
			panic!("[{:?}] Could not set the node metadata in the enclave", parentchain_id)
		});

	(parentchain_handler, last_synced_header)
}

/// Start polling loop to wait until we have a worker for a shard registered on
/// the parentchain (TEEBAG EnclaveIdentifier). This is the pre-requisite to be
/// considered initialized and ready for the next worker to start (in sidechain mode only).
fn spawn_worker_for_shard_polling<InitializationHandler>(
	shard: &ShardIdentifier,
	node_api: ParentchainApi,
	initialization_handler: Arc<InitializationHandler>,
) where
	InitializationHandler: TrackInitialization + Sync + Send + 'static,
{
	let shard_for_initialized = *shard;
	thread::spawn(move || {
		const POLL_INTERVAL_SECS: u64 = 2;

		loop {
			info!("Polling for worker for shard ({} seconds interval)", POLL_INTERVAL_SECS);
			if let Ok(Some(_account)) = node_api.primary_enclave_identifier_for_shard(
				WorkerType::Identity,
				&shard_for_initialized,
				None,
			) {
				// Set that the service is initialized.
				initialization_handler.worker_for_shard_registered();
				info!("[+] Found `WorkerForShard` on parentchain state",);
				break
			}
			thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS));
		}
	});
}

#[cfg(feature = "attesteer")]
fn fetch_marblerun_events_every_hour<E>(
	api: ParentchainApi,
	enclave: Arc<E>,
	accountid: AccountId32,
	is_development_mode: bool,
	url: String,
	marblerun_base_url: String,
) where
	E: RemoteAttestation + Clone + Sync + Send + 'static,
{
	let enclave = enclave.clone();
	let handle = thread::spawn(move || {
		const POLL_INTERVAL_5_MINUTES_IN_SECS: u64 = 5 * 60;
		loop {
			info!("Polling marblerun events for quotes to register");
			register_quotes_from_marblerun(
				&api,
				enclave.clone(),
				&accountid,
				is_development_mode,
				url.clone(),
				&marblerun_base_url,
			);

			thread::sleep(Duration::from_secs(POLL_INTERVAL_5_MINUTES_IN_SECS));
		}
	});

	handle.join().unwrap()
}
#[cfg(feature = "attesteer")]
fn register_quotes_from_marblerun(
	api: &ParentchainApi,
	enclave: Arc<dyn RemoteAttestation>,
	accountid: &AccountId32,
	is_development_mode: bool,
	url: String,
	marblerun_base_url: &str,
) {
	let enclave = enclave.as_ref();
	let events = crate::prometheus_metrics::fetch_marblerun_events(marblerun_base_url)
		.map_err(|e| {
			info!("Fetching events from Marblerun failed with: {:?}, continuing with 0 events.", e);
		})
		.unwrap_or_default();
	let quotes: Vec<&[u8]> =
		events.iter().map(|event| event.get_quote_without_prepended_bytes()).collect();

	for quote in quotes {
		match enclave.generate_dcap_ra_extrinsic_from_quote(url.clone(), &quote) {
			Ok(xt) => {
				send_litentry_extrinsic(xt, api, accountid, is_development_mode);
			},
			Err(e) => {
				error!("Extracting information from quote failed: {}", e)
			},
		}
	}
}
#[cfg(feature = "dcap")]
fn register_collateral(
	api: &ParentchainApi,
	enclave: &dyn RemoteAttestation,
	accountid: &AccountId32,
	is_development_mode: bool,
	skip_ra: bool,
) {
	//TODO generate_dcap_ra_quote() does not really need skip_ra, rethink how many layers skip_ra should be passed along
	if !skip_ra {
		let dcap_quote = enclave.generate_dcap_ra_quote(skip_ra).unwrap();
		let (fmspc, _tcb_info) = extract_tcb_info_from_raw_dcap_quote(&dcap_quote).unwrap();
		info!("[>] DCAP setup: register QE collateral");
		let uxt = enclave.generate_register_quoting_enclave_extrinsic(fmspc).unwrap();
		send_litentry_extrinsic(uxt, api, accountid, is_development_mode);

		info!("[>] DCAP setup: register TCB info");
		let uxt = enclave.generate_register_tcb_info_extrinsic(fmspc).unwrap();
		send_litentry_extrinsic(uxt, api, accountid, is_development_mode);
	}
}

fn send_litentry_extrinsic(
	extrinsic: Vec<u8>,
	api: &ParentchainApi,
	fee_payer: &AccountId32,
	is_development_mode: bool,
) -> ServiceResult<Hash> {
	let fee = crate::account_funding::estimate_fee(api, extrinsic.clone())?;
	let ed = api.get_existential_deposit()?;
	let free = api.get_free_balance(fee_payer)?;
	let missing_funds = fee.saturating_add(ed).saturating_sub(free);
	info!("[Litentry] send extrinsic");
	debug!("fee: {:?}, ed: {:?}, free: {:?} => missing: {:?}", fee, ed, free, missing_funds);
	trace!(
		"  encoded extrinsic len: {}, payload: 0x{:}",
		extrinsic.len(),
		hex::encode(extrinsic.clone())
	);

	if missing_funds > 0 {
		setup_reasonable_account_funding(
			api,
			fee_payer,
			ParentchainId::Litentry,
			is_development_mode,
		)?
	}

	match api.submit_and_watch_opaque_extrinsic_until(&extrinsic.into(), XtStatus::Finalized) {
		Ok(xt_report) => {
			info!(
				"[+] L1 extrinsic success. extrinsic hash: {:?} / status: {:?}",
				xt_report.extrinsic_hash, xt_report.status
			);
			xt_report.block_hash.ok_or(Error::Custom("no extrinsic hash returned".into()))
		},
		Err(e) => {
			panic!("Extrinsic failed {:?} parentchain genesis: {:?}", e, api.genesis_hash());
		},
	}
}

fn start_parentchain_header_subscription_thread<E: EnclaveBase + Sidechain>(
	parentchain_handler: Arc<ParentchainHandler<ParentchainApi, E>>,
	last_synced_header: Header,
	shard: ShardIdentifier,
) {
	let parentchain_id = *parentchain_handler.parentchain_id();
	thread::Builder::new()
		.name(format!("{:?}_parentchain_sync_loop", parentchain_id))
		.spawn(move || {
			if let Err(e) =
				subscribe_to_parentchain_new_headers(parentchain_handler, last_synced_header, shard)
			{
				error!(
					"[{:?}] parentchain block syncing terminated with a failure: {:?}",
					parentchain_id, e
				);
			}
			info!("[!] [{:?}] parentchain block syncing has terminated", parentchain_id);
		})
		.unwrap();
}

/// Subscribe to the node API finalized heads stream and trigger a parent chain sync
/// upon receiving a new header.
fn subscribe_to_parentchain_new_headers<E: EnclaveBase + Sidechain>(
	parentchain_handler: Arc<ParentchainHandler<ParentchainApi, E>>,
	mut last_synced_header: Header,
	shard: ShardIdentifier,
) -> Result<(), Error> {
	// TODO: this should be implemented by parentchain_handler directly, and not via
	// exposed parentchain_api
	let mut subscription = parentchain_handler
		.parentchain_api()
		.subscribe_finalized_heads()
		.map_err(Error::ApiClient)?;

	// TODO(Kai@Litentry):
	// originally we had an outer loop to try to handle the disconnection,
	// see https://github.com/litentry/litentry-parachain/commit/b8059d0fad928e4bba99178451cd0d473791c437
	// but I reverted it because:
	// - no graceful shutdown, we could have many mpsc channel when it doesn't go right
	// - we might have multiple `sync_parentchain` running concurrently, which causes chaos in enclave side
	// - I still feel it's only a workaround, not a perfect solution
	//
	// TODO: now the sync will panic if disconnected - it heavily relies on the worker-restart to work (even manually)
	let parentchain_id = parentchain_handler.parentchain_id();
	loop {
		let new_header = subscription
			.next()
			.ok_or(Error::ApiSubscriptionDisconnected)?
			.map_err(|e| Error::ApiClient(e.into()))?;

		info!(
			"[{:?}] Received finalized header update ({}), syncing parent chain...",
			parentchain_id, new_header.number
		);

		last_synced_header = parentchain_handler.sync_parentchain_until_latest_finalized(
			last_synced_header,
			0,
			shard,
			false,
		)?;
	}
}

/// Get the public signing key of the TEE.
pub fn enclave_account<E: EnclaveBase>(enclave_api: &E) -> AccountId32 {
	let tee_public = enclave_api.get_ecc_signing_pubkey().unwrap();
	trace!("[+] Got ed25519 account of TEE = {}", tee_public.to_ss58check());
	AccountId32::from(*tee_public.as_array_ref())
}

fn register_enclave<E>(
	enclave: Arc<E>,
	api: &ParentchainApi,
	tee_account: &AccountId32,
	url: &str,
	skip_ra: bool,
	is_development_mode: bool,
) -> ServiceResult<Hash>
where
	E: EnclaveBase + Sidechain + RemoteAttestation + TlsRemoteAttestation + Clone,
{
	#[cfg(not(feature = "dcap"))]
	let register_xt = move || enclave.generate_ias_ra_extrinsic(url, skip_ra).unwrap();
	#[cfg(feature = "dcap")]
	let register_xt = move || enclave.generate_dcap_ra_extrinsic(url, skip_ra).unwrap();

	info!("[+] Send register enclave extrinsic");
	send_litentry_extrinsic(register_xt(), api, tee_account, is_development_mode)
}
