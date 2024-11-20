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

mod repository;
use itp_ocall_api::EnclaveOnChainOCallApi;
pub use repository::*;

mod in_memory_store;
pub use in_memory_store::InMemoryStore;

use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use itp_types::parentchain::{
	AccountId, BlockNumber, Header, Index as ParentchainIndex, ParentchainId,
};
use litentry_primitives::MemberAccount;

pub type OmniAccounts = BTreeMap<AccountId, Vec<MemberAccount>>;

#[derive(Debug)]
pub enum Error {
	LockPoisoning,
	OCallApiError(&'static str),
}

pub fn init_in_memory_omni_account_store<OCallApi>(
	ocall_api: Arc<OCallApi>,
) -> Result<(), &'static str>
where
	OCallApi: EnclaveOnChainOCallApi,
{
	let header: Header = ocall_api.get_header().map_err(|e| {
		log::error!("Failed to get header: {:?}", e);
		"Failed to get header"
	})?;
	let block_number: BlockNumber = header.number;
	let repository = OmniAccountRepository::new(ocall_api).with_header(header);
	let account_stores = repository.get_all().map_err(|_| "Failed to get all account stores")?;
	InMemoryStore::load_account_stores(account_stores)
		.map_err(|_| "Failed to load account stores")?;
	InMemoryStore::set_block_height(block_number).map_err(|_| "Failed to set block number")?;
	log::info!("In-memory store initialized successfully");

	Ok(())
}
