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

#![cfg_attr(all(not(target_env = "sgx"), not(feature = "std")), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate alloc;

use crate::storage::StorageEntry;
use codec::{Decode, Encode};
use itp_sgx_crypto::ShieldingCryptoDecrypt;
use sp_std::{boxed::Box, fmt::Debug, vec::Vec};

pub mod parentchain;
pub mod sidechain;
pub mod storage;
pub use sidechain::SidechainBlockHash;

pub use itp_sgx_runtime_primitives::types::*;
pub use litentry_primitives::{
	decl_rsa_request, Assertion, AttestationType, DcapProvider, DecryptableRequest, Enclave,
	EnclaveFingerprint, Fmspc, MrEnclave, SidechainBlockNumber, WorkerType,
};
use parentchain::Index as ParentchainIndex;
pub use sp_core::{crypto::AccountId32 as AccountId, H256};

pub type IpfsHash = [u8; 46];
pub type CallIndex = [u8; 2];

pub type PostOpaqueTaskFn = (CallIndex, RsaRequest);

/// Simple blob to hold an encoded call
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct OpaqueCall(pub Vec<u8>);

impl OpaqueCall {
	/// Convert call tuple to an `OpaqueCall`.
	pub fn from_tuple<C: Encode>(call: &C) -> Self {
		OpaqueCall(call.encode())
	}
}

impl Encode for OpaqueCall {
	fn encode(&self) -> Vec<u8> {
		self.0.clone()
	}
}

// Litentry: re-declared due to orphan rule (that's why macro is used)
decl_rsa_request!(Debug);

impl DecryptableRequest for RsaRequest {
	type Error = ();

	fn shard(&self) -> ShardIdentifier {
		self.shard
	}

	fn payload(&self) -> &[u8] {
		self.payload.as_slice()
	}

	fn decrypt<T: Debug>(
		&mut self,
		enclave_shielding_key: Box<dyn ShieldingCryptoDecrypt<Error = T>>,
	) -> core::result::Result<Vec<u8>, ()> {
		enclave_shielding_key.decrypt(self.payload.as_slice()).map_err(|_| ())
	}
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Eq)]
pub enum DirectRequestStatus {
	/// Direct request was successfully executed
	#[codec(index = 0)]
	Ok,
	/// Trusted Call Status
	/// Litentry: embed the top hash here - TODO - use generic type?
	#[codec(index = 1)]
	TrustedOperationStatus(TrustedOperationStatus, H256),
	/// Direct request could not be executed
	#[codec(index = 2)]
	Error,
	#[codec(index = 3)]
	Processing(H256),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode, Eq)]
pub enum TrustedOperationStatus {
	/// TrustedOperation is submitted to the top pool.
	#[codec(index = 0)]
	Submitted,
	/// TrustedOperation is part of the future queue.
	#[codec(index = 1)]
	Future,
	/// TrustedOperation is part of the ready queue.
	#[codec(index = 2)]
	Ready,
	/// The operation has been broadcast to the given peers.
	#[codec(index = 3)]
	Broadcast,
	/// TrustedOperation has been included in block with given hash.
	#[codec(index = 4)]
	InSidechainBlock(BlockHash),
	/// The block this operation was included in has been retracted.
	#[codec(index = 5)]
	Retracted,
	/// Maximum number of finality watchers has been reached,
	/// old watchers are being removed.
	#[codec(index = 6)]
	FinalityTimeout,
	/// TrustedOperation has been finalized by a finality-gadget, e.g GRANDPA
	#[codec(index = 7)]
	Finalized,
	/// TrustedOperation has been replaced in the pool, by another operation
	/// that provides the same tags. (e.g. same (sender, nonce)).
	#[codec(index = 8)]
	Usurped,
	/// TrustedOperation has been dropped from the pool because of the limit.
	#[codec(index = 9)]
	Dropped,
	/// TrustedOperation is no longer valid in the current state.
	#[codec(index = 10)]
	Invalid,
	/// TrustedOperation has been executed.
	#[codec(index = 11)]
	TopExecuted(Vec<u8>, bool),
	/// TrustedOperation whose successor top has been executed, but the top itself not
	#[codec(index = 12)]
	SuccessorExecuted,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub enum WorkerRequest {
	#[codec(index = 0)]
	ChainStorage(Vec<u8>, Option<BlockHash>), // (storage_key, at_block)
	#[codec(index = 1)]
	ChainStorageKeys(Vec<u8>, Option<BlockHash>), // (storage_key_prefix, at_block)
	#[codec(index = 2)]
	ChainStorageKeysPaged(Vec<u8>, u32, Option<Vec<u8>>, Option<BlockHash>), // (storage_key_prefix, count, start_key, at_block)
	#[codec(index = 3)]
	ChainHeader(Option<BlockHash>), // (at_block)
	#[codec(index = 4)]
	ChainAccountNonce(Vec<u8>), // (account_id)
}

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub enum WorkerResponse<V: Encode + Decode> {
	#[codec(index = 0)]
	ChainStorage(Vec<u8>, Option<V>, Option<Vec<Vec<u8>>>), // (storage_key, storage_value, storage_proof)
	#[codec(index = 1)]
	ChainStorageKeys(Vec<Vec<u8>>), // (storage_keys)
	#[codec(index = 2)]
	ChainHeader(Option<V>), // (header)
	#[codec(index = 3)]
	ChainAccountNonce(Option<ParentchainIndex>), // (account_nonce)
}

impl From<WorkerResponse<Vec<u8>>> for StorageEntry<Vec<u8>> {
	fn from(response: WorkerResponse<Vec<u8>>) -> Self {
		match response {
			WorkerResponse::ChainStorage(key, value, proof) => StorageEntry { key, value, proof },
			_ => StorageEntry::default(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn opaque_call_encodes_correctly() {
		let call_tuple = ([1u8, 2u8], 5u8);
		let call = OpaqueCall::from_tuple(&call_tuple);
		assert_eq!(call.encode(), call_tuple.encode())
	}
}
