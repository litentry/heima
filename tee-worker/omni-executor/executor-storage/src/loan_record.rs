use crate::{storage_key, Storage};
use executor_primitives::AccountId;
use parity_scale_codec::{Decode, Encode};
use rocksdb::{Direction, IteratorMode, DB};
use serde::Serialize;
use std::sync::Arc;

const STORAGE_NAME: &str = "loan_record_storage";

#[derive(Encode)]
pub struct Key {
	pub account_id: AccountId,
	pub nonce: u64,
}

#[derive(Debug, Clone, Encode, Decode, Serialize)]
pub struct LoanRecord {
	pub collateral_ticker: String,
	pub collateral_size: String,
	pub usdc_sold: String,
	pub usdc_loaned: String,
	pub spot_sell_cloid: String,
	pub hedge_open_cloid: String,
}

pub struct LoanRecordStorage {
	db: Arc<DB>,
}

impl LoanRecordStorage {
	pub fn new(db: Arc<DB>) -> Self {
		Self { db }
	}

	/// Query loan records for a given account
	/// If nonce is Some, return only that specific record
	/// If nonce is None, return all records for the account
	pub fn query_records(
		&self,
		account_id: &AccountId,
		nonce: Option<u64>,
	) -> Vec<(u64, LoanRecord)> {
		if let Some(n) = nonce {
			// Query specific nonce
			let key = Key { account_id: account_id.clone(), nonce: n };
			if let Ok(Some(record)) = self.get(&key) {
				return vec![(n, record)];
			}
			return vec![];
		}

		// Query all records for the account by iterating with prefix
		let mut results = Vec::new();
		let account_prefix = storage_key(STORAGE_NAME, &account_id.encode());

		let iter = self.db.iterator(IteratorMode::From(&account_prefix, Direction::Forward));

		for (key_bytes, value_bytes) in iter.flatten() {
			// Check if this key still has our prefix
			if !key_bytes.starts_with(&account_prefix) {
				break;
			}

			// Try to decode the value
			if let Ok(record) = LoanRecord::decode(&mut &value_bytes[..]) {
				// Extract nonce from the key
				// The key structure is: twox_128(storage_name) + blake2_128(encoded_key) + encoded_key
				// encoded_key is: account_id (32 bytes) + nonce (8 bytes)
				let twox_len = 16;
				let blake2_len = 16;
				let account_len = 32;
				let offset = twox_len + blake2_len + account_len;

				if key_bytes.len() >= offset + 8 {
					let nonce_bytes = &key_bytes[offset..offset + 8];
					if let Ok(nonce) = u64::decode(&mut &nonce_bytes[..]) {
						results.push((nonce, record));
					}
				}
			}
		}

		results
	}
}

impl Storage<Key, LoanRecord> for LoanRecordStorage {
	fn db(&self) -> Arc<crate::StorageDB> {
		self.db.clone()
	}

	fn name(&self) -> &'static str {
		STORAGE_NAME
	}
}
