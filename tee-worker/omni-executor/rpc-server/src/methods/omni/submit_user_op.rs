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

use super::common::handle_omni_native_task;
use crate::error_code::AUTH_VERIFICATION_FAILED_CODE;
use crate::methods::omni::common::check_auth;
use crate::methods::omni::PumpxRpcError;
use crate::server::RpcContext;
use crate::ErrorCode;
use alloy::primitives::{Address, FixedBytes};
use executor_core::native_task::{NativeTask, NativeTaskWrapper};
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::{AccountId, Chain};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use parity_scale_codec::Decode;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct SubmitUserOpParams {
	pub user_operations: Vec<SerializablePackedUserOperation>,
	pub chain: Chain,
}

#[derive(Serialize, Clone)]
pub struct SubmitUserOpResponse {
	pub transaction_hash: Option<String>,
}

pub fn register_submit_user_op(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_submitUserOp", |params, ctx, ext| async move {
			let user = check_auth(&ext).map_err(|e| {
				error!("Authentication check failed: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ServerError(
					AUTH_VERIFICATION_FAILED_CODE,
				))
			})?;

			let params = params.parse::<SubmitUserOpParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!("Received omni_submitUserOp, params: {:?}", params);

			let address_bytes =
				hex::decode(user.omni_account.strip_prefix("0x").unwrap_or(&user.omni_account))
					.map_err(|_| {
						error!("Failed to decode omni account hex string");
						PumpxRpcError::from_error_code(ErrorCode::InternalError)
					})?;

			if address_bytes.len() != 32 {
				error!(
					"Invalid omni account length: expected 32 bytes, got {}",
					address_bytes.len()
				);
				return Err(PumpxRpcError::from_error_code(ErrorCode::InternalError));
			}

			let mut address = [0u8; 32];
			address.copy_from_slice(&address_bytes);

			// Validate that all UserOperations belong to the authenticated user
			// by comparing the OA (bytes32) stored in each contract with expected OA
			for (index, user_op) in params.user_operations.iter().enumerate() {
				let sender_address = Address::from(user_op.sender);
				if let Err(e) =
					validate_user_operation_ownership(&user, &sender_address, &params.chain, &ctx)
						.await
				{
					error!(
						"User operation {} ownership validation failed for sender {:?}",
						index, sender_address
					);
					return Err(e);
				}
			}

			let wrapper = NativeTaskWrapper::new(
				NativeTask::SubmitUserOp(
					AccountId::decode(&mut &address[..]).map_err(|_| {
						error!("Failed to decode AccountId from bytes");
						PumpxRpcError::from_error_code(ErrorCode::InternalError)
					})?,
					params.user_operations.clone(),
					params.chain.clone(),
				),
				None,
				None,
				user.client_id,
			);

			handle_omni_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::SubmitUserOp(transaction_hash) => {
					Ok(SubmitUserOpResponse { transaction_hash })
				},
				_ => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register omni_submitUserOp method");
}

async fn validate_user_operation_ownership(
	user: &crate::methods::omni::common::User,
	sender_address: &Address,
	chain: &Chain,
	ctx: &RpcContext,
) -> Result<(), PumpxRpcError> {
	// Calculate the expected OA from the user's identity
	let expected_oa_bytes = hex::decode(
		user.omni_account.strip_prefix("0x").unwrap_or(&user.omni_account),
	)
	.map_err(|_| {
		error!("Failed to decode omni account hex string");
		PumpxRpcError::from_error_code(ErrorCode::InternalError)
	})?;

	if expected_oa_bytes.len() != 32 {
		error!("Invalid omni account length: expected 32 bytes, got {}", expected_oa_bytes.len());
		return Err(PumpxRpcError::from_error_code(ErrorCode::InternalError));
	}

	let expected_oa = FixedBytes::<32>::from_slice(&expected_oa_bytes);

	// Get the chain ID from the chain enum
	let chain_id = match chain {
		Chain::Evm(id) => *id,
		_ => {
			error!("Unsupported chain type for omni account validation: {:?}", chain);
			return Err(PumpxRpcError::from_error_code(ErrorCode::InternalError));
		},
	};

	// Get the RPC client for the chain
	let Some(rpc_client) = ctx.rpc_clients.get(&chain_id) else {
		error!("No RPC client found for chain ID: {}", chain_id);
		return Err(PumpxRpcError::from_error_code(ErrorCode::InternalError));
	};

	// Create a new client with the specific wallet address
	let omni_client =
		aa_contracts_client::OmniAccountClient::new(*sender_address, rpc_client.clone());

	// Query the OmniWallet contract directly to get its stored OA
	let stored_oa = match omni_client.get_owner().await {
		Ok(oa) => oa,
		Err(e) => {
			error!("Failed to query OmniWallet owner: {:?}", e);
			return Err(PumpxRpcError::from_error_code(ErrorCode::InternalError));
		},
	};

	// Compare stored OA with expected OA
	if stored_oa != expected_oa {
		error!(
			"OA mismatch: contract has 0x{}, expected 0x{}",
			hex::encode(stored_oa.as_slice()),
			hex::encode(expected_oa.as_slice())
		);
		return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(-32010)));
	}

	Ok(())
}
