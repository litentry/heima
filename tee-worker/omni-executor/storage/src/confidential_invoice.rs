use crate::Storage;
use parity_scale_codec::{Decode, Encode};
use rocksdb::DB;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const STORAGE_NAME: &str = "confidential_invoice_storage";

#[derive(Encode)]
pub struct Key {
	pub invoice_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
pub enum InvoiceStatus {
	Pending,
	Paid,
	Expired,
	Cancelled,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct InvoiceMetadata {
	pub description: String,
	pub currency: String, // "USDC", "USDT", etc.
	pub chain_id: u64,
	pub seller_name: Option<String>,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct ConfidentialInvoice {
	pub invoice_id: String,
	pub seller_account: String,    // AccountId as string
	pub buyer_identifier: String,  // Email or account
	pub encrypted_amount: Vec<u8>, // AES-256 encrypted
	pub commitment: [u8; 32],      // keccak256(invoice_id, amount)
	pub metadata: InvoiceMetadata,
	pub status: InvoiceStatus,
	pub created_at: u64,
	pub paid_at: Option<u64>,
	pub tx_hash: Option<String>,
}

/// Parameters for creating a new confidential invoice
pub struct NewConfidentialInvoice {
	pub invoice_id: String,
	pub seller_account: String,
	pub buyer_identifier: String,
	pub encrypted_amount: Vec<u8>,
	pub commitment: [u8; 32],
	pub metadata: InvoiceMetadata,
}

pub struct ConfidentialInvoiceStorage {
	db: Arc<DB>,
}

impl ConfidentialInvoiceStorage {
	pub fn new(db: Arc<DB>) -> Self {
		Self { db }
	}

	/// Create a new confidential invoice
	pub fn create(&self, params: NewConfidentialInvoice) -> Result<(), String> {
		let key = Key { invoice_id: params.invoice_id.clone() };

		if self.contains_key(&key) {
			return Err(format!("Invoice ID already exists: {}", params.invoice_id));
		}

		let invoice = ConfidentialInvoice {
			invoice_id: params.invoice_id,
			seller_account: params.seller_account,
			buyer_identifier: params.buyer_identifier,
			encrypted_amount: params.encrypted_amount,
			commitment: params.commitment,
			metadata: params.metadata,
			status: InvoiceStatus::Pending,
			created_at: current_timestamp(),
			paid_at: None,
			tx_hash: None,
		};

		self.insert(&key, invoice)
			.map_err(|e| format!("Failed to insert invoice: {:?}", e))
	}

	/// Update invoice using a closure
	pub fn update<F>(&self, invoice_id: &str, updater: F) -> Result<(), String>
	where
		F: FnOnce(&mut ConfidentialInvoice),
	{
		let key = Key { invoice_id: invoice_id.to_string() };
		let mut invoice = self
			.get(&key)
			.map_err(|e| format!("Failed to get invoice: {:?}", e))?
			.ok_or_else(|| "Invoice not found".to_string())?;
		updater(&mut invoice);
		self.insert(&key, invoice)
			.map_err(|e| format!("Failed to update invoice: {:?}", e))
	}

	/// Get invoice by ID
	pub fn get_by_id(&self, invoice_id: &str) -> Result<Option<ConfidentialInvoice>, ()> {
		let key = Key { invoice_id: invoice_id.to_string() };
		self.get(&key)
	}
}

impl Storage<Key, ConfidentialInvoice> for ConfidentialInvoiceStorage {
	fn db(&self) -> Arc<crate::StorageDB> {
		self.db.clone()
	}

	fn name(&self) -> &'static str {
		STORAGE_NAME
	}
}

/// Get current Unix timestamp in seconds
fn current_timestamp() -> u64 {
	std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.expect("Time went backwards")
		.as_secs()
}
