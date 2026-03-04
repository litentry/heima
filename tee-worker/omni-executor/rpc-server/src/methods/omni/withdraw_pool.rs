use crate::detailed_error::DetailedError;
use crate::server::RpcContext;
use crate::utils::types::RpcResultExt;
use crate::utils::validation::parse_rpc_params;
use ethers::abi::{encode, Token};
use ethers::types::{Address, U256};
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::RpcModule;
use oe_core::intent::executor::IntentExecutor;
use oe_crypto::confidential::decrypt_amount;
use oe_crypto::privacy_pool::generate_nullifier;
use oe_storage::confidential_invoice::InvoiceStatus;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{debug, error, info};

/// Keccak256("withdraw(bytes32,uint256,address,bytes,uint256[3])")[0..4]
const WITHDRAW_SELECTOR: [u8; 4] = [0x6a, 0x8b, 0x00, 0xa6];

#[derive(Debug, Deserialize)]
pub struct WithdrawFromPoolParams {
	pub invoice_id: String,
	pub seller_address: String, // EOA or smart wallet that will receive the funds
	pub chain_id: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WithdrawFromPoolResponse {
	/// ABI-encoded calldata for SimplePrivacyPool.withdraw(...)
	pub calldata: String,
	/// Pool contract address (for the transaction destination)
	pub pool_address: String,
	/// Raw token amount in smallest units
	pub amount_raw: String,
	/// Hex-encoded nullifier
	pub nullifier_hex: String,
}

pub fn register_withdraw_from_pool<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_withdrawFromPool", |params, ctx, _ext| async move {
			let params = parse_rpc_params::<WithdrawFromPoolParams>(params)?;

			debug!(
				"Received omni_withdrawFromPool, invoice_id: {}, seller: {}, chain_id: {}",
				params.invoice_id, params.seller_address, params.chain_id
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

			// Only pending invoices can be withdrawn (on-chain nullifier guards double-spend)
			if invoice.status != InvoiceStatus::Pending {
				return Err(DetailedError::invalid_params(
					"invoice_id",
					"Invoice is not in pending status",
				)
				.to_rpc_error());
			}

			// Find the recipient entry that matches the calling seller address (case-insensitive)
			let seller_lower = params.seller_address.to_lowercase();
			let recipient = invoice
				.recipients
				.iter()
				.find(|r| r.address.to_lowercase() == seller_lower)
				.ok_or_else(|| {
					DetailedError::invalid_params(
						"seller_address",
						"Address is not a recipient of this invoice",
					)
					.to_rpc_error()
				})?;

			// Must have a pool secret — omni_payInvoice must have been called first
			let pool_secret = recipient.pool_secret.ok_or_else(|| {
				DetailedError::invalid_params(
					"invoice_id",
					"No pool deposit found for this recipient — call omni_payInvoice first",
				)
				.to_rpc_error()
			})?;

			// Decrypt this recipient's sub-amount
			let amount = decrypt_amount(&recipient.encrypted_amount, &ctx.aes256_key)
				.map_err_internal("Failed to decrypt recipient amount")?;

			let leaf_index = recipient.leaf_index.unwrap_or(0);
			let nullifier = generate_nullifier(&pool_secret, leaf_index);

			// Parse seller address
			let recipient_addr = Address::from_str(&params.seller_address).map_err(|_| {
				DetailedError::invalid_params("seller_address", "Invalid Ethereum address")
					.to_rpc_error()
			})?;

			// Build calldata: withdraw(nullifier, amount, recipient, proof=0x, pubSignals=[0,0,0])
			// MockVerifier accepts any proof; real ZK proof will be substituted in Phase 2.
			let calldata = build_withdraw_calldata(&nullifier, amount, recipient_addr);

			// Do NOT mark Paid here — the on-chain nullifier mapping is the authoritative
			// double-spend guard. Marking before tx confirmation leaves the invoice stuck
			// if the tx fails.

			info!(
				"Issued withdrawal calldata for invoice: {}, recipient: {}, nullifier: 0x{}",
				params.invoice_id,
				params.seller_address,
				hex::encode(nullifier)
			);

			Ok::<WithdrawFromPoolResponse, ErrorObjectOwned>(WithdrawFromPoolResponse {
				calldata,
				pool_address: ctx.privacy_pool_address.clone(),
				amount_raw: amount.to_string(),
				nullifier_hex: hex::encode(nullifier),
			})
		})
		.expect("Failed to register omni_withdrawFromPool");
}

/// ABI-encode withdraw(bytes32,uint256,address,bytes,uint256[3]) calldata.
/// proof is empty bytes, pubSignals are all zeros (mock verifier accepts anything).
fn build_withdraw_calldata(nullifier: &[u8; 32], amount: u128, recipient: Address) -> String {
	let encoded = encode(&[
		Token::FixedBytes(nullifier.to_vec()),
		Token::Uint(U256::from(amount)),
		Token::Address(recipient),
		Token::Bytes(vec![]),
		Token::FixedArray(vec![
			Token::Uint(U256::zero()),
			Token::Uint(U256::zero()),
			Token::Uint(U256::zero()),
		]),
	]);
	let mut calldata = WITHDRAW_SELECTOR.to_vec();
	calldata.extend_from_slice(&encoded);
	format!("0x{}", hex::encode(calldata))
}
