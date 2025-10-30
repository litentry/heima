use crate::Storage;
use executor_primitives::AccountId;
use parity_scale_codec::{Decode, Encode};
use rocksdb::{Direction, IteratorMode, DB};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const STORAGE_NAME: &str = "loan_record_storage";

#[derive(Encode)]
pub struct Key {
	pub account_id: AccountId,
	pub nonce: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
pub enum LoanState {
	SpotSold,
	ToPerpMoved,
	HedgeOpened,
	HedgeClosed,
	ToSpotMoved,
	SpotBought,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct LoanRecord {
	pub collateral_ticker: String,
	pub collateral_size: String,
	pub usdc_sold: String,
	pub usdc_loaned: String,
	pub usdc_for_perp: String,
	// contains all tx hashes that the worker submits to interact with corewriter
	// each tuple is a name -> hash mapping, e.g.:
	// ("spot_sell", "0x1234...")
	// we expect 6 hashes for a full loan request and payback cycle in normal cases,
	// and up to 7 hashes if the hedge were partially filled and need to canceled and closed
	pub txs: Vec<(String, String)>,
	// contains all cloids that the worker generate to track the hypercore orders
	// each tuple is a name -> cloid mapping, e.g.:
	// ("spot_sell", "173494584")
	// we expect 4 cloids for a full loan request and payback cycle
	pub cloids: Vec<(String, String)>,
	/// Actual position size that was opened for this loan (filled or partially filled)
	/// This should be set after the hedge order completes in request_loan
	pub position_size: String,
	pub state: LoanState,
}

/// Parameters for creating a new loan record
pub struct NewLoanRecord {
	pub collateral_ticker: String,
	pub collateral_size: String,
	pub usdc_sold: String,
	pub usdc_loaned: String,
	pub usdc_for_perp: String,
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

		// Query all records for the account by iterating with storage name prefix
		let mut results = Vec::new();
		use executor_crypto::hashing::twox_128;
		let storage_prefix = twox_128(STORAGE_NAME.as_bytes()).to_vec();

		let iter = self.db.iterator(IteratorMode::From(&storage_prefix, Direction::Forward));

		let account_id_bytes = account_id.encode();

		for (key_bytes, value_bytes) in iter.flatten() {
			// Check if this key has our storage prefix
			if !key_bytes.starts_with(&storage_prefix) {
				break;
			}

			// The key structure is: twox_128(storage_name) + blake2_128(encoded_key) + encoded_key
			// encoded_key is: account_id (32 bytes) + nonce (8 bytes)
			let twox_len = 16;
			let blake2_len = 16;
			let account_offset = twox_len + blake2_len;
			let account_len = 32;
			let nonce_offset = account_offset + account_len;

			// Check if this key belongs to our account
			if key_bytes.len() >= nonce_offset + 8 {
				let key_account_id = &key_bytes[account_offset..account_offset + account_len];
				if key_account_id == account_id_bytes.as_slice() {
					// Try to decode the value
					if let Ok(record) = LoanRecord::decode(&mut &value_bytes[..]) {
						// Extract nonce from the key
						let nonce_bytes = &key_bytes[nonce_offset..nonce_offset + 8];
						if let Ok(nonce) = u64::decode(&mut &nonce_bytes[..]) {
							results.push((nonce, record));
						}
					}
				}
			}
		}

		results
	}
}

impl LoanRecordStorage {
	pub fn create(&self, key: &Key, params: NewLoanRecord) -> Result<(), String> {
		if self.contains_key(key) {
			return Err(format!(
				"Key already exists, account_id: {:?}, nonce: {}",
				key.account_id, key.nonce
			));
		}

		let record = LoanRecord {
			collateral_ticker: params.collateral_ticker,
			collateral_size: params.collateral_size,
			usdc_sold: params.usdc_sold,
			usdc_loaned: params.usdc_loaned,
			usdc_for_perp: params.usdc_for_perp,
			txs: Vec::new(),
			cloids: Vec::new(),
			position_size: "0".to_string(),
			state: LoanState::SpotSold,
		};

		self.insert(key, record)
			.map_err(|e| format!("Failed to insert record: {:?}", e))
	}

	/// Update loan record using a closure
	pub fn update<F>(&self, key: &Key, updater: F) -> Result<(), String>
	where
		F: FnOnce(&mut LoanRecord),
	{
		let mut record = self
			.get(key)
			.map_err(|e| format!("Failed to get record: {:?}", e))?
			.ok_or_else(|| "Record not found".to_string())?;
		updater(&mut record);
		self.insert(key, record)
			.map_err(|e| format!("Failed to insert record: {:?}", e))
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
