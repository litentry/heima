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
use crate::types::{AccountId, KeyPair, ShardIdentifier};
use alloc::vec::Vec;
use codec::{Decode, Encode};
use core::fmt::Debug;
use itp_sgx_runtime_primitives::types::Index;
use litentry_primitives::{Identity, MrEnclave};
use sp_runtime::transaction_validity::{TransactionValidityError, ValidTransaction};

/// checks authorization of stf getters
pub trait GetterAuthorization {
	fn is_authorized(&self) -> bool;
}

/// knows how to sign a trusted call input and provides a signed output
pub trait TrustedCallSigning<TCS> {
	fn sign(
		&self,
		pair: &KeyPair,
		nonce: Index,
		mrenclave: &[u8; 32],
		shard: &ShardIdentifier,
	) -> TCS;
}

/// enables TrustedCallSigned verification
pub trait TrustedCallVerification {
	fn sender_identity(&self) -> &Identity;

	fn nonce(&self) -> Index;

	fn verify_signature(&self, mrenclave: &[u8; 32], shard: &ShardIdentifier) -> bool;

	// Litentry: extend the trait for metric statistic purpose
	fn metric_name(&self) -> &'static str;
}

/// validation for top pool
pub trait PoolTransactionValidation {
	fn validate(&self) -> Result<ValidTransaction, TransactionValidityError>;
}

/// Trait to be implemented on the executor to serve helper methods of the executor
/// to the `IndirectDispatch` implementation.
pub trait IndirectExecutor<TCS, Error, RRU, SRU, ERU>
where
	TCS: PartialEq + Encode + Decode + Debug + Clone + Send + Sync + TrustedCallVerification,
{
	fn submit_trusted_call(&self, shard: ShardIdentifier, encrypted_trusted_call: Vec<u8>);

	fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>, Error>;

	fn encrypt(&self, value: &[u8]) -> Result<Vec<u8>, Error>;

	fn get_enclave_account(&self) -> Result<AccountId, Error>;

	fn get_mrenclave(&self) -> Result<MrEnclave, Error>;

	fn get_default_shard(&self) -> ShardIdentifier;

	fn sign_call_with_self<TC: Encode + Debug + TrustedCallSigning<TCS>>(
		&self,
		trusted_call: &TC,
		shard: &ShardIdentifier,
	) -> Result<TCS, Error>;

	fn get_relayer_registry_updater(&self) -> &RRU {
		unimplemented!()
	}

	fn get_signer_registry_updater(&self) -> &SRU {
		unimplemented!()
	}

	fn get_enclave_registry_updater(&self) -> &ERU {
		unimplemented!()
	}
}
