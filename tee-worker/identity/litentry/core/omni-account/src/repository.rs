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

use crate::{
	AccountId, Error, Header, MemberAccount, OmniAccounts, ParentchainId, ParentchainIndex,
};
use alloc::{borrow::ToOwned, collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use frame_support::storage::storage_prefix;
use itp_ocall_api::EnclaveOnChainOCallApi;
use itp_storage::{extract_blake2_128concat_key, storage_map_key, StorageHasher};
use sp_core::H256;

pub trait GetAccountStore {
	fn get_all(&self) -> Result<OmniAccounts, Error>;
}

pub trait GetOmniAccountInfo {
	fn get_nonce(&self, account_id: AccountId) -> Result<ParentchainIndex, Error>;
}

pub struct OmniAccountRepository<OCallApi: EnclaveOnChainOCallApi> {
	ocall_api: Arc<OCallApi>,
}

impl<OCallApi: EnclaveOnChainOCallApi> OmniAccountRepository<OCallApi> {
	pub fn new(ocall_api: Arc<OCallApi>) -> Self {
		Self { ocall_api }
	}

	pub fn with_header(self, header: Header) -> OmniAccountStoreRepository<OCallApi> {
		OmniAccountStoreRepository { ocall_api: self.ocall_api, header }
	}
}

impl<OCallApi: EnclaveOnChainOCallApi> GetOmniAccountInfo for OmniAccountRepository<OCallApi> {
	fn get_nonce(&self, account_id: AccountId) -> Result<ParentchainIndex, Error> {
		self.ocall_api
			.get_account_nonce(account_id)
			.map_err(|_| Error::OCallApiError("Failed to get account nonce"))
	}
}

pub struct OmniAccountStoreRepository<OCallApi: EnclaveOnChainOCallApi> {
	ocall_api: Arc<OCallApi>,
	header: Header,
}

impl<OCallApi: EnclaveOnChainOCallApi> OmniAccountStoreRepository<OCallApi> {
	pub fn get_account_by_member_hash(
		&self,
		member_hash: H256,
	) -> Result<Option<AccountId>, Error> {
		let storage_key = storage_map_key(
			"OmniAccount",
			"MemberAccountHash",
			&member_hash,
			&StorageHasher::Blake2_128Concat,
		);
		let storage_entry = self
			.ocall_api
			.get_storage_verified(storage_key, &self.header, &ParentchainId::Litentry)
			.map_err(|_| Error::OCallApiError("Failed to get storage"))?;
		let account_id = storage_entry.value().to_owned();

		Ok(account_id)
	}
}

impl<OCallApi: EnclaveOnChainOCallApi> GetAccountStore for OmniAccountStoreRepository<OCallApi> {
	fn get_all(&self) -> Result<OmniAccounts, Error> {
		let account_store_key_prefix = storage_prefix(b"OmniAccount", b"AccountStore");
		let page_size = 300;
		let mut start_key: Option<Vec<u8>> = None;
		let mut omni_accounts: OmniAccounts = BTreeMap::new();

		loop {
			let storage_keys_paged = self
				.ocall_api
				.get_storage_keys_paged(
					account_store_key_prefix.into(),
					page_size,
					start_key.clone(),
					Some(&self.header),
				)
				.map_err(|_| Error::OCallApiError("Failed to get storage keys"))?;

			if storage_keys_paged.is_empty() || storage_keys_paged.last().cloned() == start_key {
				break
			}

			start_key = storage_keys_paged.last().cloned();

			self.ocall_api
				.get_multiple_storages_verified(
					storage_keys_paged,
					&self.header,
					&ParentchainId::Litentry,
				)
				.map_err(|_| Error::OCallApiError("Failed to get multiple storages"))?
				.into_iter()
				.filter_map(|entry| {
					let account_id: AccountId = extract_blake2_128concat_key(entry.key.as_slice())?;
					let member_accounts: Vec<MemberAccount> = entry.value?;
					Some((account_id, member_accounts))
				})
				.for_each(|(account_id, member_accounts)| {
					omni_accounts.insert(account_id, member_accounts);
				});
		}

		Ok(omni_accounts)
	}
}
