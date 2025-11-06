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
use crate::utils::user_op::convert_to_packed_user_op;
use crate::utils::validation::parse_rpc_params;
use aa_contracts_client::calculate_user_operation_hash;
use executor_core::intent_executor::IntentExecutor;
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::signature::recover_evm_address;
use executor_primitives::ChainId;
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::RpcModule;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

#[derive(Debug, Deserialize)]
pub struct VerifyUserOpParams {
	pub user_operation: SerializablePackedUserOperation,
	pub chain_id: ChainId,
}

#[derive(Serialize, Clone)]
pub struct VerifyUserOpResponse {
	pub valid: bool,
	pub message: String,
}

/// Verify a UserOperation by simulating it (without submitting to chain)
/// This is used in the x402 flow to verify payment before settlement
pub fn register_verify_user_op<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_verifyUserOp", |params, ctx, _ext| async move {
			let params = parse_rpc_params::<VerifyUserOpParams>(params)?;

			debug!("Received omni_verifyUserOp, params: {:?}", params);

			// Get EntryPoint client for this chain
			let entry_point_client =
				ctx.entry_point_clients.get(&params.chain_id).ok_or_else(|| {
					error!("No EntryPoint client configured for chain_id: {}", params.chain_id);
					DetailedError::invalid_chain_id(params.chain_id).to_rpc_error()
				})?;

			// Convert to PackedUserOperation
			let packed_user_op =
				convert_to_packed_user_op(params.user_operation.clone()).map_err(|e| {
					error!("Failed to convert UserOp: {}", e);
					DetailedError::invalid_user_op(&format!("Failed to convert UserOp: {}", e))
						.to_rpc_error()
				})?;

			// Check if signature is present
			if packed_user_op.signature.is_empty() {
				error!("UserOp verification failed: signature is required");
				return Ok::<VerifyUserOpResponse, ErrorObjectOwned>(VerifyUserOpResponse {
					valid: false,
					message: "Signature is required for verification".to_string(),
				});
			}

			info!(
				"Verifying UserOp: sender={}, nonce={}, on chain_id={}",
				packed_user_op.sender, packed_user_op.nonce, params.chain_id
			);

			// Convert to aa_contracts_client::PackedUserOperation
			let user_op = aa_contracts_client::PackedUserOperation {
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

			// Compute and log the userOp hash for debugging parity with frontend
			let entry_point_address = entry_point_client.entry_point_address();
			let user_op_hash = calculate_user_operation_hash(&user_op, entry_point_address, params.chain_id);
			info!("Computed userOpHash: 0x{}", hex::encode(user_op_hash));

			// If signature present, attempt an off-chain verification first (strip leading UserOpSigner byte)
			if !user_op.signature.is_empty() {
				let sig_bytes = user_op.signature.as_ref();
				if sig_bytes.len() < 1 {
					error!("Signature too short for UserOp");
					return Ok::<VerifyUserOpResponse, ErrorObjectOwned>(VerifyUserOpResponse {
						valid: false,
						message: "Signature too short".to_string(),
					});
				}

				let signer_type = sig_bytes[0];
				let sig_payload = &sig_bytes[1..];

				// Only implement Owner (0x00) and RootKey (0x01) pre-verification here
				if signer_type == 0x00 || signer_type == 0x01 {
					// Expect 65-byte ECDSA signature
					if sig_payload.len() != 65 {
						error!("Invalid ECDSA signature length: {}", sig_payload.len());
						return Ok::<VerifyUserOpResponse, ErrorObjectOwned>(VerifyUserOpResponse {
							valid: false,
							message: format!("Invalid ECDSA signature length: {}", sig_payload.len()),
						});
					}

					let mut sig_arr = [0u8; 65];
					sig_arr.copy_from_slice(&sig_payload[0..65]);
					// recover_evm_address expects msg (32 bytes) and signature (65 bytes)
					match recover_evm_address(&user_op_hash, &sig_arr) {
						Ok(recovered) => {
							let recovered_addr = alloy::primitives::Address::from_slice(&recovered);
							info!("Recovered signature address off-chain: {}", recovered_addr);
						}
						Err(_) => {
							error!("Failed to recover EVM address from signature");
							return Ok::<VerifyUserOpResponse, ErrorObjectOwned>(VerifyUserOpResponse {
								valid: false,
								message: "Failed to recover EVM address from signature".to_string(),
							});
						}
					}
				} else {
					info!("Signature type {} pre-verification not implemented, skipping off-chain check", signer_type);
				}
			}

			// Get beneficiary address
			let beneficiary = entry_point_client
				.get_wallet_address()
				.await
				.map_err(|_| {
					error!("Failed to get wallet address");
					DetailedError::internal_error("Failed to get wallet address")
						.to_rpc_error()
				})?;

			// Run simulation only (no actual submission)
			info!("Running simulation for UserOp verification");
			match entry_point_client.simulate_handle_ops(&[user_op], beneficiary).await {
				Ok(simulation_results) => {
					if let Some(result) = simulation_results.first() {
						info!(
							"UserOp verification successful: preOpGas={}, paid={}, accountValidationData={}, paymasterValidationData={}",
							result.preOpGas,
							result.paid,
							result.accountValidationData,
							result.paymasterValidationData
						);
					}
					Ok(VerifyUserOpResponse {
						valid: true,
						message: "UserOperation verification successful".to_string(),
					})
				},
				Err(e) => {
					let err_msg = format!("UserOp simulation failed: {}", e);
					error!("{}", err_msg);
					Ok(VerifyUserOpResponse {
						valid: false,
						message: err_msg,
					})
				},
			}
		})
		.expect("Failed to register omni_verifyUserOp method");
}
