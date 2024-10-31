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

mod helpers;
use helpers::*;

mod types;
pub use types::NativeTaskContext;
use types::*;

use codec::{Decode, Encode};
use futures::executor::ThreadPoolBuilder;
use ita_sgx_runtime::Hash;
use ita_stf::{aes_encrypt_default, Getter, TrustedCall, TrustedCallSigned};
use itp_extrinsics_factory::CreateExtrinsics;
use itp_node_api::{
	api_client::{compose_call, ExtrinsicReport, XtStatus},
	metadata::{provider::AccessNodeMetadata, NodeMetadata},
};
use itp_ocall_api::{EnclaveAttestationOCallApi, EnclaveMetricsOCallApi, EnclaveOnChainOCallApi};
use itp_sgx_crypto::{
	aes256::Aes256Key, key_repository::AccessKey, ShieldingCryptoDecrypt, ShieldingCryptoEncrypt,
};
use itp_stf_executor::traits::StfEnclaveSigning as StfEnclaveSigningTrait;
use itp_stf_primitives::types::TrustedOperation;
use itp_top_pool_author::traits::AuthorApi as AuthorApiTrait;
use itp_types::{parentchain::ParentchainId, OpaqueCall};
use lc_identity_verification::web2::verify as verify_web2_identity;
use lc_native_task_sender::init_native_task_sender;
use lc_omni_account::{
	GetOmniAccountInfo, InMemoryStore as OmniAccountStore, OmniAccountRepository,
};
use litentry_primitives::{
	AesRequest, DecryptableRequest, Identity, Intent, MemberAccount, ValidationData,
};
use sp_core::{blake2_256, H256};
use std::{borrow::ToOwned, boxed::Box, format, string::ToString, sync::Arc};

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

	while let Ok(mut req) = request_receiver.recv() {
		let context_pool = context.clone();
		thread_pool.spawn_ok(async move {
			let request = &mut req.request;
			let connection_hash = request.using_encoded(|x| H256::from(blake2_256(x)));
			match handle_request(request, context_pool.clone()) {
				Ok(trusted_call) =>
					handle_trusted_call(context_pool.clone(), trusted_call, connection_hash),
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

	let create_dispatch_as_omni_account_call = |member_identity_hash: H256, call: OpaqueCall| {
		OpaqueCall::from_tuple(&compose_call!(
			&metadata,
			"OmniAccount",
			"dispatch_as_omni_account",
			member_identity_hash,
			call
		))
	};

	let opaque_call = match call {
		TrustedCall::request_intent(who, intent) => match intent {
			Intent::SystemRemark(remark) => create_dispatch_as_omni_account_call(
				who.hash(),
				OpaqueCall::from_tuple(&compose_call!(&metadata, "System", "remark", remark)),
			),
			Intent::TransferNative(transfer) => create_dispatch_as_omni_account_call(
				who.hash(),
				OpaqueCall::from_tuple(&compose_call!(
					&metadata,
					"Balances",
					"transfer_allow_death",
					transfer.to,
					transfer.value
				)),
			),
			Intent::CallEthereum(_) | Intent::TransferEthereum(_) =>
				create_dispatch_as_omni_account_call(
					who.hash(),
					OpaqueCall::from_tuple(&compose_call!(
						&metadata,
						"OmniAccount",
						"request_intent",
						intent
					)),
				),
		},
		TrustedCall::create_account_store(who) => {
			let create_account_store_call = OpaqueCall::from_tuple(&compose_call!(
				&metadata,
				"OmniAccount",
				"create_account_store",
				who
			));

			create_account_store_call
		},
		TrustedCall::add_account(who, identity, validation_data, public_account) => {
			let omni_account_repository = OmniAccountRepository::new(context.ocall_api.clone());
			let omni_account = match OmniAccountStore::get_omni_account(who.hash()) {
				Ok(Some(account)) => account,
				_ => {
					let res: Result<(), NativeTaskError> = Err(NativeTaskError::UnauthorizedSigner);
					context.author_api.send_rpc_response(connection_hash, res.encode(), false);
					return
				},
			};
			let nonce = omni_account_repository.get_nonce(omni_account.clone()).unwrap_or(0);
			let raw_msg = get_expected_raw_message(&omni_account, &identity, nonce);

			let validation_result = match validation_data {
				ValidationData::Web2(validation_data) =>
					if !identity.is_web2() {
						Err(NativeTaskError::InvalidMemberIdentity)
					} else {
						verify_web2_identity(
							&who,
							&identity,
							&raw_msg,
							&validation_data,
							&context.data_provider_config,
						)
						.map_err(|e| {
							log::error!("Failed to verify web2 identity: {:?}", e);
							NativeTaskError::ValidationDataVerificationFailed
						})
					},
				ValidationData::Web3(validation_data) =>
					if !identity.is_web3() {
						Err(NativeTaskError::InvalidMemberIdentity)
					} else {
						verify_web3_identity(&identity, &raw_msg, &validation_data).map_err(|e| {
							log::error!("Failed to verify web3 identity: {:?}", e);
							NativeTaskError::ValidationDataVerificationFailed
						})
					},
			};

			if let Err(e) = validation_result {
				let res: Result<(), NativeTaskError> = Err(e);
				context.author_api.send_rpc_response(connection_hash, res.encode(), false);
				return
			}

			let member_account = match create_member_account(
				context.aes256_key_repository.clone(),
				&identity,
				public_account,
			) {
				Ok(account) => account,
				Err(e) => {
					log::error!("Failed to create member account: {:?}", e);
					let res: Result<(), NativeTaskError> = Err(e);
					context.author_api.send_rpc_response(connection_hash, res.encode(), false);
					return
				},
			};

			create_dispatch_as_omni_account_call(
				who.hash(),
				OpaqueCall::from_tuple(&compose_call!(
					&metadata,
					"OmniAccount",
					"add_account",
					member_account
				)),
			)
		},
		_ => {
			log::warn!("Received unsupported call: {:?}", call);
			let res: Result<(), NativeTaskError> =
				Err(NativeTaskError::UnexpectedCall(format!("Unexpected call: {:?}", call)));
			context.author_api.send_rpc_response(connection_hash, res.encode(), false);
			return
		},
	};

	let extrinsic = match context.extrinsic_factory.create_extrinsics(&[opaque_call], None) {
		Ok(extrinsic) => extrinsic,
		Err(e) => {
			log::error!("Failed to create extrinsic: {:?}", e);
			let res: Result<(), NativeTaskError> =
				Err(NativeTaskError::ExtrinsicConstructionFailed(e.to_string()));
			context.author_api.send_rpc_response(connection_hash, res.encode(), false);
			return
		},
	};

	match context.ocall_api.send_to_parentchain(
		extrinsic,
		&ParentchainId::Litentry,
		Some(XtStatus::Finalized),
	) {
		Ok(extrinsic_reports) =>
			if let Some(report) = extrinsic_reports.first() {
				let res: Result<ExtrinsicReport<H256>, NativeTaskError> = Ok(report.clone());
				context.author_api.send_rpc_response(connection_hash, res.encode(), false);
			} else {
				log::error!("Failed to get extrinsic report");
				let res: Result<(), NativeTaskError> =
					Err(NativeTaskError::ExtrinsicSendingFailed(
						"Failed to get extrinsic report".to_string(),
					));
				context.author_api.send_rpc_response(connection_hash, res.encode(), false);
			},
		Err(e) => {
			log::error!("Failed to send extrinsic to parentchain: {:?}", e);
			let res: Result<(), NativeTaskError> =
				Err(NativeTaskError::ExtrinsicSendingFailed(e.to_string()));
			context.author_api.send_rpc_response(connection_hash, res.encode(), false);
		},
	}
}

fn create_member_account<Aes256KeyRepository>(
	aes256_key_repository: Arc<Aes256KeyRepository>,
	identity: &Identity,
	is_public: bool,
) -> Result<MemberAccount, NativeTaskError>
where
	Aes256KeyRepository: AccessKey<KeyType = Aes256Key>,
{
	if is_public {
		return Ok(MemberAccount::Public(identity.clone()))
	}

	let aes_key = match aes256_key_repository.retrieve_key() {
		Ok(key) => Ok(key),
		Err(e) => {
			log::error!("Failed to retrieve aes key: {:?}", e);
			Err(NativeTaskError::MissingAesKey)
		},
	}?;

	Ok(MemberAccount::Private(
		aes_encrypt_default(&aes_key, &identity.encode()).encode(),
		identity.hash(),
	))
}
