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
use executor_core::intent_executor::IntentExecutor;
use executor_core::types::SerializablePackedUserOperation;
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
			match entry_point_client.simulate_handle_ops(&[aa_user_op], beneficiary).await {
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
