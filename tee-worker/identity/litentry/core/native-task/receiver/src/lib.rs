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

extern crate alloc;

// re-export module to properly feature gate sgx and regular std environment
#[cfg(all(not(feature = "std"), feature = "sgx"))]
pub mod sgx_reexport_prelude {
	pub use futures_sgx as futures;
}

#[cfg(all(not(feature = "std"), feature = "sgx"))]
pub use crate::sgx_reexport_prelude::*;

mod trusted_call_authenticated;
pub use trusted_call_authenticated::*;

mod types;
pub use types::NativeTaskContext;
use types::*;

use codec::{Decode, Encode};
use futures::executor::ThreadPoolBuilder;
use ita_sgx_runtime::Hash;
use ita_stf::{Getter, TrustedCall, TrustedCallSigned};
use itp_api_client_types::compose_call;
use itp_extrinsics_factory::CreateExtrinsics;
use itp_node_api::metadata::{provider::AccessNodeMetadata, NodeMetadata};
use itp_ocall_api::{EnclaveAttestationOCallApi, EnclaveMetricsOCallApi, EnclaveOnChainOCallApi};
use itp_sgx_crypto::{
	aes256::Aes256Key, key_repository::AccessKey, ShieldingCryptoDecrypt, ShieldingCryptoEncrypt,
};
use itp_stf_executor::traits::StfEnclaveSigning as StfEnclaveSigningTrait;
use itp_stf_primitives::types::TrustedOperation;
use itp_top_pool_author::traits::AuthorApi as AuthorApiTrait;
use itp_types::{parentchain::ParentchainId, OpaqueCall};
use lc_native_task_sender::init_native_task_sender;
use litentry_primitives::{AesRequest, DecryptableRequest, Intent};
use sp_core::{blake2_256, H256};
use std::{
	borrow::ToOwned,
	boxed::Box,
	format,
	string::ToString,
	sync::{
		mpsc::{channel, Sender},
		Arc,
	},
	thread,
};

// TODO: move to config
const THREAD_POOL_SIZE: usize = 10;

pub fn run_native_task_receiver<
	ShieldingKeyRepository,
	AuthorApi,
	StfEnclaveSigning,
	OCallApi,
	ExtrinsicFactory,
	NodeMetadataRepo,
	Aes256KeyRepository,
>(
	context: Arc<
		NativeTaskContext<
			ShieldingKeyRepository,
			AuthorApi,
			StfEnclaveSigning,
			OCallApi,
			ExtrinsicFactory,
			NodeMetadataRepo,
			Aes256KeyRepository,
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
	NodeMetadataRepo: AccessNodeMetadata<MetadataType = NodeMetadata> + Send + Sync + 'static,
	Aes256KeyRepository: AccessKey<KeyType = Aes256Key> + Send + Sync + 'static,
{
	let request_receiver = init_native_task_sender();
	let thread_pool = ThreadPoolBuilder::new()
		.pool_size(THREAD_POOL_SIZE)
		.create()
		.expect("Failed to create thread pool");

	let (native_req_sender, native_req_receiver) = channel::<NativeRequest>();
	let t_pool = thread_pool.clone();

	thread::spawn(move || {
		if let Ok(native_request) = native_req_receiver.recv() {
			t_pool.spawn_ok(async move {
				match native_request {
					// TODO: handle native_request: e.g: Identity verification
				}
			});
		}
	});

	while let Ok(mut req) = request_receiver.recv() {
		let context_pool = context.clone();
		let task_sender_pool = native_req_sender.clone();

		thread_pool.spawn_ok(async move {
			let request = &mut req.request;
			let connection_hash = request.using_encoded(|x| H256::from(blake2_256(x)));
			match handle_request(request, context_pool.clone()) {
				Ok(trusted_call) => handle_trusted_call(
					context_pool.clone(),
					trusted_call,
					connection_hash,
					task_sender_pool,
				),
				Err(e) => {
					log::error!("Failed to get trusted call from request: {:?}", e);
					let res: Result<(), NativeTaskError> = Err(NativeTaskError::InvalidRequest);
					context_pool.author_api.send_rpc_response(connection_hash, res.encode(), false);
				},
			};
		});
	}
	log::warn!("Native task receiver stopped");
}

fn handle_trusted_call<
	ShieldingKeyRepository,
	AuthorApi,
	StfEnclaveSigning,
	OCallApi,
	ExtrinsicFactory,
	NodeMetadataRepo,
	Aes256KeyRepository,
>(
	context: Arc<
		NativeTaskContext<
			ShieldingKeyRepository,
			AuthorApi,
			StfEnclaveSigning,
			OCallApi,
			ExtrinsicFactory,
			NodeMetadataRepo,
			Aes256KeyRepository,
		>,
	>,
	call: TrustedCall,
	connection_hash: H256,
	_tc_sender: Sender<NativeRequest>,
) where
	ShieldingKeyRepository: AccessKey + Send + Sync + 'static,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoEncrypt + ShieldingCryptoDecrypt,
	AuthorApi: AuthorApiTrait<Hash, Hash, TrustedCallSigned, Getter> + Send + Sync + 'static,
	StfEnclaveSigning: StfEnclaveSigningTrait<TrustedCallSigned> + Send + Sync + 'static,
	OCallApi:
		EnclaveOnChainOCallApi + EnclaveMetricsOCallApi + EnclaveAttestationOCallApi + 'static,
	ExtrinsicFactory: CreateExtrinsics + Send + Sync + 'static,
	NodeMetadataRepo: AccessNodeMetadata<MetadataType = NodeMetadata> + Send + Sync + 'static,
	Aes256KeyRepository: AccessKey<KeyType = Aes256Key> + Send + Sync + 'static,
{
	let metadata = match context.node_metadata_repo.get_from_metadata(|m| m.get_metadata().cloned())
	{
		Ok(Some(metadata)) => metadata,
		_ => {
			log::error!("Failed to get node metadata");
			let res: Result<(), NativeTaskError> = Err(NativeTaskError::MetadataRetrievalFailed(
				"Failed to get node metadata".to_string(),
			));
			context.author_api.send_rpc_response(connection_hash, res.encode(), false);
			return
		},
	};

	let (who, intent_call) = match call {
		TrustedCall::request_intent(who, intent) => match intent {
			Intent::SystemRemark(remark) =>
				(who, OpaqueCall::from_tuple(&compose_call!(&metadata, "System", "remark", remark))),
			Intent::TransferNative(transfer) => (
				who,
				OpaqueCall::from_tuple(&compose_call!(
					&metadata,
					"Balances",
					"transfer_allow_death",
					transfer.to,
					transfer.value
				)),
			),
			Intent::CallEthereum(_) | Intent::TransferEthereum(_) => (
				who,
				OpaqueCall::from_tuple(&compose_call!(
					&metadata,
					"OmniAccount",
					"request_intent",
					intent
				)),
			),
		},
		_ => {
			log::warn!("Received unsupported call: {:?}", call);
			let res: Result<(), NativeTaskError> =
				Err(NativeTaskError::UnexpectedCall(format!("Unexpected call: {:?}", call)));
			context.author_api.send_rpc_response(connection_hash, res.encode(), false);
			return
		},
	};

	let omni_account_call = OpaqueCall::from_tuple(&compose_call!(
		&metadata,
		"OmniAccount",
		"dispatch_as_omni_account",
		who.hash(),
		intent_call
	));

	let extrinsic = match context.extrinsic_factory.create_extrinsics(&[omni_account_call], None) {
		Ok(extrinsic) => extrinsic,
		Err(e) => {
			log::error!("Failed to create extrinsic: {:?}", e);
			let res: Result<(), NativeTaskError> =
				Err(NativeTaskError::ExtrinsicConstructionFailed(e.to_string()));
			context.author_api.send_rpc_response(connection_hash, res.encode(), false);
			return
		},
	};

	match context.ocall_api.send_to_parentchain(extrinsic, &ParentchainId::Litentry, true) {
		Ok(_) => {
			let res: Result<(), NativeTaskError> = Ok(());
			context.author_api.send_rpc_response(connection_hash, res.encode(), true);
		},
		Err(e) => {
			log::error!("Failed to send extrinsic to parentchain: {:?}", e);
			let res: Result<(), NativeTaskError> =
				Err(NativeTaskError::ExtrinsicSendingFailed(e.to_string()));
			context.author_api.send_rpc_response(connection_hash, res.encode(), false);
		},
	}
}

fn handle_request<
	ShieldingKeyRepository,
	AuthorApi,
	StfEnclaveSigning,
	OCallApi,
	ExtrinsicFactory,
	NodeMetadataRepo,
	Aes256KeyRepository,
>(
	request: &mut AesRequest,
	context: Arc<
		NativeTaskContext<
			ShieldingKeyRepository,
			AuthorApi,
			StfEnclaveSigning,
			OCallApi,
			ExtrinsicFactory,
			NodeMetadataRepo,
			Aes256KeyRepository,
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
	NodeMetadataRepo: AccessNodeMetadata<MetadataType = NodeMetadata> + Send + Sync + 'static,
	Aes256KeyRepository: AccessKey<KeyType = Aes256Key> + Send + Sync + 'static,
{
	let connection_hash = request.using_encoded(|x| H256::from(blake2_256(x)));
	let enclave_shielding_key = match context.shielding_key.retrieve_key() {
		Ok(value) => value,
		Err(e) => {
			let res: Result<(), NativeTaskError> =
				Err(NativeTaskError::ShieldingKeyRetrievalFailed(format!("{}", e)));
			context.author_api.send_rpc_response(connection_hash, res.encode(), false);
			return Err("Shielding key retrieval failed")
		},
	};
	let tca: TrustedCallAuthenticated = match request
		.decrypt(Box::new(enclave_shielding_key))
		.ok()
		.and_then(|v| {
			TrustedOperation::<TrustedCallAuthenticated, Getter>::decode(&mut v.as_slice()).ok()
		})
		.and_then(|top| top.to_call().cloned())
	{
		Some(tca) => tca,
		None => {
			let res: Result<(), NativeTaskError> =
				Err(NativeTaskError::RequestPayloadDecodingFailed);
			context.author_api.send_rpc_response(connection_hash, res.encode(), false);
			return Err("Request payload decoding failed")
		},
	};
	let mrenclave = match context.ocall_api.get_mrenclave_of_self() {
		Ok(m) => m.m,
		Err(_) => {
			let res: Result<(), NativeTaskError> = Err(NativeTaskError::MrEnclaveRetrievalFailed);
			context.author_api.send_rpc_response(connection_hash, res.encode(), false);
			return Err("MrEnclave retrieval failed")
		},
	};

	let authentication_valid = match tca.authentication {
		TCAuthentication::Web3(signature) => verify_tca_web3_authentication(
			&signature,
			&tca.call,
			tca.nonce,
			&mrenclave,
			&request.shard,
		),
		TCAuthentication::Email(verification_code) =>
			verify_tca_email_authentication(&tca.call, verification_code),
	};

	if !authentication_valid {
		let res: Result<(), NativeTaskError> =
			Err(NativeTaskError::AuthenticationVerificationFailed);
		context.author_api.send_rpc_response(connection_hash, res.encode(), false);
		return Err("Authentication verification failed")
	}

	Ok(tca.call)
}
