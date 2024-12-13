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

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

// re-export module to properly feature gate sgx and regular std environment
#[cfg(all(not(feature = "std"), feature = "sgx"))]
pub mod sgx_reexport_prelude {
	pub use futures_sgx as futures;
	pub use thiserror_sgx as thiserror;
}

#[cfg(all(not(feature = "std"), feature = "sgx"))]
use crate::sgx_reexport_prelude::*;

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

pub mod handler;

use codec::Encode;
use frame_support::sp_tracing::warn;
use futures::{executor, executor::ThreadPoolBuilder};
use handler::{identity_verification::IdentityVerificationHandler, TaskHandler};
use ita_sgx_runtime::Hash;
use ita_stf::{Getter, TrustedCall, TrustedCallSigned};
use itp_enclave_metrics::EnclaveMetric;
use itp_ocall_api::{EnclaveMetricsOCallApi, EnclaveOnChainOCallApi};
use itp_sgx_crypto::{key_repository::AccessKey, ShieldingCryptoEncrypt};
use itp_sgx_externalities::SgxExternalitiesTrait;
use itp_stf_executor::traits::StfEnclaveSigning;
use itp_stf_primitives::types::TrustedOperation;
use itp_stf_state_handler::handle_state::HandleState;
use itp_top_pool_author::traits::AuthorApi;
use itp_types::{RsaRequest, ShardIdentifier, H256};
use lc_data_providers::DataProviderConfig;
use lc_dynamic_assertion::AssertionLogicRepository;
use lc_evm_dynamic_assertions::AssertionRepositoryItem;
use lc_stf_task_sender::init_stf_task_sender_storage;
use litentry_primitives::RequestType;
use log::*;
use sp_core::{ed25519::Pair as Ed25519Pair, H160};
use std::{
	boxed::Box,
	format,
	string::{String, ToString},
	sync::{mpsc::channel, Arc},
	thread,
	time::Instant,
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod test;

#[derive(Debug, thiserror::Error, Clone)]
pub enum Error {
	#[error("Request error: {0}")]
	RequestError(String),

	#[error("Assertion error: {0}")]
	AssertionError(String),

	#[error("Other error: {0}")]
	OtherError(String),
}

#[allow(dead_code)]
pub struct StfTaskContext<
	ShieldingKeyRepository,
	A: AuthorApi<Hash, Hash, TrustedCallSigned, Getter>,
	S: StfEnclaveSigning<TrustedCallSigned>,
	H: HandleState,
	O: EnclaveOnChainOCallApi + EnclaveMetricsOCallApi,
	AR: AssertionLogicRepository<Id = H160, Item = AssertionRepositoryItem>,
> where
	ShieldingKeyRepository: AccessKey,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoEncrypt + 'static,
{
	pub shielding_key: Arc<ShieldingKeyRepository>,
	pub author_api: Arc<A>,
	pub enclave_signer: Arc<S>,
	pub enclave_account: Arc<Ed25519Pair>,
	pub state_handler: Arc<H>,
	pub ocall_api: Arc<O>,
	pub data_provider_config: Arc<DataProviderConfig>,
	pub assertion_repository: Arc<AR>,
}

impl<
		ShieldingKeyRepository,
		A: AuthorApi<Hash, Hash, TrustedCallSigned, Getter>,
		S: StfEnclaveSigning<TrustedCallSigned>,
		H: HandleState,
		O: EnclaveOnChainOCallApi + EnclaveMetricsOCallApi,
		AR: AssertionLogicRepository<Id = H160, Item = AssertionRepositoryItem>,
	> StfTaskContext<ShieldingKeyRepository, A, S, H, O, AR>
where
	ShieldingKeyRepository: AccessKey,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoEncrypt + 'static,
	H::StateT: SgxExternalitiesTrait,
{
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		shielding_key: Arc<ShieldingKeyRepository>,
		author_api: Arc<A>,
		enclave_signer: Arc<S>,
		enclave_account: Arc<Ed25519Pair>,
		state_handler: Arc<H>,
		ocall_api: Arc<O>,
		data_provider_config: Arc<DataProviderConfig>,
		assertion_repository: Arc<AR>,
	) -> Self {
		Self {
			shielding_key,
			author_api,
			enclave_signer,
			enclave_account,
			state_handler,
			ocall_api,
			data_provider_config,
			assertion_repository,
		}
	}

	pub fn submit_trusted_call(
		&self,
		shard: &ShardIdentifier,
		maybe_old_top_hash: Option<H256>,
		trusted_call: &TrustedCall,
	) -> Result<(), Error> {
		let signed_trusted_call = self
			.enclave_signer
			.sign_call_with_self(trusted_call, shard)
			.map_err(|e| Error::OtherError(format!("{:?}", e)))?;

		let top: TrustedOperation<TrustedCallSigned, Getter> =
			TrustedOperation::direct_call(signed_trusted_call);

		// find out if we have any trusted operation which has the same hash in the pool already.
		// The hash can be used to de-duplicate a trusted operation for a certain request, as the
		// `trusted_call` in this fn always contains the req_ext_hash, which is unique for each request.
		if self
			.author_api
			.get_pending_trusted_calls_for(
				*shard,
				&trusted_call.sender_identity().to_native_account().ok_or_else(|| {
					Error::OtherError(format!(
						"Not a valid account: {:?}",
						trusted_call.sender_identity()
					))
				})?,
			)
			.into_iter()
			.any(|t| t.hash() == top.hash())
		{
			// skip the submission if some top with the same hash already exists, return Ok(())
			warn!("Skip submit_trusted_call because top with the same hash exists");
			return Ok(())
		}

		// swap the hash in the rpc connection registry to make sure furthre RPC responses go to
		// the right channel
		if let Some(old_hash) = maybe_old_top_hash {
			self.author_api.swap_rpc_connection_hash(old_hash, top.hash());
		}

		let shielding_key = self
			.shielding_key
			.retrieve_key()
			.map_err(|e| Error::OtherError(format!("{:?}", e)))?;

		let encrypted_trusted_call = shielding_key
			.encrypt(&top.encode())
			.map_err(|e| Error::OtherError(format!("{:?}", e)))?;

		debug!(
			"submit encrypted trusted call: {} bytes, original encoded top: {} bytes",
			encrypted_trusted_call.len(),
			top.encode().len()
		);
		executor::block_on(self.author_api.watch_and_broadcast_top(
			RsaRequest::new(*shard, encrypted_trusted_call),
			"author_submitAndWatchBroadcastedRsaRequest".to_string(),
		))
		.map_err(|e| {
			Error::OtherError(format!("error submitting trusted call to top pool: {:?}", e))
		})?;

		Ok(())
	}
}

// lifetime elision: StfTaskContext is guaranteed to outlive the fn
pub fn run_stf_task_receiver<ShieldingKeyRepository, A, S, H, O, AR>(
	context: Arc<StfTaskContext<ShieldingKeyRepository, A, S, H, O, AR>>,
) -> Result<(), Error>
where
	ShieldingKeyRepository: AccessKey + Sync + Send + 'static,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoEncrypt,
	A: AuthorApi<Hash, Hash, TrustedCallSigned, Getter> + Send + Sync + 'static,
	S: StfEnclaveSigning<TrustedCallSigned> + Send + Sync + 'static,
	H: HandleState + Send + Sync + 'static,
	H::StateT: SgxExternalitiesTrait,
	O: EnclaveOnChainOCallApi + EnclaveMetricsOCallApi + 'static,
	AR: AssertionLogicRepository<Id = H160, Item = AssertionRepositoryItem> + Send + Sync + 'static,
{
	let stf_task_receiver = init_stf_task_sender_storage()
		.map_err(|e| Error::OtherError(format!("read storage error:{:?}", e)))?;
	let n_workers = 4;
	let pool = ThreadPoolBuilder::new().pool_size(n_workers).create().unwrap();

	let (sender, receiver) = channel::<(ShardIdentifier, H256, TrustedCall)>();

	// Spawn thread to handle received tasks, to serialize the nonce increase even if multiple threads
	// are submitting trusted calls simultaneously
	let context_cloned = context.clone();
	thread::spawn(move || loop {
		if let Ok((shard, hash, call)) = receiver.recv() {
			if let Err(e) = context_cloned.submit_trusted_call(&shard, Some(hash), &call) {
				error!("Submit Trusted Call failed: {:?}", e);
			}
		}
	});

	while let Ok(req) = stf_task_receiver.recv() {
		let context_pool = context.clone();
		let sender_pool = sender.clone();

		pool.spawn_ok(async move {
			let start_time = Instant::now();

			match &req {
				RequestType::IdentityVerification(req) =>
					IdentityVerificationHandler { req: req.clone(), context: context_pool.clone() }
						.start(sender_pool),
			}

			if let Err(e) =
				context_pool.ocall_api.update_metric(EnclaveMetric::StfTaskExecutionTime(
					Box::new(req),
					start_time.elapsed().as_secs_f64(),
				)) {
				warn!("Failed to update metric for stf execution: {:?}", e);
			}
		});
	}
	warn!("stf_task_receiver loop terminated");
	Ok(())
}
