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

use async_trait::async_trait;
use executor_primitives::AccountId;
use executor_storage::storage_key;
use parity_scale_codec::{Decode, Encode};
use rocksdb::DB;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type IntentId = heima_primitives::IntentId;

#[async_trait]
pub trait IntentIdStore: Send + Sync {
	async fn get(&self, account_id: &AccountId) -> Result<IntentId, ()>;
	async fn update(&self, account_id: AccountId, id: IntentId) -> Result<(), ()>;
}

pub struct InMemoryIntentIdStore {
	inner: RwLock<HashMap<AccountId, IntentId>>,
}

impl InMemoryIntentIdStore {
	pub fn new() -> Self {
		InMemoryIntentIdStore { inner: RwLock::new(HashMap::new()) }
	}
}

impl Default for InMemoryIntentIdStore {
	fn default() -> Self {
		Self::new()
	}
}

#[async_trait]
impl IntentIdStore for InMemoryIntentIdStore {
	async fn get(&self, account_id: &AccountId) -> Result<IntentId, ()> {
		Ok(*self.inner.read().await.get(account_id).unwrap_or(&1))
	}

	async fn update(&self, account_id: AccountId, id: IntentId) -> Result<(), ()> {
		self.inner.write().await.insert(account_id, id);
		Ok(())
	}
}

const STORAGE_NAME: &str = "intent_id_storage";

pub type IntentIdStorageKey = AccountId;

pub struct StorageDbIntentIdStore {
	db: Arc<DB>,
}

impl StorageDbIntentIdStore {
	pub fn new(db: Arc<DB>) -> Self {
		Self { db }
	}
}

#[async_trait]
impl IntentIdStore for StorageDbIntentIdStore {
	async fn get(&self, account_id: &AccountId) -> Result<IntentId, ()> {
		match self.db.get(storage_key(STORAGE_NAME, &account_id.encode())) {
			Ok(Some(value)) => IntentId::decode(&mut &value[..]).map_err(|e| {
				log::error!("Could not deode intent id: {:?}", e);
			}),
			Ok(None) => Ok(0),
			_ => {
				log::error!("Error getting intent id from storage for account {:?}", account_id);
				Err(())
			},
		}
	}

	async fn update(&self, account_id: AccountId, id: IntentId) -> Result<(), ()> {
		self.db
			.put(storage_key(STORAGE_NAME, &account_id.encode()), id.encode())
			.map_err(|e| {
				log::error!(
					"Error updating intent id in storage for account: {:?}, reason: {:?}",
					account_id,
					e
				);
			})
	}
}
