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
use crate::utils::validation::{parse_as, parse_rpc_params};
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::RpcModule;
use oe_core::intent::executor::IntentExecutor;
use oe_crypto::confidential::{decrypt_amount, encrypt_amount, generate_commitment};
use oe_storage::confidential_invoice::{InvoiceMetadata, InvoiceStatus, NewConfidentialInvoice};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvoiceParams {
	pub seller_account: String,
	pub buyer_identifier: String, // Email or omni_account
	pub amount: String,           // Token amount as string (e.g., "100.50")
	pub token_address: String,    // ERC20 token contract address (e.g., "0x...")
	pub chain_id: u64,
	pub description: String,
	pub nonce: Option<u64>, // Optional nonce for deterministic invoice ID
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateInvoiceResponse {
	pub invoice_id: String,
	pub invoice_url: String,
	pub commitment: String, // Hex-encoded
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetInvoiceDetailsParams {
	pub invoice_id: String,
	#[allow(dead_code)] // Will be used for JWT auth in production
	pub auth_token: Option<String>, // Optional JWT token for authorization
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetInvoiceDetailsResponse {
	pub invoice_id: String,
	pub amount: Option<String>, // Only if authorized (raw token units as string)
	pub token_address: String,  // ERC20 token contract address
	pub description: String,
	pub status: InvoiceStatus,
	pub created_at: u64,
	pub tx_hash: Option<String>,
	pub seller_account: String,
	pub buyer_identifier: String,
	pub chain_id: u64,
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
	pub amount: String, // Decrypted amount for buyer to confirm
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
				"Received omni_createConfidentialInvoice, seller: {}, buyer: {}, amount: {}, token: {}",
				params.seller_account, params.buyer_identifier, params.amount, params.token_address
			);

			// Validate token address format (basic check)
			if !params.token_address.starts_with("0x") || params.token_address.len() != 42 {
				return Err(DetailedError::invalid_params(
					"token_address",
					"Invalid token address format (must be 0x... with 40 hex chars)",
				)
				.to_rpc_error());
			}

			// Parse amount as raw token units (no decimal conversion - frontend handles this)
			// Amount is stored as string representation of wei/smallest unit
			let amount_units: u128 = params.amount.parse().map_err(|_| {
				DetailedError::invalid_params("amount", "Amount must be a valid number")
			})?;

			if amount_units == 0 {
				return Err(DetailedError::invalid_params(
					"amount",
					"Amount must be greater than zero",
				)
				.to_rpc_error());
			}

			// Generate deterministic invoice ID based on seller + nonce
			// This allows multiple invoices to the same buyer with same amount
			// but ensures each invoice is unique based on the nonce
			let invoice_id = if let Some(nonce) = params.nonce {
				// Deterministic: SHA256(seller || nonce)
				use sha2::{Digest, Sha256};
				let mut hasher = Sha256::new();
				hasher.update(params.seller_account.as_bytes());
				hasher.update(nonce.to_le_bytes());
				let hash = hasher.finalize();
				format!("inv_{}", hex::encode(&hash[..16])) // Use first 16 bytes for readability
			} else {
				// Fallback to random UUID if nonce not provided
				format!("inv_{}", uuid::Uuid::new_v4())
			};

			// Encrypt amount with TEE key
			let encrypted_amount = encrypt_amount(amount_units, &ctx.aes256_key)
				.map_err_internal("Failed to encrypt amount")?;

			// Generate commitment
			let commitment = generate_commitment(&invoice_id, amount_units);

			// Create invoice metadata
			let metadata = InvoiceMetadata {
				description: params.description,
				currency: params.token_address.clone(), // Store token address in currency field
				chain_id: params.chain_id,
				seller_name: None,
			};

			// Create invoice record
			let new_invoice = NewConfidentialInvoice {
				invoice_id: invoice_id.clone(),
				seller_account: params.seller_account,
				buyer_identifier: params.buyer_identifier,
				encrypted_amount,
				commitment,
				metadata,
			};

			// Store invoice
			ctx.confidential_invoice_storage
				.create(new_invoice)
				.map_err_internal("Failed to store invoice")?;

			info!("Created confidential invoice: {}", invoice_id);

			Ok::<CreateInvoiceResponse, ErrorObjectOwned>(CreateInvoiceResponse {
				invoice_id: invoice_id.clone(),
				invoice_url: format!("https://demo.heima.network/invoice/{}", invoice_id),
				commitment: hex::encode(commitment),
			})
		})
		.expect("Failed to register omni_createConfidentialInvoice");

	// Get invoice details (decrypt for authorized user)
	module
		.register_async_method("omni_getInvoiceDetails", |params, ctx, _ext| async move {
			let params = parse_rpc_params::<GetInvoiceDetailsParams>(params)?;

			debug!("Received omni_getInvoiceDetails, invoice_id: {}", params.invoice_id);

			// Retrieve invoice
			let invoice = ctx
				.confidential_invoice_storage
				.get_by_id(&params.invoice_id)
				.map_err_internal("Failed to get invoice")?
				.ok_or_else(|| {
					error!("Invoice not found: {}", params.invoice_id);
					DetailedError::invalid_params("invoice_id", "Invoice not found")
				})?;

			// Check authorization - for MVP, allow anyone to view (in production, verify JWT)
			// TODO: Implement proper JWT verification for buyer_identifier
			let is_authorized = true; // Placeholder

			// Decrypt amount if authorized (return as raw units string)
			let amount = if is_authorized {
				let amount_units = decrypt_amount(&invoice.encrypted_amount, &ctx.aes256_key)
					.map_err_internal("Failed to decrypt amount")?;
				Some(amount_units.to_string())
			} else {
				None
			};

			info!(
				"Retrieved invoice details: {}, status: {:?}, amount_shown: {}",
				params.invoice_id,
				invoice.status,
				amount.is_some()
			);

			Ok::<GetInvoiceDetailsResponse, ErrorObjectOwned>(GetInvoiceDetailsResponse {
				invoice_id: invoice.invoice_id,
				amount,
				token_address: invoice.metadata.currency.clone(), // currency field stores token address
				description: invoice.metadata.description,
				status: invoice.status,
				created_at: invoice.created_at,
				tx_hash: invoice.tx_hash,
				chain_id: invoice.metadata.chain_id,
				seller_account: invoice.seller_account,
				buyer_identifier: invoice.buyer_identifier,
			})
		})
		.expect("Failed to register omni_getInvoiceDetails");

	// Pay confidential invoice (returns amount for buyer to verify before paying)
	module
		.register_async_method("omni_payConfidentialInvoice", |params, ctx, _ext| async move {
			let params = parse_rpc_params::<PayInvoiceParams>(params)?;

			debug!(
				"Received omni_payConfidentialInvoice, invoice_id: {}, buyer: {}",
				params.invoice_id, params.buyer_account
			);

			// Retrieve invoice
			let invoice = ctx
				.confidential_invoice_storage
				.get_by_id(&params.invoice_id)
				.map_err_internal("Failed to get invoice")?
				.ok_or_else(|| {
					error!("Invoice not found: {}", params.invoice_id);
					DetailedError::invalid_params("invoice_id", "Invoice not found")
				})?;

			// Check status
			if invoice.status != InvoiceStatus::Pending {
				return Err(DetailedError::invalid_params(
					"invoice_id",
					"Invoice is not in pending status",
				)
				.to_rpc_error());
			}

			// Decrypt amount
			let amount_units = decrypt_amount(&invoice.encrypted_amount, &ctx.aes256_key)
				.map_err_internal("Failed to decrypt amount")?;
			let amount_f64 = (amount_units as f64) / 1_000_000.0;

			// Return decrypted amount and commitment for buyer to verify and pay
			// The actual payment settlement will be handled by omni_settleUserOp
			info!(
				"Prepared payment info for invoice: {}, amount: {:.2}",
				params.invoice_id, amount_f64
			);

			Ok::<PayInvoiceResponse, ErrorObjectOwned>(PayInvoiceResponse {
				status: "ready_to_pay".to_string(),
				message: format!(
					"Invoice amount: {:.2} {}. Please proceed with payment.",
					amount_f64, invoice.metadata.currency
				),
				amount: format!("{:.2}", amount_f64),
				commitment: hex::encode(invoice.commitment),
			})
		})
		.expect("Failed to register omni_payConfidentialInvoice");

	// Delete invoice (for testing/admin)
	#[derive(Debug, Deserialize)]
	struct DeleteInvoiceParams {
		invoice_id: String,
	}

	module
		.register_async_method("omni_deleteInvoice", |params, ctx, _ext| async move {
			use oe_storage::Storage; // Import trait for remove/contains_key methods

			let params = parse_rpc_params::<DeleteInvoiceParams>(params)?;

			debug!("Received omni_deleteInvoice, invoice_id: {}", params.invoice_id);

			// Check if invoice exists first
			let key =
				oe_storage::confidential_invoice::Key { invoice_id: params.invoice_id.clone() };

			if !ctx.confidential_invoice_storage.contains_key(&key) {
				return Err(
					DetailedError::invalid_params("invoice_id", "Invoice not found").to_rpc_error()
				);
			}

			// Delete from storage using remove method
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
		let aes_key = [42u8; 32]; // Test key
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
			[0u8; 33], // Test ECDSA public key
			[0u8; 32], // Test bundler private key
			[0u8; 33], // Test bundler export authorized pubkey
			Arc::new(cross_chain_intent_executor),
			aes_key,
			Arc::new(entry_point_clients),
		)
		.await
		.unwrap();

		let url = format!("ws://127.0.0.1:{}", port);
		let client = WsClientBuilder::default().build(&url).await.unwrap();

		// Create invoice (amount in smallest units, e.g., 50000.00 USDC = 50000000000 units for 6 decimals)
		let create_response: CreateInvoiceResponse = client
			.request(
				"omni_createConfidentialInvoice",
				rpc_params![
					"seller@example.com",                         // seller_account
					"buyer@example.com",                          // buyer_identifier
					"50000000000",                                // amount (raw token units)
					"0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d", // token_address (example USDC on Arbitrum Sepolia)
					421614u64,                                    // chain_id
					"Test invoice"                                // description
				],
			)
			.await
			.unwrap();

		assert!(create_response.invoice_id.starts_with("inv_"));
		assert!(!create_response.commitment.is_empty());

		// Get invoice details
		let get_response: GetInvoiceDetailsResponse = client
			.request(
				"omni_getInvoiceDetails",
				rpc_params![create_response.invoice_id.clone(), None::<String>],
			)
			.await
			.unwrap();

		assert_eq!(get_response.invoice_id, create_response.invoice_id);
		assert_eq!(get_response.amount, Some("50000000000".to_string())); // Raw token units
		assert_eq!(get_response.token_address, "0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d");
		assert_eq!(get_response.chain_id, 421614);
		assert!(matches!(get_response.status, InvoiceStatus::Pending));
	}
}
