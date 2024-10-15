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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

// re-export module to properly feature gate sgx and regular std environment
#[cfg(all(not(feature = "std"), feature = "sgx"))]
pub mod sgx_reexport_prelude {
	pub use futures_sgx as futures;
}

#[cfg(all(not(feature = "std"), feature = "sgx"))]
pub use crate::sgx_reexport_prelude::*;

use codec::{Decode, Encode};
use futures::executor::ThreadPoolBuilder;
use ita_sgx_runtime::Hash;
use ita_stf::{Getter, TrustedCall, TrustedCallSigned};
use itp_extrinsics_factory::CreateExtrinsics;
use itp_node_api::metadata::{provider::AccessNodeMetadata, NodeMetadataTrait};
use itp_ocall_api::{EnclaveAttestationOCallApi, EnclaveMetricsOCallApi, EnclaveOnChainOCallApi};
use itp_sgx_crypto::{key_repository::AccessKey, ShieldingCryptoDecrypt, ShieldingCryptoEncrypt};
use itp_stf_executor::traits::StfEnclaveSigning as StfEnclaveSigningTrait;
use itp_stf_primitives::{traits::TrustedCallVerification, types::TrustedOperation};
use itp_top_pool_author::traits::AuthorApi as AuthorApiTrait;
use lc_data_providers::DataProviderConfig;
use lc_direct_call_request_sender::init_direct_call_request_sender;
use litentry_primitives::{AesOutput, AesRequest, DecryptableRequest};
use sp_core::{blake2_256, ed25519::Pair as Ed25519Pair, H256};
use std::{
	format,
	sync::{
		mpsc::{channel, Sender},
		Arc,
	},
	thread,
};

pub struct DirectCallRequestContext<
	ShieldingKeyRepository,
	AuthorApi,
	StfEnclaveSigning,
	OCallApi,
	ExtrinsicFactory,
	NodeMetadataRepo,
> where
	ShieldingKeyRepository: AccessKey + Send + Sync + 'static,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoEncrypt + ShieldingCryptoDecrypt,
	AuthorApi: AuthorApiTrait<Hash, Hash, TrustedCallSigned, Getter> + Send + Sync + 'static,
	StfEnclaveSigning: StfEnclaveSigningTrait<TrustedCallSigned> + Send + Sync + 'static,
	OCallApi:
		EnclaveOnChainOCallApi + EnclaveMetricsOCallApi + EnclaveAttestationOCallApi + 'static,
	ExtrinsicFactory: CreateExtrinsics + Send + Sync + 'static,
	NodeMetadataRepo: AccessNodeMetadata + Send + Sync + 'static,
	NodeMetadataRepo::MetadataType: NodeMetadataTrait,
{
	pub shielding_key: Arc<ShieldingKeyRepository>,
	pub author_api: Arc<AuthorApi>,
	pub enclave_signer: Arc<StfEnclaveSigning>,
	pub enclave_account: Arc<Ed25519Pair>,
	pub ocall_api: Arc<OCallApi>,
	pub data_provider_config: Arc<DataProviderConfig>,
	pub extrinsic_factory: Arc<ExtrinsicFactory>,
	pub node_metadata_repo: Arc<NodeMetadataRepo>,
}

impl<
		ShieldingKeyRepository,
		AuthorApi,
		StfEnclaveSigning,
		OCallApi,
		ExtrinsicFactory,
		NodeMetadataRepo,
	>
	DirectCallRequestContext<
		ShieldingKeyRepository,
		AuthorApi,
		StfEnclaveSigning,
		OCallApi,
		ExtrinsicFactory,
		NodeMetadataRepo,
	> where
	ShieldingKeyRepository: AccessKey + Send + Sync + 'static,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoEncrypt + ShieldingCryptoDecrypt,
	AuthorApi: AuthorApiTrait<Hash, Hash, TrustedCallSigned, Getter> + Send + Sync + 'static,
	StfEnclaveSigning: StfEnclaveSigningTrait<TrustedCallSigned> + Send + Sync + 'static,
	OCallApi:
		EnclaveOnChainOCallApi + EnclaveMetricsOCallApi + EnclaveAttestationOCallApi + 'static,
	ExtrinsicFactory: CreateExtrinsics + Send + Sync + 'static,
	NodeMetadataRepo: AccessNodeMetadata + Send + Sync + 'static,
	NodeMetadataRepo::MetadataType: NodeMetadataTrait,
{
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		shielding_key: Arc<ShieldingKeyRepository>,
		author_api: Arc<AuthorApi>,
		enclave_signer: Arc<StfEnclaveSigning>,
		enclave_account: Arc<Ed25519Pair>,
		ocall_api: Arc<OCallApi>,
		data_provider_config: Arc<DataProviderConfig>,
		extrinsic_factory: Arc<ExtrinsicFactory>,
		node_metadata_repo: Arc<NodeMetadataRepo>,
	) -> Self {
		Self {
			shielding_key,
			author_api,
			enclave_signer,
			enclave_account,
			ocall_api,
			data_provider_config,
			extrinsic_factory,
			node_metadata_repo,
		}
	}
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum DirectCallErrorDetail {
	UnexpectedCall(String),
	ShieldingKeyRetrievalFailed(String), // Stringified itp_sgx_crypto::Error
	RequestPayloadDecodingFailed,
	ParentchainDataRetrievalFailed(String), // Stringified itp_stf_state_handler::Error
	InvalidSignerAccount,
	UnauthorizedSigner,
	MissingAesKey,
	MrEnclaveRetrievalFailed,
	EnclaveSignerRetrievalFailed,
	SignatureVerificationFailed,
	ConnectionHashNotFound(String),
	MetadataRetrievalFailed(String), // Stringified itp_node_api_metadata_provider::Error
	InvalidMetadata(String),         // Stringified itp_node_api_metadata::Error
	TrustedCallSendingFailed(String), // Stringified mpsc::SendError<(H256, TrustedCall)>
	CallSendingFailed(String),
	ExtrinsicConstructionFailed(String), // Stringified itp_extrinsics_factory::Error
	ExtrinsicSendingFailed(String),      // Stringified sgx_status_t
}

// split this type and move it to the right place once we have the new architecture
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct DirectCallResult {
	pub account_store_hash: H256,
	pub mutated_account_store: AesOutput,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub struct DirectCallRequestResult {
	pub result: Result<DirectCallResult, DirectCallErrorDetail>,
}

enum DirectCallTask {
	// TODO: Define the tasks
}

// TODO: move to config
const THREAD_POOL_SIZE: usize = 10;

pub fn run_direct_call_request_receiver<
	ShieldingKeyRepository,
	AuthorApi,
	StfEnclaveSigning,
	OCallApi,
	ExtrinsicFactory,
	NodeMetadataRepo,
>(
	context: Arc<
		DirectCallRequestContext<
			ShieldingKeyRepository,
			AuthorApi,
			StfEnclaveSigning,
			OCallApi,
			ExtrinsicFactory,
			NodeMetadataRepo,
		>,
	>,
) where
	ShieldingKeyRepository: AccessKey + Send + Sync + 'static,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoEncrypt + ShieldingCryptoDecrypt,
	AuthorApi: AuthorApiTrait<Hash, Hash, TrustedCallSigned, Getter> + Send + Sync + 'static,
	StfEnclaveSigning: StfEnclaveSigningTrait<TrustedCallSigned> + Send + Sync + 'static,
	OCallApi:
		EnclaveOnChainOCallApi + EnclaveMetricsOCallApi + EnclaveAttestationOCallApi + 'static,
	ExtrinsicFactory: CreateExtrinsics + Send + Sync + 'static,
	NodeMetadataRepo: AccessNodeMetadata + Send + Sync + 'static,
	NodeMetadataRepo::MetadataType: NodeMetadataTrait,
{
	let request_receiver = init_direct_call_request_sender();
	let thread_pool = ThreadPoolBuilder::new()
		.pool_size(THREAD_POOL_SIZE)
		.create()
		.expect("Failed to create thread pool");

	let (task_sender, task_receiver) = channel::<DirectCallTask>();
	let t_pool = thread_pool.clone();

	thread::spawn(move || {
		if let Ok(task) = task_receiver.recv() {
			t_pool.spawn_ok(async move {
				match task {
					// TODO: handle the task: e.g: Identity verification
				}
			});
		}
	});

	while let Ok(mut req) = request_receiver.recv() {
		let context_pool = context.clone();
		let task_sender_pool = task_sender.clone();

		thread_pool.spawn_ok(async move {
			let request = &mut req.request;
			let connection_hash = request.using_encoded(|x| H256::from(blake2_256(x)));
			match get_trusted_call_from_request(request, context_pool.clone()) {
				Ok(call) => {
					if let Err(e) = process_trusted_call(call, connection_hash, task_sender_pool) {
						let res = DirectCallRequestResult { result: Err(e) };
						context_pool.author_api.send_rpc_response(
							connection_hash,
							res.encode(),
							false,
						);
					}
				},
				Err(_) => {
					log::error!("Failed to get trusted call from request");
				},
			};
		});
	}
}

fn process_trusted_call(
	call: TrustedCall,
	_connection_hash: H256,
	_tc_sender: Sender<DirectCallTask>,
) -> Result<DirectCallResult, DirectCallErrorDetail> {
	match call {
		// TODO: handle AccountStore related calls
		TrustedCall::noop(_) =>
			Err(DirectCallErrorDetail::UnexpectedCall(format!("Unexpected call: {:?}", call))),
		_ => Err(DirectCallErrorDetail::UnexpectedCall(format!("Unexpected call: {:?}", call))),
	}
}

fn get_trusted_call_from_request<
	ShieldingKeyRepository,
	AuthorApi,
	StfEnclaveSigning,
	OCallApi,
	ExtrinsicFactory,
	NodeMetadataRepo,
>(
	request: &mut AesRequest,
	context: Arc<
		DirectCallRequestContext<
			ShieldingKeyRepository,
			AuthorApi,
			StfEnclaveSigning,
			OCallApi,
			ExtrinsicFactory,
			NodeMetadataRepo,
		>,
	>,
) -> Result<TrustedCall, &'static str>
where
	ShieldingKeyRepository: AccessKey + Send + Sync + 'static,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoEncrypt + ShieldingCryptoDecrypt,
	AuthorApi: AuthorApiTrait<Hash, Hash, TrustedCallSigned, Getter> + Send + Sync + 'static,
	StfEnclaveSigning: StfEnclaveSigningTrait<TrustedCallSigned> + Send + Sync + 'static,
	OCallApi:
		EnclaveOnChainOCallApi + EnclaveMetricsOCallApi + EnclaveAttestationOCallApi + 'static,
	ExtrinsicFactory: CreateExtrinsics + Send + Sync + 'static,
	NodeMetadataRepo: AccessNodeMetadata + Send + Sync + 'static,
	NodeMetadataRepo::MetadataType: NodeMetadataTrait,
{
	let connection_hash = request.using_encoded(|x| H256::from(blake2_256(x)));
	let enclave_shielding_key = match context.shielding_key.retrieve_key() {
		Ok(value) => value,
		Err(e) => {
			let res = DirectCallRequestResult {
				result: Err(DirectCallErrorDetail::ShieldingKeyRetrievalFailed(format!("{}", e))),
			};
			context.author_api.send_rpc_response(connection_hash, res.encode(), false);
			return Err("Shielding key retrieval failed")
		},
	};
	let tcs: TrustedCallSigned = match request
		.decrypt(Box::new(enclave_shielding_key))
		.ok()
		.and_then(|v| TrustedOperation::<TrustedCallSigned, Getter>::decode(&mut v.as_slice()).ok())
		.and_then(|top| top.to_call().cloned())
	{
		Some(tcs) => tcs,
		None => {
			let res = DirectCallRequestResult {
				result: Err(DirectCallErrorDetail::RequestPayloadDecodingFailed),
			};
			context.author_api.send_rpc_response(connection_hash, res.encode(), false);
			return Err("Request payload decoding failed")
		},
	};
	let mrenclave = match context.ocall_api.get_mrenclave_of_self() {
		Ok(m) => m.m,
		Err(_) => {
			let res = DirectCallRequestResult {
				result: Err(DirectCallErrorDetail::MrEnclaveRetrievalFailed),
			};
			context.author_api.send_rpc_response(connection_hash, res.encode(), false);
			return Err("MrEnclave retrieval failed")
		},
	};
	if !tcs.verify_signature(&mrenclave, &request.shard) {
		let res = DirectCallRequestResult {
			result: Err(DirectCallErrorDetail::SignatureVerificationFailed),
		};
		context.author_api.send_rpc_response(connection_hash, res.encode(), false);
		return Err("Signature verification failed")
	}

	Ok(tcs.call)
}
