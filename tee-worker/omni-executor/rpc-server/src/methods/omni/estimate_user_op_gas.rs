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
use alloy::primitives::Address;
use executor_core::native_task::{NativeTask, NativeTaskWrapper};
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::{AccountId, ChainId};
// use jsonrpsee::types::ErrorObject;
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use parity_scale_codec::Decode;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

const OMNI_ACCOUNT_BYTES_LENGTH: usize = 32;

#[derive(Debug, Deserialize)]
pub struct EstimateUserOpGasParams {
	pub user_operation: SerializablePackedUserOperation,
	pub chain_id: ChainId,
	pub wallet_index: u32,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EstimateUserOpGasResponse {
	pub call_gas_limit: String,
	pub verification_gas_limit: String,
	pub pre_verification_gas: String,
	pub paymaster_verification_gas_limit: String,
	pub paymaster_post_op_gas_limit: String,
}

pub fn register_estimate_user_op_gas(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_estimateUserOpGas", |params, ctx, ext| async move {
			let user = check_auth(&ext).map_err(|e| {
				error!("Authentication check failed: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ServerError(
					AUTH_VERIFICATION_FAILED_CODE,
				))
			})?;

			let params = params.parse::<EstimateUserOpGasParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!("Received omni_estimateUserOpGas, params: {:?}", params);

			let account_id =
				decode_account_id(&user.omni_account).map_err(PumpxRpcError::from_error_code)?;

			validate_sender_address(&params.user_operation.sender)
				.map_err(PumpxRpcError::from_error_code)?;

			let wrapper = NativeTaskWrapper::new(
				NativeTask::EstimateUserOpGas(
					account_id,
					params.user_operation.clone(),
					params.chain_id,
					params.wallet_index,
				),
				None,
				None,
				user.client_id,
			);

			handle_omni_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::EstimateUserOpGas {
					call_gas_limit,
					verification_gas_limit,
					pre_verification_gas,
					paymaster_verification_gas_limit,
					paymaster_post_op_gas_limit,
				} => Ok(EstimateUserOpGasResponse {
					call_gas_limit: call_gas_limit.to_string(),
					verification_gas_limit: verification_gas_limit.to_string(),
					pre_verification_gas: pre_verification_gas.to_string(),
					paymaster_verification_gas_limit: paymaster_verification_gas_limit.to_string(),
					paymaster_post_op_gas_limit: paymaster_post_op_gas_limit.to_string(),
				}),
				_ => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register omni_estimateUserOpGas method");
}

fn decode_account_id(hex_str: &str) -> Result<AccountId, ErrorCode> {
	let bytes = hex::decode(hex_str.strip_prefix("0x").unwrap_or(hex_str)).map_err(|e| {
		error!("Failed to decode omni account hex string: {}", e);
		ErrorCode::ParseError
	})?;

	if bytes.len() != OMNI_ACCOUNT_BYTES_LENGTH {
		error!(
			"Invalid omni account length: expected {} bytes, got {}",
			OMNI_ACCOUNT_BYTES_LENGTH,
			bytes.len()
		);
		return Err(ErrorCode::ParseError);
	}

	AccountId::decode(&mut &bytes[..]).map_err(|e| {
		error!("Failed to decode AccountId from bytes: {}", e);
		ErrorCode::ParseError
	})
}

fn validate_sender_address(sender: &str) -> Result<Address, ErrorCode> {
	sender.parse::<Address>().map_err(|e| {
		error!("Invalid sender address '{}': {}", sender, e);
		ErrorCode::ParseError
	})
}
