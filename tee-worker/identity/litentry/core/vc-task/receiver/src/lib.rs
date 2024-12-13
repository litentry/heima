#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

// re-export module to properly feature gate sgx and regular std environment
#[cfg(all(not(feature = "std"), feature = "sgx"))]
pub mod sgx_reexport_prelude {
	pub use futures_sgx as futures;
}

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
pub use crate::sgx_reexport_prelude::*;

#[cfg(feature = "std")]
use std::sync::Mutex;
#[cfg(feature = "sgx")]
use std::sync::SgxMutex as Mutex;

use codec::{Decode, Encode};
use frame_support::{ensure, sp_runtime::traits::One};
use futures::executor::ThreadPoolBuilder;
use ita_sgx_runtime::{
	pallet_identity_management_tee::get_eligible_identities, BlockNumber, Hash, Runtime,
	VERSION as SIDECHAIN_VERSION,
};

#[cfg(feature = "development")]
use ita_stf::helpers::ensure_alice;
use ita_stf::{
	aes_encrypt_default,
	helpers::ensure_self,
	trusted_call_result::{RequestVCResult, RequestVcErrorDetail, RequestVcResultOrError},
	Getter, TrustedCall, TrustedCallSigned,
};
use itp_enclave_metrics::EnclaveMetric;
use itp_extrinsics_factory::CreateExtrinsics;
use itp_node_api::metadata::{
	pallet_system::SystemConstants, pallet_vcmp::VCMPCallIndexes, provider::AccessNodeMetadata,
	NodeMetadataTrait,
};
use itp_ocall_api::{EnclaveAttestationOCallApi, EnclaveMetricsOCallApi, EnclaveOnChainOCallApi};
use itp_sgx_crypto::{key_repository::AccessKey, ShieldingCryptoDecrypt, ShieldingCryptoEncrypt};
use itp_sgx_externalities::SgxExternalitiesTrait;
use itp_stf_executor::traits::StfEnclaveSigning;
use itp_stf_primitives::{traits::TrustedCallVerification, types::TrustedOperation};
use itp_stf_state_handler::handle_state::HandleState;
use itp_storage::{storage_map_key, storage_value_key, StorageHasher};
use itp_top_pool_author::traits::AuthorApi;
use itp_types::{
	parentchain::ParentchainId, AccountId, BlockNumber as SidechainBlockNumber, OpaqueCall,
	ShardIdentifier, H256,
};
use lc_dynamic_assertion::AssertionLogicRepository;
use lc_evm_dynamic_assertions::AssertionRepositoryItem;
use lc_stf_task_receiver::{handler::assertion::create_credential_str, StfTaskContext};
use lc_vc_task_sender::init_vc_task_sender;
use litentry_macros::if_development_or;
use litentry_primitives::{
	Assertion, AssertionBuildRequest, DecryptableRequest, Identity, ParentchainBlockNumber,
};
use log::*;
use pallet_identity_management_tee::{identity_context::sort_id_graph, IdentityContext};
use sp_core::{blake2_256, H160};
use std::{
	boxed::Box,
	collections::{HashMap, HashSet},
	format,
	string::ToString,
	sync::{
		mpsc::{channel, Sender},
		Arc,
	},
	thread,
	time::Instant,
	vec::Vec,
};

pub fn run_vc_handler_runner<ShieldingKeyRepository, A, S, H, O, Z, N, AR>(
	context: Arc<StfTaskContext<ShieldingKeyRepository, A, S, H, O, AR>>,
	extrinsic_factory: Arc<Z>,
	node_metadata_repo: Arc<N>,
) where
	ShieldingKeyRepository: AccessKey + Send + Sync + 'static,
	<ShieldingKeyRepository as AccessKey>::KeyType:
		ShieldingCryptoEncrypt + ShieldingCryptoDecrypt + 'static,
	A: AuthorApi<Hash, Hash, TrustedCallSigned, Getter> + Send + Sync + 'static,
	S: StfEnclaveSigning<TrustedCallSigned> + Send + Sync + 'static,
	H: HandleState + Send + Sync + 'static,
	H::StateT: SgxExternalitiesTrait,
	O: EnclaveOnChainOCallApi + EnclaveMetricsOCallApi + EnclaveAttestationOCallApi + 'static,
	Z: CreateExtrinsics + Send + Sync + 'static,
	N: AccessNodeMetadata + Send + Sync + 'static,
	N::MetadataType: NodeMetadataTrait,
	AR: AssertionLogicRepository<Id = H160, Item = AssertionRepositoryItem> + Send + Sync + 'static,
{
	let vc_task_receiver = init_vc_task_sender();
	let n_workers = 480;
	let pool = ThreadPoolBuilder::new().pool_size(n_workers).create().unwrap();

	let (tc_sender, tc_receiver) = channel::<(ShardIdentifier, TrustedCall)>();

	// Spawn thread to handle received tasks, to serialize the nonce increase even if multiple threads
	// are submitting trusted calls simultaneously
	let context_cloned = context.clone();
	thread::spawn(move || loop {
		if let Ok((shard, call)) = tc_receiver.recv() {
			if let Err(e) = context_cloned.submit_trusted_call(&shard, None, &call) {
				error!("Submit Trusted Call failed: {:?}", e);
			}
		}
	});

	// use local registry to manage request reponse status
	let req_registry = RequestRegistry::new();

	while let Ok(mut req) = vc_task_receiver.recv() {
		let request = &mut req.request;
		let connection_hash = request.using_encoded(|x| H256::from(blake2_256(x)));
		let enclave_shielding_key = match context.shielding_key.retrieve_key() {
			Ok(value) => value,
			Err(e) => {
				send_vc_response(
					connection_hash,
					context.clone(),
					Err(RequestVcErrorDetail::ShieldingKeyRetrievalFailed(format!("{:?}", e))),
					0u8,
					0u8,
					false,
				);
				continue
			},
		};
		let tcs = match request
			.decrypt(Box::new(enclave_shielding_key))
			.ok()
			.and_then(|v| {
				TrustedOperation::<TrustedCallSigned, Getter>::decode(&mut v.as_slice()).ok()
			})
			.and_then(|top| top.to_call().cloned())
		{
			Some(tcs) => tcs,
			None => {
				send_vc_response(
					connection_hash,
					context.clone(),
					Err(RequestVcErrorDetail::RequestPayloadDecodingFailed),
					0u8,
					0u8,
					false,
				);
				continue
			},
		};
		let mrenclave = match context.ocall_api.get_mrenclave_of_self() {
			Ok(m) => m.m,
			Err(_) => {
				send_vc_response(
					connection_hash,
					context.clone(),
					Err(RequestVcErrorDetail::MrEnclaveRetrievalFailed),
					0u8,
					0u8,
					false,
				);
				continue
			},
		};
		if !tcs.verify_signature(&mrenclave, &request.shard) {
			send_vc_response(
				connection_hash,
				context.clone(),
				Err(RequestVcErrorDetail::SignatureVerificationFailed),
				0u8,
				0u8,
				false,
			);
			continue
		}

		// Until now, preparation work is done. If any error happens, error message would have been returned already.

		if let TrustedCall::request_vc(..) = tcs.call {
			req_registry.add_new_item(connection_hash, 1u8);

			let shard_pool = request.shard;
			let context_pool = context.clone();
			let extrinsic_factory_pool = extrinsic_factory.clone();
			let node_metadata_repo_pool = node_metadata_repo.clone();
			let tc_sender_pool = tc_sender.clone();
			let req_registry_pool = req_registry.clone();
			pool.spawn_ok(async move {
				let response = process_single_request(
					shard_pool,
					context_pool.clone(),
					extrinsic_factory_pool,
					node_metadata_repo_pool,
					tc_sender_pool,
					tcs.call.clone(),
				);

				// Totally fine to `unwrap` here. Because new item was just added above.
				match req_registry_pool.update_item(connection_hash) {
					Ok(do_watch) => {
						send_vc_response(connection_hash, context_pool, response, 0u8, 1, do_watch);
					},
					Err(e) => {
						error!("1 couldn't find connection_hash: {:?}", e);
						send_vc_response(
							connection_hash,
							context_pool,
							Err(RequestVcErrorDetail::ConnectionHashNotFound(e.to_string())),
							0u8,
							1,
							false,
						);
					},
				}
			});
		} else if let TrustedCall::request_batch_vc(
			signer,
			who,
			assertions,
			maybe_key,
			req_ext_hash,
		) = tcs.call
		{
			// Filter out duplicate assertions
			let mut seen: HashSet<H256> = HashSet::new();
			let mut unique_assertions = Vec::new();
			for assertion in assertions.into_iter() {
				let hash = H256::from(blake2_256(&assertion.encode()));
				if seen.insert(hash) {
					unique_assertions.push(Some(assertion));
				} else {
					unique_assertions.push(None);
				}
			}

			let assertion_len = unique_assertions.len() as u8;
			req_registry.add_new_item(connection_hash, assertion_len);
			for (idx, assertion) in unique_assertions.iter().enumerate() {
				if let Some(assertion) = assertion {
					let new_call = TrustedCall::request_vc(
						signer.clone(),
						who.clone(),
						assertion.clone(),
						maybe_key,
						req_ext_hash,
					);

					let shard_pool = request.shard;
					let context_pool = context.clone();
					let extrinsic_factory_pool = extrinsic_factory.clone();
					let node_metadata_repo_pool = node_metadata_repo.clone();
					let tc_sender_pool = tc_sender.clone();
					let req_registry_pool = req_registry.clone();

					pool.spawn_ok(async move {
						let response = process_single_request(
							shard_pool,
							context_pool.clone(),
							extrinsic_factory_pool,
							node_metadata_repo_pool,
							tc_sender_pool,
							new_call,
						);

						// Totally fine to `unwrap` here. Because new item was just added above.
						match req_registry_pool.update_item(connection_hash) {
							Ok(do_watch) => {
								send_vc_response(
									connection_hash,
									context_pool,
									response,
									idx as u8,
									assertion_len,
									do_watch,
								);
							},
							Err(e) => {
								error!("2 couldn't find connection_hash: {:?}", e);
								send_vc_response(
									connection_hash,
									context_pool,
									Err(RequestVcErrorDetail::ConnectionHashNotFound(
										e.to_string(),
									)),
									idx as u8,
									assertion_len,
									false,
								);
							},
						}
					});
				} else {
					// Totally fine to `unwrap` here. Because new item was just added above.
					match req_registry.update_item(connection_hash) {
						Ok(do_watch) => {
							send_vc_response(
								connection_hash,
								context.clone(),
								Err(RequestVcErrorDetail::DuplicateAssertionRequest),
								idx as u8,
								assertion_len,
								do_watch,
							);
						},
						Err(e) => {
							error!("3 couldn't find connection_hash: {:?}", e);
							send_vc_response(
								connection_hash,
								context.clone(),
								Err(RequestVcErrorDetail::ConnectionHashNotFound(e.to_string())),
								idx as u8,
								assertion_len,
								false,
							);
						},
					}
				}
			}
		} else {
			send_vc_response(
				connection_hash,
				context.clone(),
				Err(RequestVcErrorDetail::UnexpectedCall(
					"Expected request_vc or request_batch_vc ".to_string(),
				)),
				0u8,
				0u8,
				false,
			);
		}
	}
	warn!("vc_task_receiver loop terminated");
}

#[derive(Clone)]
struct RequestRegistry {
	status_map: Arc<Mutex<HashMap<H256, AssertionStatus>>>,
}

struct AssertionStatus {
	pub total: u8,
	pub processed: u8,
}

impl RequestRegistry {
	pub fn new() -> Self {
		Self { status_map: Arc::new(Mutex::new(HashMap::new())) }
	}

	pub fn add_new_item(&self, key: H256, total: u8) {
		let mut map = self.status_map.lock().unwrap();
		map.insert(key, AssertionStatus { total, processed: 0u8 });
	}

	// Return value indicates whether some item is still not yet processed.
	pub fn update_item(&self, key: H256) -> Result<bool, &'static str> {
		let mut map = self.status_map.lock().unwrap();

		#[allow(unused_assignments)]
		let mut all_processed = false;

		if let Some(entry) = map.get_mut(&key) {
			entry.processed += 1;
			all_processed = entry.processed == entry.total;
		} else {
			return Err("Item not found in map")
		}

		if all_processed {
			map.remove(&key);
		}

		Ok(!all_processed)
	}
}

fn send_vc_response<ShieldingKeyRepository, A, S, H, O, AR>(
	hash: H256,
	context: Arc<StfTaskContext<ShieldingKeyRepository, A, S, H, O, AR>>,
	result: Result<Vec<u8>, RequestVcErrorDetail>,
	idx: u8,
	len: u8,
	do_watch: bool,
) where
	ShieldingKeyRepository: AccessKey + core::marker::Send + core::marker::Sync + 'static,
	<ShieldingKeyRepository as AccessKey>::KeyType:
		ShieldingCryptoEncrypt + ShieldingCryptoDecrypt + 'static,
	A: AuthorApi<Hash, Hash, TrustedCallSigned, Getter> + Send + Sync + 'static,
	S: StfEnclaveSigning<TrustedCallSigned> + Send + Sync + 'static,
	H: HandleState + Send + Sync + 'static,
	H::StateT: SgxExternalitiesTrait,
	O: EnclaveOnChainOCallApi + EnclaveMetricsOCallApi + EnclaveAttestationOCallApi + 'static,
	AR: AssertionLogicRepository<Id = H160, Item = AssertionRepositoryItem>,
{
	let vc_res = RequestVcResultOrError { result: result.clone(), idx, len };

	context.author_api.send_rpc_response(hash, vc_res.encode(), do_watch);

	if result.is_err() {
		if let Err(e) = context.ocall_api.update_metric(EnclaveMetric::FailedVCIssuance) {
			warn!("Failed to update metric for VC Issuance: {:?}", e);
		}
	} else if let Err(e) = context.ocall_api.update_metric(EnclaveMetric::SuccessfullVCIssuance) {
		warn!("Failed to update metric for VC Issuance: {:?}", e);
	}
}

fn process_single_request<ShieldingKeyRepository, A, S, H, O, Z, N, AR>(
	shard: H256,
	context: Arc<StfTaskContext<ShieldingKeyRepository, A, S, H, O, AR>>,
	extrinsic_factory: Arc<Z>,
	node_metadata_repo: Arc<N>,
	tc_sender: Sender<(ShardIdentifier, TrustedCall)>,
	call: TrustedCall,
) -> Result<Vec<u8>, RequestVcErrorDetail>
where
	ShieldingKeyRepository: AccessKey + core::marker::Send + core::marker::Sync,
	<ShieldingKeyRepository as AccessKey>::KeyType:
		ShieldingCryptoEncrypt + ShieldingCryptoDecrypt + 'static,
	A: AuthorApi<Hash, Hash, TrustedCallSigned, Getter> + Send + Sync + 'static,
	S: StfEnclaveSigning<TrustedCallSigned> + Send + Sync + 'static,
	H: HandleState + Send + Sync + 'static,
	H::StateT: SgxExternalitiesTrait,
	O: EnclaveOnChainOCallApi + EnclaveMetricsOCallApi + EnclaveAttestationOCallApi + 'static,
	Z: CreateExtrinsics + Send + Sync + 'static,
	N: AccessNodeMetadata + Send + Sync + 'static,
	N::MetadataType: NodeMetadataTrait,
	AR: AssertionLogicRepository<Id = H160, Item = AssertionRepositoryItem>,
{
	let start_time = Instant::now();
	let parachain_runtime_version = node_metadata_repo
		.get_from_metadata(|m| {
			m.system_version()
				.map_err(|e| RequestVcErrorDetail::InvalidMetadata(format!("{:?}", e)))
		})
		.map_err(|e| RequestVcErrorDetail::MetadataRetrievalFailed(e.to_string()))??
		.spec_version;
	let sidechain_runtime_version = SIDECHAIN_VERSION.spec_version;

	// The `call` should always be `TrustedCall:request_vc`. Once decided to remove 'request_vc', this part can be refactored regarding the parameters.
	if let TrustedCall::request_vc(signer, who, assertion, maybe_key, req_ext_hash) = call {
		debug!(
			"Processing vc request for {}, assertion: {:?}",
			who.to_did().unwrap_or_default(),
			assertion
		);
		let (mut id_graph, is_already_linked, parachain_block_number, sidechain_block_number) =
			context
				.state_handler
				.execute_on_current(&shard, |state, _| {
					let storage_key = storage_map_key(
						"IdentityManagement",
						"IDGraphs",
						&who,
						&StorageHasher::Blake2_128Concat,
					);

					// `None` means empty IDGraph, thus `unwrap_or_default`
					let mut id_graph: Vec<(Identity, IdentityContext<Runtime>)> = state
						.iter_prefix::<Identity, IdentityContext<Runtime>>(&storage_key)
						.unwrap_or_default();

					// Sorts the IDGraph in place
					sort_id_graph::<Runtime>(&mut id_graph);

					let storage_key = storage_map_key(
						"IdentityManagement",
						"LinkedIdentities",
						&who,
						&StorageHasher::Blake2_128Concat,
					);

					// should never be `None`, but use `unwrap_or_default` to not panic
					let parachain_block_number = state
						.get(&storage_value_key("Parentchain", "Number"))
						.and_then(|v| ParentchainBlockNumber::decode(&mut v.as_slice()).ok())
						.unwrap_or_default();
					let sidechain_block_number = state
						.get(&storage_value_key("System", "Number"))
						.and_then(|v| SidechainBlockNumber::decode(&mut v.as_slice()).ok())
						.unwrap_or_default();

					(
						id_graph,
						state.contains_key(&storage_key),
						parachain_block_number,
						sidechain_block_number,
					)
				})
				.map_err(|e| RequestVcErrorDetail::SidechainDataRetrievalFailed(e.to_string()))?;

		let mut should_create_id_graph = false;
		if id_graph.is_empty() {
			debug!("IDGraph is empty, will pre-create one");
			// To create IDGraph upon first vc request (see P-410), there're two options:
			//
			// 1. synchronous creation:
			// we delegate the vc handling to the STF version, which only returns when the IDGraph is actually created (= InSidechainBlock state).
			// The downside of this method is that the first vc_request processing time is limited to the sidechain block interval.
			//
			// 2. asynchronous creation (this implementation):
			// we check if an IDGraph **could** be created and then process the VC request right away, meanwhile, we submit a trusted call to
			// top pool. So the IDGraph will be created async: in the next sidechain block. In the `RequestVCResult` we return the pre-calculated
			// `mutated_id_graph` and `id_graph_hash`.
			//
			// Corner case: there's a small chance that some IDGraph mutation was injectd in between. For example, a client sends `request_vc` which
			// is closely followed by a `link_identity` request. In this case, the IDGrpah creation resulting from `vc_request` would fail, as
			// the IDGraph would have been created already by that time. But this is OK as long as it reaches the desired state eventually.
			//
			// However, `RequestVCResult` might carry with outdated `mutated_id_graph` and `id_graph_hash` if it lands later than `LinkIdentityResult`.
			// So we call the fields `pre_mutated_id_graph` and `pre_id_graph_hash` to show they are pre-calculated.
			// The client should take proper actions against it, e.g., only use the value when the local IDGraph is empty.
			//
			// Please note we can't mutate the state inside vc-task-receiver via `load_for_mutation` even
			// though it's lock guarded, because: a) it intereferes with the block import on another thread, which eventually
			// cause state mismatch before/after applying the state diff b) it's not guaranteed to be broadcasted to other workers.
			//
			ensure!(!is_already_linked, RequestVcErrorDetail::IdentityAlreadyLinked);
			// we are safe to use `default_web3networks` and `Active` as IDGraph would be non-empty otherwise
			id_graph.push((
				who.clone(),
				IdentityContext::new(BlockNumber::one(), who.default_web3networks()),
			));
			should_create_id_graph = true;
		}
		info!("should_create_id_graph: {}", should_create_id_graph);

		let id_graph_hash = H256::from(blake2_256(&id_graph.encode()));
		let assertion_networks = assertion.get_supported_web3networks();
		let identities = get_eligible_identities(
			id_graph.as_ref(),
			assertion_networks,
			assertion.skip_identity_filtering(),
		);
		ensure!(!identities.is_empty(), RequestVcErrorDetail::NoEligibleIdentity);

		let signer_account =
			signer.to_native_account().ok_or(RequestVcErrorDetail::InvalidSignerAccount)?;

		match assertion {
			// the signer will be checked inside A13, as we don't seem to have access to ocall_api here
			Assertion::A13(_) => (),
			_ => if_development_or!(
				ensure!(
					ensure_self(&signer, &who) || ensure_alice(&signer_account),
					RequestVcErrorDetail::UnauthorizedSigner,
				),
				ensure!(ensure_self(&signer, &who), RequestVcErrorDetail::UnauthorizedSigner)
			),
		}

		let req = AssertionBuildRequest {
			shard,
			signer: signer_account,
			who: who.clone(),
			assertion: assertion.clone(),
			identities,
			top_hash: H256::zero(),
			parachain_block_number,
			sidechain_block_number,
			parachain_runtime_version,
			sidechain_runtime_version,
			maybe_key,
			should_create_id_graph,
			req_ext_hash,
		};

		let (vc_payload, vc_logs) = create_credential_str::<A, S, O, AR>(
			&req,
			context.enclave_signer.clone(),
			context.enclave_account.clone(),
			context.ocall_api.clone(),
			context.data_provider_config.clone(),
			context.assertion_repository.clone(),
		)
		.map_err(|e| RequestVcErrorDetail::AssertionBuildFailed(Box::new(e)))?;

		let call_index = node_metadata_repo
			.get_from_metadata(|m| m.vc_issued_call_indexes())
			.map_err(|e| RequestVcErrorDetail::MetadataRetrievalFailed(e.to_string()))?
			.map_err(|e| RequestVcErrorDetail::InvalidMetadata(format!("{:?}", e)))?;

		let key = maybe_key.ok_or(RequestVcErrorDetail::MissingAesKey)?;
		let call = OpaqueCall::from_tuple(&(
			call_index,
			who.clone(),
			assertion.clone(),
			id_graph_hash,
			req_ext_hash,
		));

		let mutated_id_graph = if should_create_id_graph { id_graph } else { Default::default() };

		let res = RequestVCResult {
			vc_payload: aes_encrypt_default(&key, &vc_payload),
			vc_logs: vc_logs.map(|v| aes_encrypt_default(&key, &v)),
			pre_mutated_id_graph: aes_encrypt_default(&key, &mutated_id_graph.encode()),
			pre_id_graph_hash: id_graph_hash,
		};

		// submit TrustedCall::maybe_create_id_graph to the reciever thread
		let enclave_signer: AccountId = context
			.enclave_signer
			.get_enclave_account()
			.map_err(|_| RequestVcErrorDetail::EnclaveSignerRetrievalFailed)?;
		let c = TrustedCall::maybe_create_id_graph(enclave_signer.into(), who.clone());
		tc_sender
			.send((shard, c))
			.map_err(|e| RequestVcErrorDetail::TrustedCallSendingFailed(e.to_string()))?;

		// this internally fetches nonce from a mutex and then updates it thereby ensuring ordering
		let xt = extrinsic_factory
			.create_extrinsics(&[call], None)
			.map_err(|e| RequestVcErrorDetail::ExtrinsicConstructionFailed(e.to_string()))?;

		context
			.ocall_api
			.send_to_parentchain(xt, &ParentchainId::Litentry, None)
			.map_err(|e| RequestVcErrorDetail::ExtrinsicSendingFailed(e.to_string()))?;

		if let Err(e) = context
			.ocall_api
			.update_metric(EnclaveMetric::VCBuildTime(assertion.clone(), start_time.elapsed()))
		{
			warn!("Failed to update metric for vc build time: {:?}", e);
		}

		info!("Vc issued for {}, assertion: {:?}", who.to_did().unwrap_or_default(), assertion);
		Ok(res.encode())
	} else {
		// Would never come here.
		Err(RequestVcErrorDetail::UnexpectedCall("Expected request_vc".to_string()))
	}
}
