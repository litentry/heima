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

#[cfg(all(not(feature = "std"), feature = "sgx"))]
use crate::sgx_reexport_prelude::*;
use core::fmt::Debug;

use crate::{
	client_error::Error as ClientError,
	error::{Error as StateRpcError, Result},
	top_filter::Filter,
	traits::{AuthorApi, OnBlockImported},
};
use codec::{Decode, Encode};
use itp_enclave_metrics::EnclaveMetric;
use itp_ocall_api::EnclaveMetricsOCallApi;
use itp_sgx_crypto::{key_repository::AccessKey, ShieldingCryptoDecrypt};
use itp_stf_primitives::{
	traits::{PoolTransactionValidation, TrustedCallVerification},
	types::{AccountId, Hash, TrustedOperation as StfTrustedOperation, TrustedOperationOrHash},
};
use itp_stf_state_handler::query_shard_state::QueryShardState;
use itp_top_pool::{
	error::{Error as PoolError, IntoPoolError},
	primitives::{
		BlockHash, InPoolOperation, PoolFuture, PoolStatus, TrustedOperationPool,
		TrustedOperationSource, TxHash,
	},
};
use itp_types::{BlockHash as SidechainBlockHash, DecryptableRequest, ShardIdentifier};
use itp_utils::hex::ToHexPrefixed;
use jsonrpc_core::{
	futures::future::{ready, TryFutureExt},
	Error as RpcError,
};
use litentry_primitives::BroadcastedRequest;
use log::*;
use sp_runtime::generic;
use std::{
	boxed::Box,
	string::String,
	sync::{mpsc::SyncSender, Arc},
	vec::Vec,
};

/// Define type of TOP filter that is used in the Author
#[cfg(feature = "sidechain")]
pub type AuthorTopFilter<TCS, G> = crate::top_filter::CallsOnlyFilter<TCS, G>;
#[cfg(feature = "sidechain")]
pub type BroadcastedTopFilter<TCS, G> = crate::top_filter::DirectCallsOnlyFilter<TCS, G>;

#[cfg(feature = "offchain-worker")]
pub type AuthorTopFilter<TCS, G> = crate::top_filter::IndirectCallsOnlyFilter<TCS, G>;
#[cfg(feature = "offchain-worker")]
pub type BroadcastedTopFilter<TCS, G> = crate::top_filter::DenyAllFilter<TCS, G>;

#[cfg(not(any(feature = "sidechain", feature = "offchain-worker")))]
pub type AuthorTopFilter<TCS, G> = crate::top_filter::CallsOnlyFilter<TCS, G>;

#[cfg(not(any(feature = "sidechain", feature = "offchain-worker")))]
pub type BroadcastedTopFilter<TCS, G> = crate::top_filter::DenyAllFilter<TCS, G>;

/// Currently we treat all RPC operations as externals.
///
/// Possibly in the future we could allow opt-in for special treatment
/// of such operations, so that the block authors can inject
/// some unique operations via RPC and have them included in the pool.
const TX_SOURCE: TrustedOperationSource = TrustedOperationSource::External;

// remove duplication of this type definiton ?
pub type RequestIdWithParamsAndMethod = Option<(Hash, Vec<String>)>;

/// Authoring API for RPC calls
///
///
pub struct Author<
	TopPool,
	TopFilter,
	BroadcastedTopFilter,
	StateFacade,
	ShieldingKeyRepository,
	OCallApi,
	TCS,
	G,
> where
	TopPool: TrustedOperationPool<StfTrustedOperation<TCS, G>> + Sync + Send + 'static,
	TopFilter: Filter<Value = StfTrustedOperation<TCS, G>>,
	BroadcastedTopFilter: Filter<Value = StfTrustedOperation<TCS, G>>,
	StateFacade: QueryShardState,
	ShieldingKeyRepository: AccessKey,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoDecrypt + 'static,
	TCS: PartialEq + Encode + Clone + Debug + Send + Sync,
	G: PartialEq + Encode + Clone + PoolTransactionValidation + Debug + Send + Sync,
{
	top_pool: Arc<TopPool>,
	top_filter: TopFilter,
	broadcasted_top_filter: BroadcastedTopFilter,
	state_facade: Arc<StateFacade>,
	shielding_key_repo: Arc<ShieldingKeyRepository>,
	ocall_api: Arc<OCallApi>,
	request_sink: Arc<SyncSender<BroadcastedRequest>>,
}

impl<
		TopPool,
		TopFilter,
		BroadcastedTopFilter,
		StateFacade,
		ShieldingKeyRepository,
		OCallApi,
		TCS,
		G,
	>
	Author<
		TopPool,
		TopFilter,
		BroadcastedTopFilter,
		StateFacade,
		ShieldingKeyRepository,
		OCallApi,
		TCS,
		G,
	> where
	TopPool: TrustedOperationPool<StfTrustedOperation<TCS, G>> + Sync + Send + 'static,
	TopFilter: Filter<Value = StfTrustedOperation<TCS, G>>,
	BroadcastedTopFilter: Filter<Value = StfTrustedOperation<TCS, G>>,
	StateFacade: QueryShardState,
	ShieldingKeyRepository: AccessKey,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoDecrypt + 'static,
	OCallApi: EnclaveMetricsOCallApi + Send + Sync + 'static,
	TCS: PartialEq + Encode + Clone + Debug + Send + Sync,
	G: PartialEq + Encode + Clone + PoolTransactionValidation + Debug + Send + Sync,
{
	/// Create new instance of Authoring API.
	pub fn new(
		top_pool: Arc<TopPool>,
		top_filter: TopFilter,
		broadcasted_top_filter: BroadcastedTopFilter,
		state_facade: Arc<StateFacade>,
		encryption_key: Arc<ShieldingKeyRepository>,
		ocall_api: Arc<OCallApi>,
		request_sink: Arc<SyncSender<BroadcastedRequest>>,
	) -> Self {
		Author {
			top_pool,
			top_filter,
			broadcasted_top_filter,
			state_facade,
			shielding_key_repo: encryption_key,
			ocall_api,
			request_sink,
		}
	}
}

enum TopSubmissionMode {
	Submit,
	SubmitWatch,
	SubmitWatchAndBroadcast(String),
}

impl<
		TopPool,
		TopFilter,
		BroadcastedTopFilter,
		StateFacade,
		ShieldingKeyRepository,
		OCallApi,
		TCS,
		G,
	>
	Author<
		TopPool,
		TopFilter,
		BroadcastedTopFilter,
		StateFacade,
		ShieldingKeyRepository,
		OCallApi,
		TCS,
		G,
	> where
	TopPool: TrustedOperationPool<StfTrustedOperation<TCS, G>> + Sync + Send + 'static,
	TopFilter: Filter<Value = StfTrustedOperation<TCS, G>>,
	BroadcastedTopFilter: Filter<Value = StfTrustedOperation<TCS, G>>,
	StateFacade: QueryShardState,
	ShieldingKeyRepository: AccessKey,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoDecrypt + 'static,
	OCallApi: EnclaveMetricsOCallApi + Send + Sync + 'static,
	TCS: PartialEq
		+ Encode
		+ Decode
		+ Clone
		+ Debug
		+ Send
		+ Sync
		+ TrustedCallVerification
		+ 'static,
	G: PartialEq
		+ Encode
		+ Decode
		+ Clone
		+ PoolTransactionValidation
		+ Debug
		+ Send
		+ Sync
		+ 'static,
{
	fn process_top<R: DecryptableRequest + Encode>(
		&self,
		mut request: R,
		submission_mode: TopSubmissionMode,
	) -> PoolFuture<TxHash, RpcError> {
		let shard = request.shard();

		//we need to save it here as other function may eventually mutate it
		let request_to_broadcast = request.to_hex();
		// check if shard exists
		match self.state_facade.shard_exists(&shard) {
			Err(_) => return Box::pin(ready(Err(ClientError::InvalidShard.into()))),
			Ok(shard_exists) => {
				if !shard_exists {
					return Box::pin(ready(Err(ClientError::InvalidShard.into())));
				}
			},
		};

		// decrypt call
		let shielding_key = match self.shielding_key_repo.retrieve_key() {
			Ok(k) => k,
			Err(_) => return Box::pin(ready(Err(ClientError::BadFormatDecipher.into()))),
		};
		let request_vec = match request.decrypt(Box::new(shielding_key)) {
			Ok(req) => req,
			Err(_) => return Box::pin(ready(Err(ClientError::BadFormatDecipher.into()))),
		};
		// decode call
		let trusted_operation =
			match StfTrustedOperation::<TCS, G>::decode(&mut request_vec.as_slice()) {
				Ok(op) => op,
				Err(_) => return Box::pin(ready(Err(ClientError::BadFormat.into()))),
			};

		trace!("decrypted indirect invocation: {:?}", trusted_operation);

		// apply top filter - return error if this specific type of trusted operation
		// is not allowed by the filter
		if !self.top_filter.filter(&trusted_operation) {
			warn!("unsupported operation");
			return Box::pin(ready(Err(ClientError::UnsupportedOperation.into())));
		}

		//let best_block_hash = self.client.info().best_hash;
		// dummy block hash
		let best_block_hash = Default::default();

		// Update metric
		if let Err(e) = self.ocall_api.update_metric(EnclaveMetric::TopPoolSizeIncrement) {
			warn!("Failed to update metric for top pool size: {:?}", e);
		}

		if let Some(trusted_call_signed) = trusted_operation.to_call() {
			info!(
				"Submitting trusted call, hash: {:?}, call: {:?}",
				self.hash_of(&trusted_operation),
				trusted_call_signed
			);
		} else if let StfTrustedOperation::<TCS, G>::get(ref getter) = trusted_operation {
			info!(
				"Submitting trusted getter, hash {:?}, call: {:?}",
				self.hash_of(&trusted_operation),
				getter,
			);
		}

		match submission_mode {
			TopSubmissionMode::Submit => Box::pin(
				self.top_pool
					.submit_one(
						&generic::BlockId::hash(best_block_hash),
						TX_SOURCE,
						trusted_operation,
						shard,
					)
					.map_err(map_top_error::<TopPool, TCS, G>),
			),

			TopSubmissionMode::SubmitWatch => Box::pin(
				self.top_pool
					.submit_and_watch(
						&generic::BlockId::hash(best_block_hash),
						TX_SOURCE,
						trusted_operation,
						shard,
					)
					.map_err(map_top_error::<TopPool, TCS, G>),
			),

			TopSubmissionMode::SubmitWatchAndBroadcast(s) => {
				let hash = self.hash_of(&trusted_operation).to_hex();
				let can_be_broadcasted = self.broadcasted_top_filter.filter(&trusted_operation);
				let result = Box::pin(
					self.top_pool
						.submit_and_watch(
							&generic::BlockId::hash(best_block_hash),
							TX_SOURCE,
							trusted_operation,
							shard,
						)
						.map_err(map_top_error::<TopPool, TCS, G>),
				);
				// broadcast only if filter allowed
				if can_be_broadcasted {
					if let Err(e) = self.request_sink.send(BroadcastedRequest {
						id: hash.clone(),
						payload: request_to_broadcast,
						rpc_method: s,
					}) {
						error!("Could not send broadcasted request, reason: {:?}", e);
					} else {
						debug!("Broadcast request OK, hash = {}", hash);
					}
				}
				result
			},
		}
	}

	fn remove_top(
		&self,
		bytes_or_hash: TrustedOperationOrHash<TCS, G>,
		shard: ShardIdentifier,
		inblock: bool,
	) -> Result<TxHash> {
		let hash = match bytes_or_hash {
			TrustedOperationOrHash::Hash(h) => Ok(h),
			TrustedOperationOrHash::OperationEncoded(bytes) => {
				match Decode::decode(&mut bytes.as_slice()) {
					Ok(op) => Ok(self.top_pool.hash_of(&op)),
					Err(e) => {
						error!("Failed to decode trusted operation: {:?}, operation will not be removed from pool", e);
						Err(StateRpcError::CodecError(e))
					},
				}
			},
			TrustedOperationOrHash::Operation(op) => Ok(self.top_pool.hash_of(&op)),
		}?;

		info!("removing {:?} from top pool", hash);

		// Update metric
		if let Err(e) = self.ocall_api.update_metric(EnclaveMetric::TopPoolSizeDecrement) {
			warn!("Failed to update metric for top pool size: {:?}", e);
		}

		let removed_op_hash = self
			.top_pool
			.remove_invalid(&[hash], shard, inblock)
			// Only remove a single element, so first should return Ok().
			.first()
			.map(|o| o.hash())
			.ok_or(PoolError::InvalidTrustedOperation)?;

		Ok(removed_op_hash)
	}
}

fn map_top_error<P: TrustedOperationPool<StfTrustedOperation<TCS, G>>, TCS, G>(
	error: P::Error,
) -> RpcError
where
	TCS: PartialEq + Encode + Debug,
	G: PartialEq + Encode + Debug,
{
	StateRpcError::PoolError(
		error
			.into_pool_error()
			.map(Into::into)
			.unwrap_or_else(|_error| PoolError::Verification),
	)
	.into()
}

impl<
		TopPool,
		TopFilter,
		BroadcastedTopFilter,
		StateFacade,
		ShieldingKeyRepository,
		OCallApi,
		TCS,
		G,
	> AuthorApi<TxHash, BlockHash, TCS, G>
	for Author<
		TopPool,
		TopFilter,
		BroadcastedTopFilter,
		StateFacade,
		ShieldingKeyRepository,
		OCallApi,
		TCS,
		G,
	> where
	TopPool: TrustedOperationPool<StfTrustedOperation<TCS, G>> + Sync + Send + 'static,
	TopFilter: Filter<Value = StfTrustedOperation<TCS, G>>,
	BroadcastedTopFilter: Filter<Value = StfTrustedOperation<TCS, G>>,
	StateFacade: QueryShardState,
	ShieldingKeyRepository: AccessKey,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoDecrypt + 'static,
	OCallApi: EnclaveMetricsOCallApi + Send + Sync + 'static,
	G: PartialEq
		+ Encode
		+ Decode
		+ Clone
		+ PoolTransactionValidation
		+ Debug
		+ Send
		+ Sync
		+ 'static,
	TCS: PartialEq
		+ Encode
		+ Decode
		+ Clone
		+ Debug
		+ Send
		+ Sync
		+ TrustedCallVerification
		+ 'static,
{
	fn submit_top<R: DecryptableRequest + Encode>(&self, req: R) -> PoolFuture<TxHash, RpcError> {
		self.process_top(req, TopSubmissionMode::Submit)
	}

	/// Get hash of TrustedOperation
	fn hash_of(&self, xt: &StfTrustedOperation<TCS, G>) -> TxHash {
		self.top_pool.hash_of(xt)
	}

	fn pending_tops(&self, shard: ShardIdentifier) -> Result<Vec<Vec<u8>>> {
		Ok(self.top_pool.ready(shard).map(|top| top.data().encode()).collect())
	}

	fn get_pending_getters(&self, shard: ShardIdentifier) -> Vec<StfTrustedOperation<TCS, G>> {
		self.top_pool
			.ready(shard)
			.filter_map(|o| match o.data() {
				StfTrustedOperation::<TCS, G>::get(_) => Some(o.data().clone()),
				StfTrustedOperation::<TCS, G>::direct_call(_)
				| StfTrustedOperation::<TCS, G>::indirect_call(_) => None,
			})
			.collect()
	}

	fn get_pending_trusted_calls(
		&self,
		shard: ShardIdentifier,
	) -> Vec<StfTrustedOperation<TCS, G>> {
		self.top_pool
			.ready(shard)
			.filter_map(|o| match o.data() {
				StfTrustedOperation::<TCS, G>::direct_call(_)
				| StfTrustedOperation::<TCS, G>::indirect_call(_) => Some(o.data().clone()),
				StfTrustedOperation::<TCS, G>::get(_) => None,
			})
			.collect()
	}

	fn get_status(&self, shard: ShardIdentifier) -> PoolStatus {
		self.top_pool.status(shard)
	}

	fn get_pending_trusted_calls_for(
		&self,
		shard: ShardIdentifier,
		account: &AccountId,
	) -> Vec<StfTrustedOperation<TCS, G>> {
		self.get_pending_trusted_calls(shard)
			.into_iter()
			.filter(|o| o.signed_caller_account().as_ref() == Some(account))
			.collect()
	}

	fn get_shards(&self) -> Vec<ShardIdentifier> {
		self.top_pool.shards()
	}

	fn list_handled_shards(&self) -> Vec<ShardIdentifier> {
		self.state_facade.list_shards().unwrap_or_default()
	}

	fn remove_calls_from_pool(
		&self,
		shard: ShardIdentifier,
		executed_calls: Vec<(TrustedOperationOrHash<TCS, G>, bool)>,
	) -> Vec<TrustedOperationOrHash<TCS, G>> {
		let mut failed_to_remove = Vec::new();
		for (executed_call, inblock) in executed_calls {
			if let Err(e) = self.remove_top(executed_call.clone(), shard, inblock) {
				// We don't want to return here before all calls have been iterated through,
				// hence log message and collect failed calls in vec.
				warn!("Error removing trusted call from top pool: {:?}", e);
				failed_to_remove.push(executed_call);
			}
		}
		failed_to_remove
	}

	fn watch_top<R: DecryptableRequest + Encode>(
		&self,
		request: R,
	) -> PoolFuture<TxHash, RpcError> {
		self.process_top(request, TopSubmissionMode::SubmitWatch)
	}

	fn watch_and_broadcast_top<R: DecryptableRequest + Encode>(
		&self,
		request: R,
		json_rpc_method: String,
	) -> PoolFuture<TxHash, RpcError> {
		self.process_top(request, TopSubmissionMode::SubmitWatchAndBroadcast(json_rpc_method))
	}

	fn update_connection_state(&self, updates: Vec<(TxHash, (Vec<u8>, bool))>) {
		self.top_pool.update_connection_state(updates)
	}

	fn send_rpc_response(&self, hash: TxHash, encoded_value: Vec<u8>, do_watch: bool) {
		self.top_pool.send_rpc_response(hash, encoded_value, do_watch)
	}

	fn swap_rpc_connection_hash(&self, old_hash: TxHash, new_hash: TxHash) {
		self.top_pool.swap_rpc_connection_hash(old_hash, new_hash)
	}
}

impl<
		TopPool,
		TopFilter,
		BroadcastedTopFilter,
		StateFacade,
		ShieldingKeyRepository,
		OCallApi,
		TCS,
		G,
	> OnBlockImported
	for Author<
		TopPool,
		TopFilter,
		BroadcastedTopFilter,
		StateFacade,
		ShieldingKeyRepository,
		OCallApi,
		TCS,
		G,
	> where
	TopPool: TrustedOperationPool<StfTrustedOperation<TCS, G>> + Sync + Send + 'static,
	TopFilter: Filter<Value = StfTrustedOperation<TCS, G>>,
	BroadcastedTopFilter: Filter<Value = StfTrustedOperation<TCS, G>>,
	StateFacade: QueryShardState,
	ShieldingKeyRepository: AccessKey,
	<ShieldingKeyRepository as AccessKey>::KeyType: ShieldingCryptoDecrypt + 'static,
	OCallApi: EnclaveMetricsOCallApi + Send + Sync + 'static,
	G: PartialEq + Encode + Clone + PoolTransactionValidation + Debug + Send + Sync,
	TCS: PartialEq + Encode + Clone + Debug + Send + Sync,
{
	type Hash = TxHash;

	fn on_block_imported(&self, hashes: &[Self::Hash], block_hash: SidechainBlockHash) {
		self.top_pool.on_block_imported(hashes, block_hash)
	}
}
