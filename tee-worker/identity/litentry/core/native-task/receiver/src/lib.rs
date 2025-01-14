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

#[cfg(feature = "std")]
use std::sync::Mutex;
#[cfg(feature = "sgx")]
use std::sync::SgxMutex as Mutex;

mod authentication_utils;
use authentication_utils::{
	verify_tca_auth_token_authentication, verify_tca_email_authentication,
	verify_tca_oauth2_authentication, verify_tca_web3_authentication,
};

mod trusted_call_handlers;

mod trusted_call_authenticated;
pub use trusted_call_authenticated::*;
use trusted_call_handlers::request_vc_handler::{
	handle_request_vc, send_vc_response, VcRequestRegistry,
};

mod types;
pub use types::NativeTaskContext;
use types::*;

use alloc::{borrow::ToOwned, boxed::Box, format, string::ToString, sync::Arc, vec::Vec};
use codec::{Compact, Decode, Encode};
use futures::executor::{ThreadPool, ThreadPoolBuilder};
use ita_stf::{
	helpers::{get_expected_raw_message, verify_web3_identity},
	trusted_call_result::RequestVcErrorDetail,
	Getter, TrustedCall, TrustedCallSigned,
};
use itp_extrinsics_factory::CreateExtrinsics;
use itp_node_api::{
	api_client::{compose_call, ParentchainAdditionalParams, XtStatus},
	metadata::{provider::AccessNodeMetadata, NodeMetadata},
};
use itp_ocall_api::{EnclaveAttestationOCallApi, EnclaveMetricsOCallApi, EnclaveOnChainOCallApi};
use itp_sgx_crypto::{
	aes256::Aes256Key, aes_encrypt_default, key_repository::AccessKey, ShieldingCryptoDecrypt,
	ShieldingCryptoEncrypt,
};
use itp_sgx_externalities::SgxExternalitiesTrait;
use itp_stf_executor::traits::StfEnclaveSigning as StfEnclaveSigningTrait;
use itp_stf_primitives::types::TrustedOperation;
use itp_stf_state_handler::handle_state::HandleState;
use itp_top_pool_author::traits::AuthorApi as AuthorApiTrait;
use itp_types::{
	parentchain::{Address, ParachainHeader, ParentchainId},
	AccountId, OpaqueCall,
};
use itp_utils::stringify::account_id_to_string_without_prefix;
use lc_authentication::jwt;
use lc_dynamic_assertion::AssertionLogicRepository;
use lc_evm_dynamic_assertions::AssertionRepositoryItem;
use lc_identity_verification::web2::verify as verify_web2_identity;
use lc_native_task_sender::init_native_task_sender;
use lc_omni_account::{
	GetOmniAccountInfo, InMemoryStore as OmniAccountStore, OmniAccountRepository,
};
use litentry_primitives::{
	AesRequest, DecryptableRequest, Identity, Intent, MemberAccount, OmniAccountAuthType,
	ValidationData,
};
use sp_core::{blake2_256, H160, H256};
use sp_runtime::generic::Era;
use std::collections::HashSet;

const THREAD_POOL_SIZE: usize = 480;

type Context<ShieldingKeyRepository, AA, SES, OA, EF, NMR, AKR, AR, SH> =
	Arc<NativeTaskContext<ShieldingKeyRepository, AA, SES, OA, EF, NMR, AKR, AR, SH>>;

pub fn run_native_task_receiver<ShieldingKeyRepository, AA, SES, OA, EF, NMR, AKR, AR, SH>(
	context: Context<ShieldingKeyRepository, AA, SES, OA, EF, NMR, AKR, AR, SH>,
) where
	ShieldingKeyRepository: AccessKey + Send + Sync + 'static,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoEncrypt + ShieldingCryptoDecrypt,
	AA: AuthorApiTrait<H256, H256, TrustedCallSigned, Getter> + Send + Sync + 'static,
	SES: StfEnclaveSigningTrait<TrustedCallSigned> + Send + Sync + 'static,
	OA: EnclaveOnChainOCallApi + EnclaveAttestationOCallApi + EnclaveMetricsOCallApi + 'static,
	EF: CreateExtrinsics + Send + Sync + 'static,
	NMR: AccessNodeMetadata<MetadataType = NodeMetadata> + Send + Sync + 'static,
	AKR: AccessKey<KeyType = Aes256Key> + Send + Sync + 'static,
	AR: AssertionLogicRepository<Id = H160, Item = AssertionRepositoryItem> + Send + Sync + 'static,
	SH: HandleState + Send + Sync + 'static,
	SH::StateT: SgxExternalitiesTrait,
{
	let request_receiver = init_native_task_sender();
	let thread_pool = ThreadPoolBuilder::new()
		.pool_size(THREAD_POOL_SIZE)
		.create()
		.expect("Failed to create thread pool");

	while let Ok(mut req) = request_receiver.recv() {
		thread_pool.spawn_ok({
			let context = context.clone();
			let thread_pool = thread_pool.clone();
			async move {
				let request = &mut req.request;
				let connection_hash = request.using_encoded(|x| H256::from(blake2_256(x)));
				match handle_request(request, context.clone()) {
					Ok((trusted_call, auth_type)) => handle_trusted_call(
						context.clone(),
						trusted_call,
						auth_type,
						connection_hash,
						request.shard,
						thread_pool,
					),
					Err(e) => {
						log::error!("Failed to get trusted call from request: {:?}", e);
						let res: TrustedCallResult = Err(TrustedCallError::InvalidRequest);
						context.author_api.send_rpc_response(connection_hash, res.encode(), false);
					},
				};
			}
		});
	}
	log::warn!("Native task receiver stopped");
}

fn handle_request<ShieldingKeyRepository, AA, SES, OA, EF, NMR, AKR, AR, SH>(
	request: &mut AesRequest,
	context: Context<ShieldingKeyRepository, AA, SES, OA, EF, NMR, AKR, AR, SH>,
) -> Result<(TrustedCall, OmniAccountAuthType), &'static str>
where
	ShieldingKeyRepository: AccessKey + Send + Sync + 'static,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoEncrypt + ShieldingCryptoDecrypt,
	AA: AuthorApiTrait<H256, H256, TrustedCallSigned, Getter> + Send + Sync + 'static,
	SES: StfEnclaveSigningTrait<TrustedCallSigned> + Send + Sync + 'static,
	OA: EnclaveOnChainOCallApi + EnclaveAttestationOCallApi + EnclaveMetricsOCallApi + 'static,
	EF: CreateExtrinsics + Send + Sync + 'static,
	NMR: AccessNodeMetadata<MetadataType = NodeMetadata> + Send + Sync + 'static,
	AKR: AccessKey<KeyType = Aes256Key> + Send + Sync + 'static,
	AR: AssertionLogicRepository<Id = H160, Item = AssertionRepositoryItem> + Send + Sync + 'static,
	SH: HandleState + Send + Sync + 'static,
	SH::StateT: SgxExternalitiesTrait,
{
	let connection_hash = request.using_encoded(|x| H256::from(blake2_256(x)));
	let enclave_shielding_key = match context.shielding_key.retrieve_key() {
		Ok(value) => value,
		Err(e) => {
			let res: TrustedCallResult =
				Err(TrustedCallError::ShieldingKeyRetrievalFailed(format!("{}", e)));
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
			let res: TrustedCallResult = Err(TrustedCallError::RequestPayloadDecodingFailed);
			context.author_api.send_rpc_response(connection_hash, res.encode(), false);
			return Err("Request payload decoding failed")
		},
	};
	let mrenclave = match context.ocall_api.get_mrenclave_of_self() {
		Ok(m) => m.m,
		Err(_) => {
			let res: TrustedCallResult = Err(TrustedCallError::MrEnclaveRetrievalFailed);
			context.author_api.send_rpc_response(connection_hash, res.encode(), false);
			return Err("MrEnclave retrieval failed")
		},
	};

	let authentication_result = match tca.authentication {
		TCAuthentication::Web3(ref signature) => verify_tca_web3_authentication(
			signature,
			&tca.call,
			tca.nonce,
			&mrenclave,
			&request.shard,
		),
		TCAuthentication::Email(ref verification_code) => {
			let sender_identity = tca.call.sender_identity();
			let omni_account =
				match get_omni_account(context.ocall_api.clone(), sender_identity, None) {
					Ok(account) => account,
					_ => sender_identity.to_omni_account(),
				};
			verify_tca_email_authentication(
				sender_identity.hash(),
				&omni_account,
				verification_code,
			)
		},
		TCAuthentication::OAuth2(ref oauth2_data) => {
			let sender_identity = tca.call.sender_identity();
			let omni_account =
				match get_omni_account(context.ocall_api.clone(), sender_identity, None) {
					Ok(account) => account,
					_ => sender_identity.to_omni_account(),
				};
			verify_tca_oauth2_authentication(
				context.data_provider_config.clone(),
				sender_identity.hash(),
				&omni_account,
				oauth2_data,
			)
		},
		TCAuthentication::AuthToken(ref auth_token) => {
			let Ok(header)= context.ocall_api.get_header::<ParachainHeader>() else {
				let res: Result<(), TrustedCallError> = Err(TrustedCallError::ParentchainHeaderRetrievalFailed);
				context.author_api.send_rpc_response(connection_hash, res.encode(), false);
				return Err("Failed to get parachain header")
			};
			let current_block = header.number;
			let sender_identity = tca.call.sender_identity();
			let omni_account =
				match get_omni_account(context.ocall_api.clone(), sender_identity, Some(header)) {
					Ok(account) => account,
					_ => sender_identity.to_omni_account(),
				};
			verify_tca_auth_token_authentication(
				&omni_account,
				current_block,
				auth_token,
				context.data_provider_config.jwt_secret.as_bytes(),
			)
		},
	};

	match authentication_result {
		Ok(_) => Ok((tca.call, tca.authentication.into())),
		Err(e) => {
			log::error!("Failed to verify authentication: {:?}", e);
			let res: TrustedCallResult = Err(TrustedCallError::AuthenticationVerificationFailed);
			context.author_api.send_rpc_response(connection_hash, res.encode(), false);
			Err("Authentication verification failed")
		},
	}
}

type TrustedCallResult = Result<TrustedCallOk<H256>, TrustedCallError>;

fn handle_trusted_call<ShieldingKeyRepository, AA, SES, OA, EF, NMR, AKR, AR, SH>(
	context: Context<ShieldingKeyRepository, AA, SES, OA, EF, NMR, AKR, AR, SH>,
	call: TrustedCall,
	auth_type: OmniAccountAuthType,
	connection_hash: H256,
	shard: H256,
	thread_pool: ThreadPool,
) where
	ShieldingKeyRepository: AccessKey + Send + Sync + 'static,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoEncrypt + ShieldingCryptoDecrypt,
	AA: AuthorApiTrait<H256, H256, TrustedCallSigned, Getter> + Send + Sync + 'static,
	SES: StfEnclaveSigningTrait<TrustedCallSigned> + Send + Sync + 'static,
	OA: EnclaveOnChainOCallApi + EnclaveMetricsOCallApi + 'static,
	EF: CreateExtrinsics + Send + Sync + 'static,
	NMR: AccessNodeMetadata<MetadataType = NodeMetadata> + Send + Sync + 'static,
	AKR: AccessKey<KeyType = Aes256Key> + Send + Sync + 'static,
	AR: AssertionLogicRepository<Id = H160, Item = AssertionRepositoryItem> + Send + Sync + 'static,
	SH: HandleState + Send + Sync + 'static,
	SH::StateT: SgxExternalitiesTrait,
{
	let metadata = match context.node_metadata_repo.get_from_metadata(|m| m.get_metadata().cloned())
	{
		Ok(Some(metadata)) => metadata,
		_ => {
			log::error!("Failed to get node metadata");
			let result: TrustedCallResult = Err(TrustedCallError::MetadataRetrievalFailed(
				"Failed to get node metadata".to_string(),
			));
			context.author_api.send_rpc_response(connection_hash, result.encode(), false);
			return
		},
	};

	let opaque_call = match call {
		TrustedCall::request_intent(who, intent) => match intent {
			Intent::SystemRemark(remark) => OpaqueCall::from_tuple(&compose_call!(
				&metadata,
				"OmniAccount",
				"dispatch_as_signed",
				who.hash(),
				OpaqueCall::from_tuple(&compose_call!(&metadata, "System", "remark", remark)),
				auth_type
			)),
			Intent::TransferNative(transfer) => OpaqueCall::from_tuple(&compose_call!(
				&metadata,
				"OmniAccount",
				"dispatch_as_signed",
				who.hash(),
				OpaqueCall::from_tuple(&compose_call!(
					&metadata,
					"Balances",
					"transfer_allow_death",
					Address::Id(transfer.to),
					Compact(transfer.value)
				)),
				auth_type
			)),
			Intent::CallEthereum(_) | Intent::TransferEthereum(_) | Intent::TransferSolana(_) =>
				OpaqueCall::from_tuple(&compose_call!(
					&metadata,
					"OmniAccount",
					"dispatch_as_omni_account",
					who.hash(),
					OpaqueCall::from_tuple(&compose_call!(
						&metadata,
						"OmniAccount",
						"request_intent",
						intent
					)),
					auth_type
				)),
		},
		TrustedCall::create_account_store(who) => OpaqueCall::from_tuple(&compose_call!(
			&metadata,
			"OmniAccount",
			"create_account_store",
			who
		)),
		TrustedCall::add_account(who, identity, validation_data, public_account) => {
			let omni_account = match get_omni_account(context.ocall_api.clone(), &who, None) {
				Ok(account) => account,
				Err(e) => {
					log::error!("Failed to get omni account: {:?}", e);
					let result: TrustedCallResult = Err(TrustedCallError::UnauthorizedSigner);
					context.author_api.send_rpc_response(connection_hash, result.encode(), false);
					return
				},
			};
			let repository = OmniAccountRepository::new(context.ocall_api.clone());
			let nonce = repository.get_nonce(omni_account).unwrap_or(0);
			let raw_msg = get_expected_raw_message(&who, &identity, nonce);

			let validation_result = match validation_data {
				ValidationData::Web2(validation_data) =>
					if !identity.is_web2() {
						Err(TrustedCallError::InvalidMemberIdentity)
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
							TrustedCallError::ValidationDataVerificationFailed
						})
					},
				ValidationData::Web3(validation_data) =>
					if !identity.is_web3() {
						Err(TrustedCallError::InvalidMemberIdentity)
					} else {
						verify_web3_identity(&identity, &raw_msg, &validation_data).map_err(|e| {
							log::error!("Failed to verify web3 identity: {:?}", e);
							TrustedCallError::ValidationDataVerificationFailed
						})
					},
			};

			if let Err(e) = validation_result {
				let result: TrustedCallResult = Err(e);
				context.author_api.send_rpc_response(connection_hash, result.encode(), false);
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
					let result: TrustedCallResult = Err(e);
					context.author_api.send_rpc_response(connection_hash, result.encode(), false);
					return
				},
			};

			// TOOD: get permissions from the trusted call, this is is temporary hack
			// it will be fixed in P-1241
			let permissions: Option<Vec<u32>> = None; // default permissions

			OpaqueCall::from_tuple(&compose_call!(
				&metadata,
				"OmniAccount",
				"dispatch_as_omni_account",
				who.hash(),
				OpaqueCall::from_tuple(&compose_call!(
					&metadata,
					"OmniAccount",
					"add_account",
					member_account,
					permissions
				)),
				auth_type
			))
		},
		TrustedCall::remove_accounts(who, identities) => OpaqueCall::from_tuple(&compose_call!(
			&metadata,
			"OmniAccount",
			"dispatch_as_omni_account",
			who.hash(),
			OpaqueCall::from_tuple(&compose_call!(
				&metadata,
				"OmniAccount",
				"remove_accounts",
				identities.iter().map(|i| i.hash()).collect::<Vec<H256>>()
			)),
			auth_type
		)),
		TrustedCall::publicize_account(who, identity) => OpaqueCall::from_tuple(&compose_call!(
			&metadata,
			"OmniAccount",
			"dispatch_as_omni_account",
			who.hash(),
			OpaqueCall::from_tuple(&compose_call!(
				&metadata,
				"OmniAccount",
				"publicize_account",
				identity
			)),
			auth_type
		)),
		TrustedCall::request_vc(signer, who, assertion, maybe_key, req_ext_hash) => {
			let result = handle_request_vc(
				context.clone(),
				shard,
				signer,
				who,
				assertion,
				maybe_key,
				req_ext_hash,
			);
			send_vc_response(connection_hash, context, result, 0u8, 1u8, false);
			return
		},
		TrustedCall::request_batch_vc(signer, who, assertions, maybe_key, req_ext_hash) => {
			// use local registry to manage request reponse status
			let vc_req_registry = VcRequestRegistry::new();

			// Detect duplicate assertions
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
			vc_req_registry.add_new_item(connection_hash, assertion_len);

			for (idx, maybe_assertion) in unique_assertions.into_iter().enumerate() {
				if let Some(assertion) = maybe_assertion {
					thread_pool.spawn_ok({
						let context = context.clone();
						let signer = signer.clone();
						let who = who.clone();
						let vc_req_registry = vc_req_registry.clone();

						async move {
							let result = handle_request_vc(
								context.clone(),
								shard,
								signer,
								who,
								assertion,
								maybe_key,
								req_ext_hash,
							);
							match vc_req_registry.update_item(connection_hash) {
								Ok(do_watch) => {
									send_vc_response(
										connection_hash,
										context,
										result,
										idx as u8,
										assertion_len,
										do_watch,
									);
								},
								Err(e) => {
									log::error!(
										"Failed to find connection_hash in registry: {:?}",
										e
									);
									send_vc_response(
										connection_hash,
										context,
										Err(RequestVcErrorDetail::ConnectionHashNotFound(
											e.to_string(),
										)),
										idx as u8,
										assertion_len,
										false,
									);
								},
							}
						}
					});
				} else {
					match vc_req_registry.update_item(connection_hash) {
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
							log::error!("[DuplicateAssertionRequest] Failed to find connection_hash in registry: {:?}", e);
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
			return
		},
		TrustedCall::request_auth_token(who, auth_options) => {
			let omni_account = match get_omni_account(context.ocall_api.clone(), &who, None) {
				Ok(account) => account,
				Err(e) => {
					log::error!("Failed to get omni account: {:?}", e);
					let result: TrustedCallResult = Err(TrustedCallError::UnauthorizedSigner);
					context.author_api.send_rpc_response(connection_hash, result.encode(), false);
					return
				},
			};
			let subject = account_id_to_string_without_prefix(&omni_account);
			let payload = jwt::Payload::new(subject, auth_options);
			let Ok(auth_token) = jwt::create(&payload, context.data_provider_config.jwt_secret.as_bytes()) else {
				log::error!("Failed to create jwt token");
				let result: TrustedCallResult = Err(TrustedCallError::AuthTokenCreationFailed);
				context.author_api.send_rpc_response(connection_hash, result.encode(), false);
				return
			};

			let auth_token_requested_call = OpaqueCall::from_tuple(&compose_call!(
				&metadata,
				"OmniAccount",
				"auth_token_requested",
				omni_account,
				payload.exp
			));
			let params = context.ocall_api.get_header().ok().map(|h: itp_types::Header| {
				ParentchainAdditionalParams::new().era(Era::mortal(5, h.number.into()), h.hash())
			});
			match context
				.extrinsic_factory
				.create_extrinsics(&[auth_token_requested_call], params)
			{
				Ok(extrinsic) => {
					if let Err(e) = context.ocall_api.send_to_parentchain(
						extrinsic,
						&ParentchainId::Litentry,
						None,
					) {
						log::error!("Failed to send extrinsic to parentchain: {:?}", e);
					}
				},
				Err(e) => {
					log::error!("Failed to create extrinsic: {:?}", e);
				},
			};

			let result: TrustedCallResult = Ok(auth_token.into());
			context.author_api.send_rpc_response(connection_hash, result.encode(), false);
			return
		},
		_ => {
			log::warn!("Received unsupported call: {:?}", call);
			let result: TrustedCallResult =
				Err(TrustedCallError::UnexpectedCall(format!("Unexpected call: {:?}", call)));
			context.author_api.send_rpc_response(connection_hash, result.encode(), false);
			return
		},
	};

	let params = context.ocall_api.get_header().ok().map(|h: itp_types::Header| {
		ParentchainAdditionalParams::new().era(Era::mortal(5, h.number.into()), h.hash())
	});

	let extrinsic = match context.extrinsic_factory.create_extrinsics(&[opaque_call], params) {
		Ok(extrinsic) => extrinsic,
		Err(e) => {
			log::error!("Failed to create extrinsic: {:?}", e);
			let result: TrustedCallResult =
				Err(TrustedCallError::ExtrinsicConstructionFailed(e.to_string()));
			context.author_api.send_rpc_response(connection_hash, result.encode(), false);
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
				let result: TrustedCallResult = Ok(report.into());
				context.author_api.send_rpc_response(connection_hash, result.encode(), false);
			} else {
				log::error!("Failed to get extrinsic report");
				let result: TrustedCallResult = Err(TrustedCallError::ExtrinsicSendingFailed(
					"Failed to get extrinsic report".to_string(),
				));
				context.author_api.send_rpc_response(connection_hash, result.encode(), false);
			},
		Err(e) => {
			log::error!("Failed to send extrinsic to parentchain: {:?}", e);
			let result: TrustedCallResult =
				Err(TrustedCallError::ExtrinsicSendingFailed(e.to_string()));
			context.author_api.send_rpc_response(connection_hash, result.encode(), false);
		},
	}
}

fn create_member_account<Aes256KeyRepository>(
	aes256_key_repository: Arc<Aes256KeyRepository>,
	identity: &Identity,
	is_public: bool,
) -> Result<MemberAccount, TrustedCallError>
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
			Err(TrustedCallError::MissingAesKey)
		},
	}?;

	Ok(MemberAccount::Private(
		aes_encrypt_default(&aes_key, &identity.encode()).encode(),
		identity.hash(),
	))
}

fn get_omni_account<OCallApi: EnclaveOnChainOCallApi>(
	ocall_api: Arc<OCallApi>,
	who: &Identity,
	header: Option<ParachainHeader>,
) -> Result<AccountId, &'static str> {
	let omni_account = match OmniAccountStore::get_omni_account(who.hash()) {
		Ok(Some(account)) => account,
		_ => {
			log::warn!("Failed to get omni account from the in-memory store");
			let header = match header {
				Some(h) => h,
				None => ocall_api.get_header().map_err(|_| {
					log::error!("Failed to get header");
					"Failed to get header"
				})?,
			};
			OmniAccountRepository::new(ocall_api)
				.with_header(header)
				.get_account_by_member_hash(who.hash())
				.map_err(|_| {
					log::error!("Failed to get account by member hash");
					"Failed to get account by member hash"
				})?
				.ok_or_else(|| {
					log::error!("Omni account not found for the given member identity");
					"Omni account not found for the given member identity"
				})?
		},
	};
	Ok(omni_account)
}
