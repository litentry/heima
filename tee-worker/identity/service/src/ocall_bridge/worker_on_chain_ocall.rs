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

use crate::ocall_bridge::bridge_api::{OCallBridgeError, OCallBridgeResult, WorkerOnChainBridge};
use codec::{Decode, Encode};
use itp_enclave_api::enclave_base::EnclaveBase;
use itp_node_api::{
	api_client::{AccountApi, ExtrinsicReport, ParentchainApi, XtStatus},
	node_api_factory::CreateNodeApi,
};
use itp_types::{
	parentchain::{AccountId, ParentchainId},
	WorkerRequest, WorkerResponse,
};
use log::*;
use sp_core::H256;
use sp_runtime::OpaqueExtrinsic;
use std::{sync::Arc, thread, vec::Vec};
use substrate_api_client::{
	ac_primitives::serde_impls::StorageKey, GetAccountInformation, GetChainInfo, GetStorage,
	SubmitAndWatch, SubmitExtrinsic,
};

#[cfg(feature = "link-binary")]
use crate::main_impl::enclave_account;

pub struct WorkerOnChainOCall<E, F> {
	enclave_api: Arc<E>,
	integritee_api_factory: Arc<F>,
	target_a_parentchain_api_factory: Option<Arc<F>>,
	target_b_parentchain_api_factory: Option<Arc<F>>,
}

impl<E, F> WorkerOnChainOCall<E, F> {
	pub fn new(
		enclave_api: Arc<E>,
		integritee_api_factory: Arc<F>,
		target_a_parentchain_api_factory: Option<Arc<F>>,
		target_b_parentchain_api_factory: Option<Arc<F>>,
	) -> Self {
		WorkerOnChainOCall {
			enclave_api,
			integritee_api_factory,
			target_a_parentchain_api_factory,
			target_b_parentchain_api_factory,
		}
	}
}

impl<E: EnclaveBase, F: CreateNodeApi> WorkerOnChainOCall<E, F> {
	pub fn create_api(&self, parentchain_id: ParentchainId) -> OCallBridgeResult<ParentchainApi> {
		Ok(match parentchain_id {
			ParentchainId::Litentry => self.integritee_api_factory.create_api()?,
			ParentchainId::TargetA => self
				.target_a_parentchain_api_factory
				.as_ref()
				.ok_or(OCallBridgeError::TargetAParentchainNotInitialized)
				.and_then(|f| f.create_api().map_err(Into::into))?,
			ParentchainId::TargetB => self
				.target_b_parentchain_api_factory
				.as_ref()
				.ok_or(OCallBridgeError::TargetBParentchainNotInitialized)
				.and_then(|f| f.create_api().map_err(Into::into))?,
		})
	}
}

impl<E, F> WorkerOnChainBridge for WorkerOnChainOCall<E, F>
where
	E: EnclaveBase,
	F: CreateNodeApi,
{
	fn worker_request(
		&self,
		request: Vec<u8>,
		parentchain_id: Vec<u8>,
	) -> OCallBridgeResult<Vec<u8>> {
		trace!("    Entering ocall_worker_request");

		let requests: Vec<WorkerRequest> = Decode::decode(&mut request.as_slice())?;
		if requests.is_empty() {
			debug!("requests is empty, returning empty vector");
			return Ok(Vec::<u8>::new().encode())
		}

		let parentchain_id = ParentchainId::decode(&mut parentchain_id.as_slice())?;

		let api = self.create_api(parentchain_id)?;

		let resp: Vec<WorkerResponse<Vec<u8>>> = requests
			.into_iter()
			.map(|req| match req {
				WorkerRequest::ChainStorage(key, hash) => WorkerResponse::ChainStorage(
					key.clone(),
					api.get_opaque_storage_by_key(StorageKey(key.clone()), hash).unwrap(),
					api.get_storage_proof_by_keys(vec![StorageKey(key)], hash).unwrap().map(
						|read_proof| read_proof.proof.into_iter().map(|bytes| bytes.0).collect(),
					),
				),
				WorkerRequest::ChainStorageKeys(key, hash) => {
					let keys: Vec<Vec<u8>> = match api.get_keys(StorageKey(key), hash) {
						Ok(Some(keys)) => keys.iter().map(String::encode).collect(),
						_ => Default::default(),
					};
					WorkerResponse::ChainStorageKeys(keys)
				},
				WorkerRequest::ChainStorageKeysPaged(prefix, count, start_key, hash) => {
					let keys: Vec<Vec<u8>> = match api.get_storage_keys_paged(
						Some(StorageKey(prefix)),
						count,
						start_key.map(StorageKey),
						hash,
					) {
						Ok(keys) => keys.iter().map(|k| k.0.to_vec()).collect(),
						_ => Default::default(),
					};
					WorkerResponse::ChainStorageKeys(keys)
				},
				WorkerRequest::ChainHeader(block_hash) => {
					let header = match api.get_header(block_hash) {
						Ok(Some(header)) => Some(header.encode()),
						_ => None,
					};
					WorkerResponse::ChainHeader(header)
				},
				WorkerRequest::ChainAccountNonce(encoded_account_id) => {
					let maybe_nonce = match AccountId::decode(&mut encoded_account_id.as_slice()) {
						Ok(account_id) => api.get_account_nonce(&account_id).ok(),
						_ => {
							error!("[ChainAccountNonce] account_id could not be decoded");
							None
						},
					};
					WorkerResponse::ChainAccountNonce(maybe_nonce)
				},
			})
			.collect();

		let encoded_response: Vec<u8> = resp.encode();

		Ok(encoded_response)
	}

	fn send_to_parentchain(
		&self,
		extrinsics_encoded: Vec<u8>,
		parentchain_id: Vec<u8>,
		watch_until: Vec<u8>,
	) -> OCallBridgeResult<Vec<u8>> {
		let maybe_watch_until: Option<XtStatus> = Decode::decode(&mut watch_until.as_slice())
			.map_err(|_| {
				OCallBridgeError::SendExtrinsicsToParentchain(
					"Could not decode watch_until".to_string(),
				)
			})?;
		let extrinsics: Vec<OpaqueExtrinsic> = Decode::decode(&mut extrinsics_encoded.as_slice())
			.map_err(|_| {
			OCallBridgeError::SendExtrinsicsToParentchain("Could not decode extrinsics".to_string())
		})?;
		let mut extrinsic_reports: Vec<ExtrinsicReport<H256>> = Vec::new();
		let parentchain_id = ParentchainId::decode(&mut parentchain_id.as_slice())?;
		debug!(
			"Enclave wants to send {} extrinsics to parentchain: {:?}. watch_until: {:?}",
			extrinsics.len(),
			parentchain_id,
			maybe_watch_until
		);
		let api = self.create_api(parentchain_id)?;
		let mut send_extrinsic_failed = false;

		for call in extrinsics.into_iter() {
			if let Some(xt_status) = maybe_watch_until {
				match api.submit_and_watch_opaque_extrinsic_until(
					&call.encode().into(),
					xt_status.into(),
				) {
					Ok(report) => extrinsic_reports.push(report.into()),
					Err(e) => {
						error!(
							"Could not send extrinsic to {:?}: {:?}, error: {:?}",
							parentchain_id,
							serde_json::to_string(&call),
							e
						);
						send_extrinsic_failed = true;
					},
				}
			} else if let Err(e) = api.submit_opaque_extrinsic(&call.encode().into()) {
				error!(
					"Could not send extrinsic to {:?}: {:?}, error: {:?}",
					parentchain_id,
					serde_json::to_string(&call),
					e
				);
				send_extrinsic_failed = true;
			}
		}

		// Try to reset nonce, see
		// - https://github.com/litentry/heima/issues/1036
		// - https://github.com/integritee-network/worker/issues/970
		// It has to be done in a separate thread as nested ECALL/OCALL is disallowed
		//
		// This workaround is likely to cause duplicate nonce or "transaction outdated" error in the parentchain
		// tx pool, because the retrieved on-chain nonce doesn't count the pending tx, meanwhile the extrinsic factory
		// keeps composing new extrinsics. So the nonce used for composing the new extrinsics can collide with the nonce
		// in the already submitted tx. As a result, a few txs could be dropped during the parentchain tx pool processing.
		// Not to mention the thread dispatch delay and network delay (query on-chain nonce).
		//
		// However, we still consider it better than the current situation, where the nonce never gets rectified and
		// all following extrinsics will be blocked. Moreover, the txs sent to the parentchain are mostly
		// "notification extrinsics" and don't cause chain state change, therefore we deem it less harmful to drop them.
		// The worst case is some action is wrongly intepreted as "failed" (because F/E doesn't get the event in time)
		// while it actually succeeds. In that case, the user needs to re-do the extrinsic, which is suboptimal,
		// but still better than the chain stalling.
		//
		// To have a better synchronisation handling we probably need a sending queue in extrinsic factory that
		// can be paused on demand (or wait for the nonce synchronisation).
		//
		// Another small thing that can be improved is to use rpc.system.accountNextIndex instead of system.account.nonce
		// see https://polkadot.js.org/docs/api/cookbook/tx/#how-do-i-take-the-pending-tx-pool-into-account-in-my-nonce
		#[cfg(feature = "link-binary")]
		if send_extrinsic_failed {
			// drop &self lifetime
			let node_api_factory_cloned = self.integritee_api_factory.clone();
			let enclave_cloned = self.enclave_api.clone();
			thread::spawn(move || {
				let api = node_api_factory_cloned.create_api().unwrap();
				let enclave_account = enclave_account(enclave_cloned.as_ref());
				warn!("send_extrinsic failed, try to reset nonce ...");
				match api.get_account_next_index(&enclave_account) {
					Ok(nonce) => {
						warn!("query on-chain nonce OK, reset nonce to: {}", nonce);
						if let Err(e) = enclave_cloned.set_nonce(nonce, ParentchainId::Litentry) {
							warn!("failed to reset nonce due to: {:?}", e);
						}
					},
					Err(e) => warn!("query on-chain nonce failed: {:?}", e),
				}
			});
		}

		Ok(extrinsic_reports.encode())
	}
}

#[cfg(test)]
mod tests {

	use super::*;
	use crate::tests::mocks::enclave_api_mock::EnclaveMock;
	use itp_node_api::{
		api_client::ParentchainApi,
		node_api_factory::{CreateNodeApi, Result as NodeApiResult},
	};
	use mockall::mock;

	#[test]
	fn given_empty_worker_request_when_submitting_then_return_empty_response() {
		mock! {
			NodeApiFactory {}
			impl CreateNodeApi for NodeApiFactory {
				fn create_api(&self) -> NodeApiResult<ParentchainApi>;
			}
		}

		let mock_enclave = Arc::new(EnclaveMock {});
		let mock_node_api_factory = Arc::new(MockNodeApiFactory::new());

		let on_chain_ocall =
			WorkerOnChainOCall::new(mock_enclave, mock_node_api_factory, None, None);

		let response = on_chain_ocall
			.worker_request(Vec::<u8>::new().encode(), ParentchainId::Litentry.encode())
			.unwrap();

		assert!(!response.is_empty()); // the encoded empty vector is not empty
		let decoded_response: Vec<u8> = Decode::decode(&mut response.as_slice()).unwrap();
		assert!(decoded_response.is_empty()); // decode the response, and we get an empty vector again
	}
}
