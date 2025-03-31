/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG

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

//! Request state keys from a fellow validateer.

use crate::{
	enclave::tls_ra::enclave_request_state_provisioning,
	error::{Error, ServiceResult as Result},
};
use futures::executor;
use itc_rpc_client::direct_client::{DirectApi, DirectClient as DirectWorkerApi};
use itp_enclave_api::{
	enclave_base::EnclaveBase,
	remote_attestation::{RemoteAttestation, TlsRemoteAttestation},
};
use itp_node_api::api_client::PalletTeebagApi;
use itp_settings::worker_mode::{ProvideWorkerMode, WorkerMode};
use itp_types::{ShardIdentifier, WorkerType};
use log::info;
use sgx_types::sgx_quote_sign_type_t;
use sp_runtime::MultiSigner;
use std::string::String;

pub(crate) fn sync_state<
	E: TlsRemoteAttestation + EnclaveBase + RemoteAttestation,
	NodeApi: PalletTeebagApi,
	WorkerModeProvider: ProvideWorkerMode,
>(
	node_api: &NodeApi,
	shard: &ShardIdentifier,
	enclave_api: &E,
	skip_ra: bool,
) {
	let provider_url = match WorkerModeProvider::worker_mode() {
		WorkerMode::Sidechain | WorkerMode::OffChainWorker =>
		// TODO(Litentry P-629): maybe implement `get_enclave_url_of_last_active`
		{
			executor::block_on(get_enclave_url_of_primary_worker_for_shard(node_api, shard))
				.expect("Author of primary worker for shard could not be found")
		},
	};

	println!("Requesting state provisioning from worker at {}", &provider_url);

	enclave_request_state_provisioning(
		enclave_api,
		sgx_quote_sign_type_t::SGX_UNLINKABLE_SIGNATURE,
		&provider_url,
		shard,
		skip_ra,
	)
	.unwrap();
	println!("[+] State provisioning successfully performed.");
}

/// Returns the url of the primary worker for the given shard
async fn get_enclave_url_of_primary_worker_for_shard<NodeApi: PalletTeebagApi>(
	node_api: &NodeApi,
	shard: &ShardIdentifier,
) -> Result<String> {
	let enclave = node_api
		.primary_enclave_for_shard(WorkerType::Identity, shard, None)?
		.ok_or_else(|| Error::NoWorkerForShardFound(*shard))?;
	let worker_api_direct =
		DirectWorkerApi::new(String::from_utf8_lossy(enclave.url.as_slice()).to_string());
	Ok(worker_api_direct.get_mu_ra_url()?)
}

/// Returns the url of the first Enclave that matches our own MRENCLAVE.
///
/// This should be run before we register ourselves as enclave, to ensure we don't get our own url.
async fn get_enclave_url_of_first_registered<NodeApi: PalletTeebagApi, EnclaveApi: EnclaveBase>(
	node_api: &NodeApi,
	enclave_api: &EnclaveApi,
) -> Result<String> {
	let self_mrenclave = enclave_api.get_fingerprint()?;
	let first_enclave = node_api
		.all_enclaves(WorkerType::Identity, None)?
		.into_iter()
		.find(|e| e.mrenclave == self_mrenclave.to_fixed_bytes())
		.ok_or(Error::NoPeerWorkerFound)?;
	let worker_api_direct =
		DirectWorkerApi::new(String::from_utf8_lossy(first_enclave.url.as_slice()).to_string());
	Ok(worker_api_direct.get_mu_ra_url()?)
}
