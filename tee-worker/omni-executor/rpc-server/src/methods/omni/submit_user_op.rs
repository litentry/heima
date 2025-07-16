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
use executor_primitives::{AccountId, ChainId};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use parity_scale_codec::Decode;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct SubmitUserOpParams {
	pub user_operations: Vec<SerializablePackedUserOperation>,
	pub chain_id: ChainId,
	pub wallet_index: u32,
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

			// Collect unique sender addresses to avoid redundant validation calls
			let unique_addresses: HashSet<Address> = params
				.user_operations
				.iter()
				.map(|op| {
					op.sender.parse::<Address>().map_err(|e| {
						error!("Invalid sender address '{}': {}", op.sender, e);
						PumpxRpcError::from_error_code(ErrorCode::ParseError)
					})
				})
				.collect::<Result<HashSet<_>, _>>()?;

			let wrapper = NativeTaskWrapper::new(
				NativeTask::SubmitUserOp(
					AccountId::decode(&mut &address[..]).map_err(|_| {
						error!("Failed to decode AccountId from bytes");
						PumpxRpcError::from_error_code(ErrorCode::InternalError)
					})?,
					params.user_operations.clone(),
					params.chain_id,
					params.wallet_index,
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
