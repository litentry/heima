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
pub struct SettleUserOpParams {
	pub user_operation: SerializablePackedUserOperation,
	pub chain_id: ChainId,
}

#[derive(Serialize, Clone)]
pub struct SettleUserOpResponse {
	pub transaction_hash: Option<String>,
	pub message: String,
}

/// Settle a UserOperation by submitting it to the chain
/// This is used in the x402 flow after verification succeeds
pub fn register_settle_user_op<CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_settleUserOp", |params, ctx, _ext| async move {
			let params = parse_rpc_params::<SettleUserOpParams>(params)?;

			debug!("Received omni_settleUserOp, params: {:?}", params);

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
				error!("UserOp settlement failed: signature is required");
				return Err(DetailedError::invalid_user_op("Signature is required for settlement")
					.to_rpc_error());
			}

			info!(
				"Settling UserOp: sender={}, nonce={}, on chain_id={}",
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

			// Step 1: Run simulation first
			info!("Running simulation before settlement");
			match entry_point_client.simulate_handle_ops(&[aa_user_op.clone()], beneficiary).await
			{
				Ok(simulation_results) => {
					if let Some(result) = simulation_results.first() {
						info!(
							"UserOp simulation successful: preOpGas={}, paid={}, accountValidationData={}, paymasterValidationData={}",
							result.preOpGas,
							result.paid,
							result.accountValidationData,
							result.paymasterValidationData
						);
					}
				},
				Err(e) => {
					let err_msg = format!("UserOp simulation failed before settlement: {}", e);
					error!("{}", err_msg);
					return Err(DetailedError::invalid_user_op(&err_msg).to_rpc_error());
				},
			}

			// Step 2: Submit to chain with retry logic
			info!("Submitting UserOp to chain");
			match entry_point_client
				.handle_ops_with_retry(&[aa_user_op], beneficiary)
				.await
			{
				Ok(transaction_hash) => {
					info!("UserOp settlement successful, tx_hash: {}", transaction_hash);
					Ok::<SettleUserOpResponse, ErrorObjectOwned>(SettleUserOpResponse {
						transaction_hash: Some(transaction_hash.clone()),
						message: format!(
							"UserOperation settled successfully, transaction hash: {}",
							transaction_hash
						),
					})
				},
				Err(_) => {
					let err_msg = format!("Failed to submit UserOp to chain");
					error!("{}", err_msg);
					Err(DetailedError::internal_error(&err_msg).to_rpc_error())
				},
			}
		})
		.expect("Failed to register omni_settleUserOp method");
}
