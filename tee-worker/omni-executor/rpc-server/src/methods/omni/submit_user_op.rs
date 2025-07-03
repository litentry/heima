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
use executor_primitives::{utils::hex::FromHexPrefixed, AccountId, Chain};
use heima_primitives::Address32;
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct SubmitUserOpParams {
	pub user_operations: Vec<SerializablePackedUserOperation>,
	pub chain: Chain,
}

#[derive(Serialize, Clone)]
pub struct SubmitUserOpResponse {
	pub user_op_hashes: Vec<String>,
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

			let Ok(address) = Address32::from_hex(&user.omni_account) else {
				error!("Failed to parse from omni account token");
				return Err(PumpxRpcError::from_error_code(ErrorCode::InternalError));
			};

			// Validate that all UserOperations belong to the authenticated user
			let expected_omni_address = calculate_expected_omni_address(&user, &ctx).await?;
			let expected_address: [u8; 20] = expected_omni_address.into();

			for (index, user_op) in params.user_operations.iter().enumerate() {
				let sender_address: [u8; 20] = user_op.sender;
				if sender_address != expected_address {
					error!(
						"User operation {} sender mismatch: expected {:?}, got {:?}",
						index, expected_omni_address, sender_address
					);
					return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(-32010)));
					// Unauthorized sender
				}
			}

			let wrapper = NativeTaskWrapper::new(
				NativeTask::OmniSubmitUserOp(
					AccountId::from(address),
					params.user_operations.clone(),
					params.chain.clone(),
				),
				None,
				None,
				user.client_id,
			);

			handle_omni_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::OmniSubmitUserOp(user_op_hashes, transaction_hash) => {
					Ok(SubmitUserOpResponse { user_op_hashes, transaction_hash })
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

async fn calculate_expected_omni_address(
	user: &crate::methods::omni::common::User,
	ctx: &RpcContext,
) -> Result<Address, PumpxRpcError> {
	// Parse factory and implementation addresses from configuration
	let factory_address = ctx.omni_factory_address.parse::<Address>().map_err(|_| {
		error!("Failed to parse factory address from configuration");
		PumpxRpcError::from_error_code(ErrorCode::InternalError)
	})?;

	let implementation_address =
		ctx.omni_implementation_address.parse::<Address>().map_err(|_| {
			error!("Failed to parse implementation address from configuration");
			PumpxRpcError::from_error_code(ErrorCode::InternalError)
		})?;

	let root_address = Address::ZERO; // TODO: Get from configuration or derive from user

	let Ok(address) = Address32::from_hex(&user.omni_account) else {
		error!("Failed to parse omni account address");
		return Err(PumpxRpcError::from_error_code(ErrorCode::InternalError));
	};

	let omni_account_bytes: FixedBytes<32> = FixedBytes::from_slice(address.as_ref());
	let client_id_bytes = user.client_id.as_bytes();

	let calculated_address = aa_contracts_client::calculate_omni_account_address(
		factory_address,
		implementation_address,
		omni_account_bytes,
		client_id_bytes,
		root_address,
	);

	Ok(calculated_address)
}
