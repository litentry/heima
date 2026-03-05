use crate::detailed_error::DetailedError;
use crate::server::RpcContext;
use crate::utils::types::RpcResultExt;
use crate::utils::validation::parse_rpc_params;
use ethers::abi::{encode, Token};
use ethers::types::U256;
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::RpcModule;
use oe_core::intent::executor::IntentExecutor;
use oe_crypto::confidential::decrypt_amount;
use oe_crypto::privacy_pool::{generate_pool_commitment, generate_secret};
use oe_storage::confidential_invoice::InvoiceStatus;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

/// Keccak256("deposit(uint256,uint256)")[0..4]
const DEPOSIT_SELECTOR: [u8; 4] = [0xe2, 0xbb, 0xb1, 0x58];

#[derive(Debug, Deserialize)]
pub struct PayInvoiceParams {
	pub invoice_id: String,
	pub buyer_smart_wallet: String, // Smart wallet address that will call deposit()
	pub chain_id: u64,
}

/// One deposit calldata entry — one per recipient.
#[derive(Debug, Clone, Serialize)]
pub struct DepositInfo {
	/// Hex-encoded commitment (bytes32) — for UI display
	pub commitment_hex: String,
	/// ABI-encoded calldata for SimplePrivacyPool.deposit(commitment, amount)
	pub calldata: String,
	/// Raw token amount for this recipient (smallest units)
	pub amount_raw: String,
	/// Recipient address (for informational display; not revealed on-chain)
	pub recipient_address: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PayInvoiceResponse {
	/// One deposit per recipient
	pub deposits: Vec<DepositInfo>,
	/// Pool contract address (destination for all deposit calls)
	pub pool_address: String,
	/// Total token amount across all recipients (for the approve call)
	pub total_amount_raw: String,
}

pub fn register_pay_invoice<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_payInvoice", |params, ctx, _ext| async move {
			let params = parse_rpc_params::<PayInvoiceParams>(params)?;

			debug!(
				"Received omni_payInvoice, invoice_id: {}, buyer: {}, chain_id: {}",
				params.invoice_id, params.buyer_smart_wallet, params.chain_id
			);

			// Load invoice
			let invoice = ctx
				.confidential_invoice_storage
				.get_by_id(&params.invoice_id)
				.map_err_internal("Failed to get invoice")?
				.ok_or_else(|| {
					error!("Invoice not found: {}", params.invoice_id);
					DetailedError::invalid_params("invoice_id", "Invoice not found")
				})?;

			// Only pending invoices can be paid
			if invoice.status != InvoiceStatus::Pending {
				return Err(DetailedError::invalid_params(
					"invoice_id",
					"Invoice is not in pending status",
				)
				.to_rpc_error());
			}

			// Decrypt total amount for the response
			let total_amount = decrypt_amount(&invoice.encrypted_amount, &ctx.aes256_key)
				.map_err_internal("Failed to decrypt total amount")?;

			// Build deposits for each recipient.
			// If a recipient already has a pool_commitment (browser retry), reuse it.
			let mut deposits: Vec<DepositInfo> = Vec::new();
			let mut any_new = false;

			for (idx, recipient) in invoice.recipients.iter().enumerate() {
				let sub_amount = decrypt_amount(&recipient.encrypted_amount, &ctx.aes256_key)
					.map_err_internal("Failed to decrypt recipient amount")?;

				let commitment = if let Some(existing) = recipient.pool_commitment {
					// Idempotent: buyer closed tab and retried — return existing commitment
					existing
				} else {
					// Generate a fresh secret + commitment for this recipient
					let secret =
						generate_secret().map_err_internal("Failed to generate pool secret")?;

					// Poseidon(secret, amount) — unique secret per recipient prevents
					// commitment collisions even when amounts are identical.
					let _ = idx; // no longer needed in hash input
					let commitment = generate_pool_commitment(&secret, sub_amount)
						.map_err_internal("Failed to compute commitment")?;

					// Persist secret + commitment for this recipient
					let recipient_address = recipient.address.clone();
					ctx.confidential_invoice_storage
						.update(&params.invoice_id, |inv| {
							if let Some(r) =
								inv.recipients.iter_mut().find(|r| r.address == recipient_address)
							{
								r.pool_commitment = Some(commitment);
								r.pool_secret = Some(secret);
							}
						})
						.map_err_internal("Failed to store recipient pool commitment")?;

					any_new = true;
					commitment
				};

				deposits.push(DepositInfo {
					commitment_hex: hex::encode(commitment),
					calldata: build_deposit_calldata(&commitment, sub_amount),
					amount_raw: sub_amount.to_string(),
					recipient_address: recipient.address.clone(),
				});
			}

			if any_new {
				info!(
					"Prepared pool deposits for invoice: {}, {} recipients, total: {}",
					params.invoice_id,
					deposits.len(),
					total_amount
				);
			} else {
				info!(
					"Returning existing pool deposits for invoice: {} (retry)",
					params.invoice_id
				);
			}

			Ok::<PayInvoiceResponse, ErrorObjectOwned>(PayInvoiceResponse {
				deposits,
				pool_address: ctx.privacy_pool_address.clone(),
				total_amount_raw: total_amount.to_string(),
			})
		})
		.expect("Failed to register omni_payInvoice");
}

/// ABI-encode deposit(uint256 commitment, uint256 amount) calldata.
fn build_deposit_calldata(commitment: &[u8; 32], amount: u128) -> String {
	let commitment_uint = U256::from_big_endian(commitment);
	let encoded = encode(&[Token::Uint(commitment_uint), Token::Uint(U256::from(amount))]);
	let mut calldata = DEPOSIT_SELECTOR.to_vec();
	calldata.extend_from_slice(&encoded);
	format!("0x{}", hex::encode(calldata))
}
