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
use crate::detailed_error::DetailedError;
use crate::error_code::{AUTH_VERIFICATION_FAILED_CODE, PARSE_ERROR_CODE};
use crate::methods::omni::common::check_auth;
use crate::methods::omni::PumpxRpcError;
use crate::server::RpcContext;
use crate::validation_helpers::{
	validate_chain_id, validate_omni_account_hex, validate_omni_account_length,
	validate_user_operations, validate_wallet_index,
};
use executor_core::intent_executor::IntentExecutor;
use executor_core::native_task::{NativeTask, NativeTaskWrapper};
use executor_core::types::SerializablePackedUserOperation;
use executor_primitives::{AccountId, ChainId};
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use parity_scale_codec::Decode;
use serde::{Deserialize, Serialize};
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

pub fn register_submit_user_op<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	
>(
	module: &mut RpcModule<
		RpcContext<EthereumIntentExecutor, SolanaIntentExecutor>,
	>,
) {
	module
		.register_async_method("omni_submitUserOp", |params, ctx, ext| async move {
			let user = check_auth(&ext).map_err(|e| {
				error!("Authentication check failed: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(AUTH_VERIFICATION_FAILED_CODE, "Authentication failed")
						.with_suggestion("Please provide valid authentication credentials"),
				)
			})?;

			let params = params.parse::<SubmitUserOpParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Failed to parse request parameters")
						.with_suggestion(format!("Invalid JSON structure: {}", e)),
				)
			})?;

			debug!("Received omni_submitUserOp, params: {:?}", params);

			validate_chain_id(params.chain_id as u32, Some("evm")).map_err(PumpxRpcError::from)?;

			validate_wallet_index(params.wallet_index).map_err(PumpxRpcError::from)?;

			validate_user_operations(&params.user_operations).map_err(PumpxRpcError::from)?;

			let address_bytes = validate_omni_account_hex(&user.omni_account, "omni_account")
				.map_err(PumpxRpcError::from)?;

			validate_omni_account_length(&address_bytes, "omni_account")
				.map_err(PumpxRpcError::from)?;

			let wrapper = NativeTaskWrapper::new(
				NativeTask::SubmitUserOp(
					AccountId::decode(&mut &address_bytes[..]).map_err(|e| {
						error!("Failed to decode AccountId from bytes: {:?}", e);
						PumpxRpcError::from(DetailedError::account_parse_error(
							&user.omni_account,
							&format!("Failed to decode account: {:?}", e),
						))
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
					error!("Unexpected response type from native task handler");
					Err(DetailedError::unexpected_response_type("SubmitUserOp", "Unknown").into())
				},
			})
			.await
		})
		.expect("Failed to register omni_submitUserOp method");
}
