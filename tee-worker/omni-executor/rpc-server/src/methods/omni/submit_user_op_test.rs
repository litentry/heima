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
use crate::server::RpcContext;
use crate::utils::user_op::submit_user_ops;
use alloy::primitives::Address;
use executor_core::intent_executor::IntentExecutor;
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::{AccountId, ChainId};
use jsonrpsee::RpcModule;
use parity_scale_codec::Decode;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct SubmitUserOpTestParams {
	pub user_operations: Vec<SerializablePackedUserOperation>,
	pub chain_id: ChainId,
	pub wallet_index: u32,
	pub omni_account: String,
	#[allow(dead_code)]
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
				DetailedError::new(PARSE_ERROR_CODE, "Parse error")
					.with_reason("Invalid JSON format or missing required fields")
					.to_rpc_error()
			})?;

			debug!("Received omni_submitUserOpTest, params: {:?}", params);

			let address_bytes =
				hex::decode(params.omni_account.strip_prefix("0x").unwrap_or(&params.omni_account))
					.map_err(|_| {
						error!("Failed to decode omni account hex string");
						DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
							.with_reason("Failed to decode omni account hex string")
							.to_rpc_error()
					})?;

			if address_bytes.len() != 32 {
				error!(
					"Invalid omni account length: expected 32 bytes, got {}",
					address_bytes.len()
				);
				return Err(DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason(format!(
						"Invalid omni account length: expected 32 bytes, got {}",
						address_bytes.len()
					))
					.to_rpc_error());
			}

			for op in &params.user_operations {
				op.sender.parse::<Address>().map_err(|e| {
					error!("Invalid sender address '{}': {}", op.sender, e);
					DetailedError::new(PARSE_ERROR_CODE, "Parse error")
						.with_field("sender")
						.with_reason(format!("Invalid sender address '{}': {}", op.sender, e))
						.to_rpc_error()
				})?;
			}

			let omni_account = AccountId::decode(&mut &address_bytes[..]).map_err(|_| {
				error!("Failed to decode AccountId from bytes");
				DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
					.with_reason("Failed to decode AccountId from bytes")
					.to_rpc_error()
			})?;

			// Call the common submission logic
			let transaction_hash = submit_user_ops(
				&ctx,
				params.user_operations,
				params.chain_id,
				params.wallet_index,
				&omni_account,
			)
			.await?;

			Ok(SubmitUserOpTestResponse { transaction_hash })
		})
		.expect("Failed to register omni_submitUserOpTest method");
}
