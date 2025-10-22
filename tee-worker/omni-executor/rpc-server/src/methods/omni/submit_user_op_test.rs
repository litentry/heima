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
use crate::error_code::{INTERNAL_ERROR_CODE, PARSE_ERROR_CODE};
use crate::methods::omni::PumpxRpcError;
use crate::server::RpcContext;
use crate::utils::paymaster::{
	extract_paymaster_address, is_whitelisted_paymaster, parse_whitelisted_paymasters,
	process_erc20_paymaster_data,
};
use crate::utils::user_op::{convert_to_packed_user_op, substrate_to_ethereum_signature};
use aa_contracts_client::calculate_user_operation_hash;
use alloy::primitives::{Address, Bytes};
use binance_api::BinancePaymasterApi;
use executor_core::intent_executor::IntentExecutor;
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::{AccountId, ChainId};
use jsonrpsee::RpcModule;
use parity_scale_codec::Decode;
use serde::{Deserialize, Serialize};
use signer_client::ChainType;
use tracing::{debug, error, info};

#[derive(Debug, Deserialize)]
pub struct SubmitUserOpTestParams {
	pub user_operations: Vec<SerializablePackedUserOperation>,
	pub chain_id: ChainId,
	pub wallet_index: u32,
	pub omni_account: String,
	pub client_id: String,
}

#[derive(Serialize, Clone)]
pub struct SubmitUserOpTestResponse {
	pub transaction_hash: Option<String>,
}

pub fn register_submit_user_op_test<
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_submitUserOpTest", |params, ctx, _ext| async move {
			let params = params.parse::<SubmitUserOpTestParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Parse error")
						.with_reason("Invalid JSON format or missing required fields"),
				)
			})?;

			debug!("Received omni_submitUserOpTest, params: {:?}", params);

			let address_bytes =
				hex::decode(params.omni_account.strip_prefix("0x").unwrap_or(&params.omni_account))
					.map_err(|_| {
						error!("Failed to decode omni account hex string");
						PumpxRpcError::from(
							DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
								.with_reason("Failed to decode omni account hex string"),
						)
					})?;

			if address_bytes.len() != 32 {
				error!(
					"Invalid omni account length: expected 32 bytes, got {}",
					address_bytes.len()
				);
				return Err(PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error").with_reason(format!(
						"Invalid omni account length: expected 32 bytes, got {}",
						address_bytes.len()
					)),
				));
			}

			for op in &params.user_operations {
				op.sender.parse::<Address>().map_err(|e| {
					error!("Invalid sender address '{}': {}", op.sender, e);
					PumpxRpcError::from(
						DetailedError::new(PARSE_ERROR_CODE, "Parse error")
							.with_field("sender")
							.with_reason(format!("Invalid sender address '{}': {}", op.sender, e)),
					)
				})?;
			}

			let omni_account = AccountId::decode(&mut &address_bytes[..]).map_err(|_| {
				error!("Failed to decode AccountId from bytes");
				PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason("Failed to decode AccountId from bytes"),
				)
			})?;

			// Inlined handler logic from handle_submit_user_op
			info!(
				"Processing SubmitUserOp for {} UserOperations on chain_id: {}",
				params.user_operations.len(),
				params.chain_id
			);

			// Get EntryPoint client for this chain (needed for both signing and submission)
			let entry_point_client = ctx.entry_point_clients.get(&params.chain_id).ok_or_else(|| {
				error!("No EntryPoint client configured for chain_id: {}", params.chain_id);
				PumpxRpcError::from(DetailedError::chain_not_supported(params.chain_id))
			})?;

			// Parse whitelisted paymasters once
			let whitelisted_paymaster = parse_whitelisted_paymasters();

			// Process each UserOperation in the batch
			let mut aa_user_ops = Vec::new();

			for (index, serializable_user_op) in params.user_operations.iter().enumerate() {
				// Convert SerializablePackedUserOperation to PackedUserOperation
				let mut packed_user_op = convert_to_packed_user_op(serializable_user_op.clone())
					.map_err(|e| {
						error!("Failed to convert UserOperation {}: {}", index, e);
						PumpxRpcError::from(DetailedError::invalid_user_operation_error(&format!(
							"Invalid user operation at index {}",
							index
						)))
					})?;

				// Check userOp signature status and validate paymaster usage
				if packed_user_op.signature.is_empty() {
					// UNSIGNED userOp: If paymaster specified, must be whitelisted
					if !packed_user_op.paymasterAndData.is_empty() {
						if let Some(paymaster_address) =
							extract_paymaster_address(&packed_user_op.paymasterAndData)
						{
							if !is_whitelisted_paymaster(&paymaster_address, &whitelisted_paymaster)
							{
								error!(
									"UserOperation {} uses non-whitelisted paymaster {}. Only whitelisted paymasters are allowed for unsigned userOps.",
									index, paymaster_address
								);
								return Err(PumpxRpcError::from(
									DetailedError::invalid_user_operation_error(&format!(
										"UserOperation at index {} uses non-whitelisted paymaster {}",
										index, paymaster_address
									)),
								));
							}
						}

						match process_erc20_paymaster_data(
							ctx.binance_api_client.as_ref() as &dyn BinancePaymasterApi,
							&packed_user_op.paymasterAndData,
							params.chain_id,
						)
						.await
						{
							Ok(Some(updated_paymaster_data)) => {
								packed_user_op.paymasterAndData = updated_paymaster_data;
								info!("Updated ERC20 paymaster data for UserOperation {}", index);
							},
							Ok(None) => {
								// Not an ERC20 paymaster, continue as normal
								debug!("UserOperation {} does not use ERC20 paymaster", index);
							},
							Err(e) => {
								error!(
									"Failed to process ERC20 paymaster data for UserOperation {}: {}",
									index, e
								);
								return Err(PumpxRpcError::from(
									DetailedError::invalid_user_operation_error(&format!(
										"ERC20 paymaster processing failed for operation at index {}: {}",
										index, e
									)),
								));
							},
						}
					}

					info!("Requesting signature from pumpx signer for UserOperation {}", index);

					// Log UserOp details for debugging
					info!(
						"UserOp details - Sender: {}, Nonce: {}, InitCode length: {}, CallData length: {}",
						packed_user_op.sender,
						packed_user_op.nonce,
						packed_user_op.initCode.len(),
						packed_user_op.callData.len()
					);

					let entry_point_address = entry_point_client.entry_point_address();

					let user_op_hash_bytes = calculate_user_operation_hash(
						&packed_user_op,
						entry_point_address,
						params.chain_id,
					);
					let message_to_sign = user_op_hash_bytes.to_vec();

					info!(
						"Signing UserOp hash: 0x{}, EntryPoint: {}, ChainID: {}",
						hex::encode(user_op_hash_bytes),
						entry_point_address,
						params.chain_id
					);

					// Request signature from pumpx signer for EVM chain
					let signature_result = ctx
						.signer_client
						.request_signature(
							ChainType::Evm,
							params.wallet_index,
							omni_account.clone().into(),
							message_to_sign,
						)
						.await;

					let signature = match signature_result {
						Ok(sig) => substrate_to_ethereum_signature(&sig)
							.map_err(|e| {
								error!("Failed to convert signature: {}", e);
								PumpxRpcError::from(DetailedError::signature_service_unavailable())
							})?
							.to_vec(),
						Err(_) => {
							error!("Failed to sign user operation {}", index);
							return Err(PumpxRpcError::from(
								DetailedError::signature_service_unavailable(),
							));
						},
					};

					// Prepend 0x01 byte to indicate Root signature type (according to UserOpSigner enum)
					let mut signature_with_prefix: Vec<u8> = vec![0x01];
					signature_with_prefix.extend_from_slice(&signature);
					packed_user_op.signature = Bytes::from(signature_with_prefix);
					info!("UserOperation {} signed successfully", index);
				} else {
					// SIGNED userOp: Only allowed if no paymaster specified
					if !packed_user_op.paymasterAndData.is_empty() {
						error!(
							"UserOperation {} is signed but has paymaster data. Signed userOps are only allowed without paymaster.",
							index
						);
						return Err(PumpxRpcError::from(
							DetailedError::invalid_user_operation_error(&format!(
								"UserOperation at index {} is signed but specifies a paymaster",
								index
							)),
						));
					}
					info!("UserOperation {} is signed with no paymaster, processing", index);
				}

				// Convert to aa_contracts_client::PackedUserOperation for EntryPoint call
				let aa_user_op = aa_contracts_client::PackedUserOperation {
					sender: packed_user_op.sender,
					nonce: packed_user_op.nonce,
					initCode: packed_user_op.initCode.clone(),
					callData: packed_user_op.callData.clone(),
					accountGasLimits: packed_user_op.accountGasLimits,
					preVerificationGas: packed_user_op.preVerificationGas,
					gasFees: packed_user_op.gasFees,
					paymasterAndData: packed_user_op.paymasterAndData.clone(),
					signature: packed_user_op.signature.clone(),
				};
				aa_user_ops.push(aa_user_op);
			}

			// Get beneficiary address from the EntryPoint client's wallet
			let beneficiary = entry_point_client.get_wallet_address().await.map_err(|_| {
				let err_msg = "Failed to get wallet address from EntryPoint client".to_string();
				error!("{}", err_msg.clone());
				PumpxRpcError::from_code_and_message(
					crate::error_code::INTERNAL_ERROR_CODE,
					err_msg,
				)
			})?;

			// Run batch simulation for all UserOperations before submission
			info!("Running batch simulation for {} UserOperations", aa_user_ops.len());
			match entry_point_client.simulate_handle_ops(&aa_user_ops, beneficiary).await {
				Ok(simulation_results) => {
					for (index, result) in simulation_results.iter().enumerate() {
						info!(
							"UserOperation {} simulation successful. PreOpGas: {}, Paid: {}, AccountValidation: {}, PaymasterValidation: {}",
							index,
							result.preOpGas,
							result.paid,
							result.accountValidationData,
							result.paymasterValidationData
						);
					}
					info!("All {} UserOperations passed batch simulation checks", aa_user_ops.len());
				},
				Err(e) => {
					let err_msg: String = format!("Batch UserOperation simulation failed: {}", e);
					error!("{}", err_msg.clone());
					return Err(PumpxRpcError::from(
						DetailedError::invalid_user_operation_error(&err_msg),
					));
				},
			}

			// Submit all UserOperations via EntryPoint.handleOps() with retry logic
			let transaction_hash =
				match entry_point_client.handle_ops_with_retry(&aa_user_ops, beneficiary).await {
					Ok(tx_hash) => {
						// Return the actual transaction hash from handle_ops
						Some(tx_hash)
					},
					Err(_) => {
						let err_msg =
							"Failed to submit UserOperations to EntryPoint via handleOps after retries"
								.to_string();
						error!("{}", err_msg.clone());
						return Err(PumpxRpcError::from_code_and_message(
							crate::error_code::INTERNAL_ERROR_CODE,
							err_msg,
						));
					},
				};

			Ok(SubmitUserOpTestResponse { transaction_hash })
		})
		.expect("Failed to register omni_submitUserOpTest method");
}
