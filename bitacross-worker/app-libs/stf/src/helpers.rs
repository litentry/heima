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
use crate::ENCLAVE_ACCOUNT_KEY;
use codec::{Decode, Encode};
use ita_sgx_runtime::{ParentchainLitentry, ParentchainTargetA, ParentchainTargetB};
use itp_stf_interface::{BlockMetadata, ShardCreationInfo};
use itp_stf_primitives::error::{StfError, StfResult};
use itp_storage::{storage_double_map_key, storage_map_key, storage_value_key, StorageHasher};

use itp_utils::stringify::account_id_to_string;
use log::*;
use std::prelude::v1::*;

#[cfg(feature = "development")]
pub use non_prod::*;

pub fn get_storage_value<V: Decode>(
	storage_prefix: &'static str,
	storage_key_name: &'static str,
) -> Option<V> {
	let key = storage_value_key(storage_prefix, storage_key_name);
	get_storage_by_key_hash(key)
}

pub fn get_storage_map<K: Encode, V: Decode + Clone>(
	storage_prefix: &'static str,
	storage_key_name: &'static str,
	map_key: &K,
	hasher: &StorageHasher,
) -> Option<V> {
	let key = storage_map_key::<K>(storage_prefix, storage_key_name, map_key, hasher);
	get_storage_by_key_hash(key)
}

pub fn get_storage_double_map<K: Encode, Q: Encode, V: Decode + Clone>(
	storage_prefix: &'static str,
	storage_key_name: &'static str,
	first: &K,
	first_hasher: &StorageHasher,
	second: &Q,
	second_hasher: &StorageHasher,
) -> Option<V> {
	let key = storage_double_map_key::<K, Q>(
		storage_prefix,
		storage_key_name,
		first,
		first_hasher,
		second,
		second_hasher,
	);
	get_storage_by_key_hash(key)
}

/// Get value in storage.
pub fn get_storage_by_key_hash<V: Decode>(key: Vec<u8>) -> Option<V> {
	if let Some(value_encoded) = sp_io::storage::get(&key) {
		if let Ok(value) = Decode::decode(&mut value_encoded.as_slice()) {
			Some(value)
		} else {
			error!("could not decode state for key {:?}", hex::encode(&key));
			None
		}
	} else {
		info!("key not found in state {:?}", hex::encode(key));
		None
	}
}

/// Get the AccountInfo key where the account is stored.
pub fn account_key_hash<AccountId: Encode>(account: &AccountId) -> Vec<u8> {
	storage_map_key("System", "Account", account, &StorageHasher::Blake2_128Concat)
}

pub fn enclave_signer_account<AccountId: Decode>() -> AccountId {
	get_storage_value("Sudo", ENCLAVE_ACCOUNT_KEY).expect("No enclave account")
}

/// Ensures an account is a registered enclave account.
pub fn ensure_enclave_signer_account<AccountId: Encode + Decode + PartialEq>(
	account: &AccountId,
) -> StfResult<()> {
	let expected_enclave_account: AccountId = enclave_signer_account();
	if &expected_enclave_account == account {
		Ok(())
	} else {
		error!(
			"Expected enclave account {}, but found {}",
			account_id_to_string(&expected_enclave_account),
			account_id_to_string(account)
		);
		Err(StfError::RequireEnclaveSignerAccount)
	}
}

pub fn set_block_number(block_number: u32) {
	sp_io::storage::set(&storage_value_key("System", "Number"), &block_number.encode());
}

pub fn ensure_self<AccountId: PartialEq>(signer: &AccountId, who: &AccountId) -> bool {
	signer == who
}

pub fn ensure_enclave_signer_or_self<AccountId: Encode + Decode + PartialEq>(
	signer: &AccountId,
	who: Option<AccountId>,
) -> bool {
	match who {
		Some(ref who) =>
			signer == &enclave_signer_account::<AccountId>() || ensure_self(signer, who),
		None => false,
	}
}

pub fn shard_creation_info() -> ShardCreationInfo {
	let maybe_litentry_info: Option<BlockMetadata> = ParentchainLitentry::creation_block_number()
		.and_then(|number| {
			ParentchainLitentry::creation_block_hash().map(|hash| BlockMetadata { number, hash })
		});
	let maybe_target_a_info: Option<BlockMetadata> = ParentchainTargetA::creation_block_number()
		.and_then(|number| {
			ParentchainTargetA::creation_block_hash().map(|hash| BlockMetadata { number, hash })
		});
	let maybe_target_b_info: Option<BlockMetadata> = ParentchainTargetB::creation_block_number()
		.and_then(|number| {
			ParentchainTargetB::creation_block_hash().map(|hash| BlockMetadata { number, hash })
		});

	ShardCreationInfo {
		litentry: maybe_litentry_info,
		target_a: maybe_target_a_info,
		target_b: maybe_target_b_info,
	}
}

#[cfg(feature = "development")]
mod non_prod {
	use super::*;
	use hex_literal::hex;
	use sp_runtime::AccountId32;

	pub const ALICE_ACCOUNTID32: AccountId32 =
		AccountId32::new(hex!["d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"]);

	pub fn ensure_alice(signer: &AccountId32) -> bool {
		signer == &ALICE_ACCOUNTID32
	}

	pub fn ensure_enclave_signer_or_alice(signer: &AccountId32) -> bool {
		signer == &enclave_signer_account::<AccountId32>() || ensure_alice(signer)
	}
}
