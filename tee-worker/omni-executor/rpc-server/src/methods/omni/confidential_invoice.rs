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

use crate::detailed_error::DetailedError;
use crate::server::RpcContext;
use crate::utils::types::RpcResultExt;
use crate::utils::validation::parse_rpc_params;
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::RpcModule;
use oe_core::intent::executor::IntentExecutor;
use oe_crypto::confidential::{decrypt_amount, encrypt_amount, generate_commitment};
use oe_storage::confidential_invoice::{
	InvoiceMetadata, InvoiceStatus, NewConfidentialInvoice, Recipient,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

/// One recipient specified when creating an invoice.
#[derive(Debug, Serialize, Deserialize)]
pub struct RecipientInput {
	/// Ethereum address of this recipient
	pub address: String,
	/// Amount this recipient should receive, as raw token units (string to avoid JS precision loss)
	pub amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvoiceParams {
	/// Address of the invoice creator (may differ from recipients, e.g. a billing admin)
	pub created_by: String,
	pub buyer_identifier: String, // Email or omni_account
	pub token_address: String,    // ERC20 token contract address (e.g., "0x...")
	pub chain_id: u64,
	pub description: String,
	pub nonce: Option<u64>, // Optional nonce for deterministic invoice ID
	/// At least one recipient required
	pub recipients: Vec<RecipientInput>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateInvoiceResponse {
	pub invoice_id: String,
	pub invoice_url: String,
	pub commitment: String,   // Hex-encoded total commitment
	pub total_amount: String, // Total raw token units (sum of all recipients)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetInvoiceDetailsParams {
	pub invoice_id: String,
	#[allow(dead_code)] // Will be used for JWT auth in production
	pub auth_token: Option<String>, // Optional JWT token for authorization
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecipientDetails {
	pub address: String,
	pub amount: Option<String>, // Decrypted raw amount (only shown when authorized)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetInvoiceDetailsResponse {
	pub invoice_id: String,
	pub amount: Option<String>, // Total amount (raw token units)
	pub token_address: String,  // ERC20 token contract address
	pub description: String,
	pub status: InvoiceStatus,
	pub created_at: u64,
	pub tx_hash: Option<String>,
	pub created_by: String,
	pub buyer_identifier: String,
	pub chain_id: u64,
	pub recipients: Vec<RecipientDetails>,
}

#[derive(Debug, Deserialize)]
pub struct PayInvoiceParams {
	pub invoice_id: String,
	pub buyer_account: String, // For verification
}

#[derive(Debug, Serialize, Clone)]
pub struct PayInvoiceResponse {
	pub status: String,
	pub message: String,
	pub amount: String, // Decrypted total amount for buyer to confirm
	pub commitment: String,
}

pub fn register_confidential_invoice<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	// Create confidential invoice
	module
		.register_async_method("omni_createConfidentialInvoice", |params, ctx, _ext| async move {
			let params = parse_rpc_params::<CreateInvoiceParams>(params)?;

			debug!(
				"Received omni_createConfidentialInvoice, created_by: {}, buyer: {}, recipients: {}",
				params.created_by,
				params.buyer_identifier,
				params.recipients.len()
			);

			// Validate token address format
			if !params.token_address.starts_with("0x") || params.token_address.len() != 42 {
				return Err(DetailedError::invalid_params(
					"token_address",
					"Invalid token address format (must be 0x... with 40 hex chars)",
				)
				.to_rpc_error());
			}

			// Validate creator address
			if !params.created_by.starts_with("0x") || params.created_by.len() != 42 {
				return Err(DetailedError::invalid_params(
					"created_by",
					"Invalid Ethereum address format",
				)
				.to_rpc_error());
			}

			// Require at least one recipient
			if params.recipients.is_empty() {
				return Err(DetailedError::invalid_params(
					"recipients",
					"At least one recipient is required",
				)
				.to_rpc_error());
			}

			// Parse and validate all recipient amounts
			let mut parsed_recipients: Vec<(String, u128)> = Vec::new();
			let mut total_amount: u128 = 0;

			for (i, r) in params.recipients.iter().enumerate() {
				if !r.address.starts_with("0x") || r.address.len() != 42 {
					return Err(DetailedError::invalid_params(
						"recipients",
						&format!("Recipient {}: invalid Ethereum address", i),
					)
					.to_rpc_error());
				}

				let amount: u128 = r.amount.parse().map_err(|_| {
					DetailedError::invalid_params(
						"recipients",
						&format!("Recipient {}: amount must be a valid integer", i),
					)
				})?;

				if amount == 0 {
					return Err(DetailedError::invalid_params(
						"recipients",
						&format!("Recipient {}: amount must be greater than zero", i),
					)
					.to_rpc_error());
				}

				total_amount = total_amount.checked_add(amount).ok_or_else(|| {
					DetailedError::invalid_params("recipients", "Total amount overflow")
						.to_rpc_error()
				})?;

				parsed_recipients.push((r.address.clone(), amount));
			}

			// Generate deterministic invoice ID based on creator + nonce (or random UUID)
			let invoice_id = if let Some(nonce) = params.nonce {
				use sha2::{Digest, Sha256};
				let mut hasher = Sha256::new();
				hasher.update(params.created_by.as_bytes());
				hasher.update(nonce.to_le_bytes());
				let hash = hasher.finalize();
				format!("inv_{}", hex::encode(&hash[..16]))
			} else {
				format!("inv_{}", uuid::Uuid::new_v4())
			};

			// Encrypt total amount with TEE key (for the invoice-level commitment)
			let encrypted_total = encrypt_amount(total_amount, &ctx.aes256_key)
				.map_err_internal("Failed to encrypt total amount")?;

			// Generate invoice-level commitment over total amount
			let commitment = generate_commitment(&invoice_id, total_amount);

			// Build per-recipient Recipient structs (encrypt each sub-amount individually)
			let mut recipients: Vec<Recipient> = Vec::new();
			for (address, amount) in &parsed_recipients {
				let encrypted_amount = encrypt_amount(*amount, &ctx.aes256_key)
					.map_err_internal("Failed to encrypt recipient amount")?;

				recipients.push(Recipient {
					address: address.clone(),
					encrypted_amount,
					pool_commitment: None,
					pool_secret: None,
					leaf_index: None,
				});
			}

			// Create invoice metadata
			let metadata = InvoiceMetadata {
				description: params.description,
				currency: params.token_address.clone(),
				chain_id: params.chain_id,
				seller_name: None,
			};

			// Store invoice
			let new_invoice = NewConfidentialInvoice {
				invoice_id: invoice_id.clone(),
				created_by: params.created_by,
				buyer_identifier: params.buyer_identifier,
				encrypted_amount: encrypted_total,
				commitment,
				metadata,
				recipients,
			};

			ctx.confidential_invoice_storage
				.create(new_invoice)
				.map_err_internal("Failed to store invoice")?;

			info!(
				"Created confidential invoice: {}, recipients: {}, total: {}",
				invoice_id,
				parsed_recipients.len(),
				total_amount
			);

			Ok::<CreateInvoiceResponse, ErrorObjectOwned>(CreateInvoiceResponse {
				invoice_id: invoice_id.clone(),
				invoice_url: format!("https://demo.heima.network/invoice/{}", invoice_id),
				commitment: hex::encode(commitment),
				total_amount: total_amount.to_string(),
			})
		})
		.expect("Failed to register omni_createConfidentialInvoice");

	// Get invoice details (decrypt for authorized user)
	module
		.register_async_method("omni_getInvoiceDetails", |params, ctx, _ext| async move {
			let params = parse_rpc_params::<GetInvoiceDetailsParams>(params)?;

			debug!("Received omni_getInvoiceDetails, invoice_id: {}", params.invoice_id);

			let invoice = ctx
				.confidential_invoice_storage
				.get_by_id(&params.invoice_id)
				.map_err_internal("Failed to get invoice")?
				.ok_or_else(|| {
					error!("Invoice not found: {}", params.invoice_id);
					DetailedError::invalid_params("invoice_id", "Invoice not found")
				})?;

			// MVP: allow anyone to view (production: verify JWT)
			let is_authorized = true;

			let amount = if is_authorized {
				let amount_units = decrypt_amount(&invoice.encrypted_amount, &ctx.aes256_key)
					.map_err_internal("Failed to decrypt amount")?;
				Some(amount_units.to_string())
			} else {
				None
			};

			// Decrypt per-recipient amounts if authorized
			let recipient_details: Vec<RecipientDetails> = invoice
				.recipients
				.iter()
				.map(|r| {
					let recipient_amount = if is_authorized {
						decrypt_amount(&r.encrypted_amount, &ctx.aes256_key)
							.ok()
							.map(|a| a.to_string())
					} else {
						None
					};
					RecipientDetails { address: r.address.clone(), amount: recipient_amount }
				})
				.collect();

			info!(
				"Retrieved invoice details: {}, status: {:?}, recipients: {}",
				params.invoice_id,
				invoice.status,
				invoice.recipients.len()
			);

			Ok::<GetInvoiceDetailsResponse, ErrorObjectOwned>(GetInvoiceDetailsResponse {
				invoice_id: invoice.invoice_id,
				amount,
				token_address: invoice.metadata.currency.clone(),
				description: invoice.metadata.description,
				status: invoice.status,
				created_at: invoice.created_at,
				tx_hash: invoice.tx_hash,
				chain_id: invoice.metadata.chain_id,
				created_by: invoice.created_by,
				buyer_identifier: invoice.buyer_identifier,
				recipients: recipient_details,
			})
		})
		.expect("Failed to register omni_getInvoiceDetails");

	// Pay confidential invoice (legacy helper — returns total amount for buyer to verify)
	module
		.register_async_method("omni_payConfidentialInvoice", |params, ctx, _ext| async move {
			let params = parse_rpc_params::<PayInvoiceParams>(params)?;

			debug!(
				"Received omni_payConfidentialInvoice, invoice_id: {}, buyer: {}",
				params.invoice_id, params.buyer_account
			);

			let invoice = ctx
				.confidential_invoice_storage
				.get_by_id(&params.invoice_id)
				.map_err_internal("Failed to get invoice")?
				.ok_or_else(|| {
					error!("Invoice not found: {}", params.invoice_id);
					DetailedError::invalid_params("invoice_id", "Invoice not found")
				})?;

			if invoice.status != InvoiceStatus::Pending {
				return Err(DetailedError::invalid_params(
					"invoice_id",
					"Invoice is not in pending status",
				)
				.to_rpc_error());
			}

			let amount_units = decrypt_amount(&invoice.encrypted_amount, &ctx.aes256_key)
				.map_err_internal("Failed to decrypt amount")?;

			info!("Prepared payment info for invoice: {}", params.invoice_id);

			// Return raw token units — the frontend knows the token decimals
			// and must do the conversion itself to avoid hardcoding decimals here.
			Ok::<PayInvoiceResponse, ErrorObjectOwned>(PayInvoiceResponse {
				status: "ready_to_pay".to_string(),
				message: format!(
					"Invoice amount (raw units): {}. Please proceed with payment.",
					amount_units
				),
				amount: amount_units.to_string(),
				commitment: hex::encode(invoice.commitment),
			})
		})
		.expect("Failed to register omni_payConfidentialInvoice");

	// List invoices by creator address
	#[derive(Debug, Deserialize)]
	struct ListInvoicesByCreatorParams {
		created_by: String,
	}

	#[derive(Debug, Clone, Serialize)]
	struct InvoiceSummary {
		invoice_id: String,
		description: String,
		status: String,
		total_amount_raw: String,
		created_at: u64,
		token_address: String,
		chain_id: u64,
	}

	#[derive(Debug, Clone, Serialize)]
	struct ListInvoicesByCreatorResponse {
		invoices: Vec<InvoiceSummary>,
	}

	module
		.register_async_method("omni_listInvoicesByCreator", |params, ctx, _ext| async move {
			let params = parse_rpc_params::<ListInvoicesByCreatorParams>(params)?;

			debug!("Received omni_listInvoicesByCreator, created_by: {}", params.created_by);

			let invoices = ctx.confidential_invoice_storage.get_by_creator(&params.created_by);

			let summaries: Vec<InvoiceSummary> = invoices
				.into_iter()
				.map(|inv| {
					let total_amount_raw = decrypt_amount(&inv.encrypted_amount, &ctx.aes256_key)
						.map(|a| a.to_string())
						.unwrap_or_default();
					InvoiceSummary {
						invoice_id: inv.invoice_id,
						description: inv.metadata.description,
						status: format!("{:?}", inv.status),
						total_amount_raw,
						created_at: inv.created_at,
						token_address: inv.metadata.currency,
						chain_id: inv.metadata.chain_id,
					}
				})
				.collect();

			info!("Listed {} invoices for creator: {}", summaries.len(), params.created_by);

			Ok::<ListInvoicesByCreatorResponse, ErrorObjectOwned>(ListInvoicesByCreatorResponse {
				invoices: summaries,
			})
		})
		.expect("Failed to register omni_listInvoicesByCreator");

	// Delete invoice (for testing/admin)
	#[derive(Debug, Deserialize)]
	struct DeleteInvoiceParams {
		invoice_id: String,
	}

	module
		.register_async_method("omni_deleteInvoice", |params, ctx, _ext| async move {
			use oe_storage::Storage;

			let params = parse_rpc_params::<DeleteInvoiceParams>(params)?;

			debug!("Received omni_deleteInvoice, invoice_id: {}", params.invoice_id);

			let key =
				oe_storage::confidential_invoice::Key { invoice_id: params.invoice_id.clone() };

			if !ctx.confidential_invoice_storage.contains_key(&key) {
				return Err(
					DetailedError::invalid_params("invoice_id", "Invoice not found").to_rpc_error()
				);
			}

			ctx.confidential_invoice_storage
				.remove(&key)
				.map_err(|_| DetailedError::internal_error("Failed to delete invoice"))?;

			info!("Deleted invoice: {}", params.invoice_id);

			Ok::<serde_json::Value, ErrorObjectOwned>(serde_json::json!({
				"success": true,
				"message": format!("Invoice {} deleted", params.invoice_id)
			}))
		})
		.expect("Failed to register omni_deleteInvoice");

	// Mark invoice as paid after on-chain deposit confirms
	#[derive(Debug, Deserialize)]
	struct MarkInvoicePaidParams {
		invoice_id: String,
		tx_hash: String,
	}

	module
		.register_async_method("omni_markInvoicePaid", |params, ctx, _ext| async move {
			let params = parse_rpc_params::<MarkInvoicePaidParams>(params)?;

			debug!(
				"Received omni_markInvoicePaid, invoice_id: {}, tx_hash: {}",
				params.invoice_id, params.tx_hash
			);

			let invoice = ctx
				.confidential_invoice_storage
				.get_by_id(&params.invoice_id)
				.map_err_internal("Failed to get invoice")?
				.ok_or_else(|| {
					DetailedError::invalid_params("invoice_id", "Invoice not found").to_rpc_error()
				})?;

			if invoice.status != InvoiceStatus::Pending {
				return Err(DetailedError::invalid_params(
					"invoice_id",
					"Invoice is not in pending status",
				)
				.to_rpc_error());
			}

			ctx.confidential_invoice_storage
				.update(&params.invoice_id, |inv| {
					inv.status = InvoiceStatus::Paid;
					inv.tx_hash = Some(params.tx_hash.clone());
				})
				.map_err_internal("Failed to update invoice status")?;

			info!("Invoice {} marked as paid, tx: {}", params.invoice_id, params.tx_hash);

			Ok::<serde_json::Value, ErrorObjectOwned>(serde_json::json!({
				"success": true,
				"invoice_id": params.invoice_id
			}))
		})
		.expect("Failed to register omni_markInvoicePaid");
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{start_server, ShieldingKey};
	use jsonrpsee::core::client::ClientT;
	use jsonrpsee::rpc_params;
	use jsonrpsee::ws_client::WsClientBuilder;
	use oe_client_binance::mocks::MockBinanceApiClient;
	use oe_client_pumpx::PumpxApiClient;
	use oe_client_signer::{mocks::MockSignerClient, SignerClient};
	use oe_client_wildmeta::{MockWildmetaApi, WildmetaApi};
	use oe_core::config::ConfigLoader;
	use oe_core::intent::executor::MockedIntentExecutor;
	use oe_storage::{
		ConfidentialInvoiceStorage, LoanRecordStorage, StorageDB, WildmetaTimestampStorage,
	};
	use rsa::{pkcs1::EncodeRsaPrivateKey, rand_core::OsRng, RsaPrivateKey};
	use std::collections::HashMap;
	use std::sync::Arc;
	use tempfile::tempdir;

	#[tokio::test]
	async fn test_create_and_get_invoice() {
		let tmp_dir = tempdir().unwrap();
		let port = 2010;
		let shielding_key = ShieldingKey::new();
		let db = Arc::new(StorageDB::open_default(tmp_dir.path()).unwrap());

		let mut rng = OsRng;
		let rsa_private_key =
			RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
		let jwt_private_key = rsa_private_key.to_pkcs1_der().unwrap();
		let pumpx_api = PumpxApiClient::new("https://api.pumpx.ai".to_string());
		let config_loader = ConfigLoader::from_env();
		let signer_client: Arc<Box<dyn SignerClient>> = Arc::new(Box::new(MockSignerClient::new()));
		let oe_client_binance_client: Arc<dyn oe_client_binance::BinancePaymasterApi> =
			Arc::new(MockBinanceApiClient::new());

		let wildmeta_api: Arc<Box<dyn WildmetaApi>> = Arc::new(Box::new(MockWildmetaApi));
		let wildmeta_timestamp_storage = Arc::new(WildmetaTimestampStorage::new(db.clone()));
		let loan_record_storage = Arc::new(LoanRecordStorage::new(db.clone()));
		let confidential_invoice_storage = Arc::new(ConfidentialInvoiceStorage::new(db.clone()));

		let (cross_chain_intent_executor, _cross_chain_mock_recv) = MockedIntentExecutor::new();
		let aes_key = [42u8; 32];
		let entry_point_clients = HashMap::new();

		start_server(
			port,
			shielding_key.clone(),
			Arc::new(Box::new(pumpx_api)),
			db,
			jwt_private_key.as_bytes().to_vec(),
			&config_loader,
			signer_client,
			oe_client_binance_client,
			wildmeta_api,
			wildmeta_timestamp_storage,
			loan_record_storage,
			confidential_invoice_storage,
			[0u8; 33],
			[0u8; 32],
			[0u8; 33],
			Arc::new(cross_chain_intent_executor),
			aes_key,
			Arc::new(entry_point_clients),
		)
		.await
		.unwrap();

		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();

		// Create invoice with two recipients
		let create_response: CreateInvoiceResponse = client
			.request(
				"omni_createConfidentialInvoice",
				rpc_params![
					"0x742d35Cc6634C0532925a3b844Bc9e7595f6bEb0", // created_by
					"buyer@example.com",                          // buyer_identifier
					"0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d", // token_address
					421614u64,                                    // chain_id
					"Test invoice",                               // description
					serde_json::json!([
						{"address": "0x742d35Cc6634C0532925a3b844Bc9e7595f6bEb0", "amount": "30000000000"},
						{"address": "0xAbCdEf0123456789AbCdEf0123456789AbCdEf01", "amount": "20000000000"}
					])  // recipients
				],
			)
			.await
			.unwrap();

		assert!(create_response.invoice_id.starts_with("inv_"));
		assert!(!create_response.commitment.is_empty());
		assert_eq!(create_response.total_amount, "50000000000");

		// Get invoice details
		let get_response: GetInvoiceDetailsResponse = client
			.request(
				"omni_getInvoiceDetails",
				rpc_params![create_response.invoice_id.clone(), None::<String>],
			)
			.await
			.unwrap();

		assert_eq!(get_response.invoice_id, create_response.invoice_id);
		assert_eq!(get_response.amount, Some("50000000000".to_string()));
		assert_eq!(get_response.token_address, "0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d");
		assert_eq!(get_response.chain_id, 421614);
		assert!(matches!(get_response.status, InvoiceStatus::Pending));
		assert_eq!(get_response.recipients.len(), 2);
		assert_eq!(get_response.recipients[0].amount, Some("30000000000".to_string()));
		assert_eq!(get_response.recipients[1].amount, Some("20000000000".to_string()));
	}
}
