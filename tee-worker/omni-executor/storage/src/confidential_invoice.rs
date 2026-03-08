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
	pub currency: String, // stores token address (e.g. "0x...")
	pub chain_id: u64,
	pub seller_name: Option<String>,
}

/// One recipient entry in a multi-seller invoice.
/// Each recipient gets their own independent pool commitment + secret.
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Recipient {
	/// Ethereum address of this recipient
	pub address: String,
	/// AES-256-GCM encrypted raw token amount for this recipient
	pub encrypted_amount: Vec<u8>,
	/// SHA256(invoice_id || recipient_index || amount || secret) — set on omni_payInvoice
	pub pool_commitment: Option<[u8; 32]>,
	/// TEE-only random secret — never leaves TEE in plaintext
	pub pool_secret: Option<[u8; 32]>,
	/// Merkle tree leaf index from on-chain Deposit event
	pub leaf_index: Option<u32>,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct ConfidentialInvoice {
	pub invoice_id: String,
	/// Address of the invoice creator (may or may not be in recipients)
	pub created_by: String,
	pub buyer_identifier: String, // Email or account
	/// Total encrypted amount (sum of all recipients' amounts)
	pub encrypted_amount: Vec<u8>,
	pub commitment: [u8; 32], // keccak256(invoice_id, total_amount)
	pub metadata: InvoiceMetadata,
	pub status: InvoiceStatus,
	pub created_at: u64,
	pub paid_at: Option<u64>,
	pub tx_hash: Option<String>,
	/// Per-recipient sub-commitments; one deposit per entry
	pub recipients: Vec<Recipient>,
	/// Deposit transaction hash (set after on-chain confirmation)
	pub pool_tx_hash: Option<String>,
}

/// Parameters for creating a new confidential invoice
pub struct NewConfidentialInvoice {
	pub invoice_id: String,
	pub created_by: String,
	pub buyer_identifier: String,
	pub encrypted_amount: Vec<u8>,
	pub commitment: [u8; 32],
	pub metadata: InvoiceMetadata,
	pub recipients: Vec<Recipient>,
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
			created_by: params.created_by,
			buyer_identifier: params.buyer_identifier,
			encrypted_amount: params.encrypted_amount,
			commitment: params.commitment,
			metadata: params.metadata,
			status: InvoiceStatus::Pending,
			created_at: current_timestamp(),
			paid_at: None,
			tx_hash: None,
			recipients: params.recipients,
			pool_tx_hash: None,
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

	/// Get all invoices created by a given address (case-insensitive).
	/// Scans the full storage prefix — acceptable for demo scale.
	pub fn get_by_creator(&self, created_by: &str) -> Vec<ConfidentialInvoice> {
		use oe_crypto::hashing::twox_128;
		use rocksdb::{Direction, IteratorMode};

		let prefix = twox_128(STORAGE_NAME.as_bytes()).to_vec();
		let creator_lower = created_by.to_lowercase();
		let mut results = Vec::new();

		let db = self.db();
		let iter = db.iterator(IteratorMode::From(&prefix, Direction::Forward));
		for item in iter.flatten() {
			let (k, v) = item;
			if !k.starts_with(&prefix) {
				break;
			}
			if let Ok(invoice) = ConfidentialInvoice::decode(&mut &v[..]) {
				if invoice.created_by.to_lowercase() == creator_lower {
					results.push(invoice);
				}
			}
		}

		results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
		results
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
