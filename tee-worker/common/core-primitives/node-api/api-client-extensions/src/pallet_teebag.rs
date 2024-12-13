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

use crate::ApiResult;
use itp_api_client_types::{traits::GetStorage, Api, Config, Request};
use itp_types::{AccountId, Enclave, ShardIdentifier, WorkerType};

pub const TEEBAG: &str = "Teebag";

/// ApiClient extension that enables communication with the `teebag` pallet.
pub trait PalletTeebagApi {
	type Hash;

	fn enclave(
		&self,
		account: &AccountId,
		at_block: Option<Self::Hash>,
	) -> ApiResult<Option<Enclave>>;
	fn enclave_count(
		&self,
		worker_type: WorkerType,
		at_block: Option<Self::Hash>,
	) -> ApiResult<u64>;
	fn primary_enclave_identifier_for_shard(
		&self,
		worker_type: WorkerType,
		shard: &ShardIdentifier,
		at_block: Option<Self::Hash>,
	) -> ApiResult<Option<AccountId>>;
	fn primary_enclave_for_shard(
		&self,
		worker_type: WorkerType,
		shard: &ShardIdentifier,
		at_block: Option<Self::Hash>,
	) -> ApiResult<Option<Enclave>>;
	fn all_enclaves(
		&self,
		worker_type: WorkerType,
		at_block: Option<Self::Hash>,
	) -> ApiResult<Vec<Enclave>>;
}

impl<RuntimeConfig, Client> PalletTeebagApi for Api<RuntimeConfig, Client>
where
	RuntimeConfig: Config,
	Client: Request,
{
	type Hash = RuntimeConfig::Hash;

	fn enclave(
		&self,
		account: &AccountId,
		at_block: Option<Self::Hash>,
	) -> ApiResult<Option<Enclave>> {
		self.get_storage_map(TEEBAG, "EnclaveRegistry", account, at_block)
	}

	fn enclave_count(
		&self,
		worker_type: WorkerType,
		at_block: Option<Self::Hash>,
	) -> ApiResult<u64> {
		// Vec<> and BoundedVec<> have the same encoding, thus they are used interchangeably
		let identifiers: Vec<AccountId> = self
			.get_storage_map(TEEBAG, "EnclaveIdentifier", worker_type, at_block)?
			.unwrap_or_default();
		Ok(identifiers.len() as u64)
	}

	// please note we don't use dedicated on-chain storage for this (like the upstream `WorkerForShard`)
	// so this API will always return the "first" registered and qualified enclave.
	// Wheter it meets our needs needs to be further evaluated
	fn primary_enclave_identifier_for_shard(
		&self,
		worker_type: WorkerType,
		shard: &ShardIdentifier,
		at_block: Option<Self::Hash>,
	) -> ApiResult<Option<AccountId>> {
		let identifiers: Vec<AccountId> = self
			.get_storage_map(TEEBAG, "EnclaveIdentifier", worker_type, at_block)?
			.unwrap_or_default();
		let mut maybe_account: Option<AccountId> = None;
		for account in identifiers {
			match self.enclave(&account, at_block)? {
				Some(e) =>
					if e.mrenclave == shard.as_ref() {
						maybe_account = Some(account.clone());
						break
					},
				None => continue,
			}
		}
		Ok(maybe_account)
	}

	fn primary_enclave_for_shard(
		&self,
		worker_type: WorkerType,
		shard: &ShardIdentifier,
		at_block: Option<Self::Hash>,
	) -> ApiResult<Option<Enclave>> {
		self.primary_enclave_identifier_for_shard(worker_type, shard, at_block)?
			.map_or_else(|| Ok(None), |account| self.enclave(&account, at_block))
	}

	fn all_enclaves(
		&self,
		worker_type: WorkerType,
		at_block: Option<Self::Hash>,
	) -> ApiResult<Vec<Enclave>> {
		let identifiers: Vec<AccountId> = self
			.get_storage_map(TEEBAG, "EnclaveIdentifier", worker_type, at_block)?
			.unwrap_or_default();

		let enclaves = identifiers
			.into_iter()
			.filter_map(|account| self.enclave(&account, at_block).ok()?)
			.collect();
		Ok(enclaves)
	}
}
