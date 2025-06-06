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

use crate::error_code::AUTH_VERIFICATION_FAILED_CODE;
use crate::methods::omni::common::check_auth;
use crate::methods::omni::PumpxRpcError;
use crate::server::RpcContext;
use crate::ErrorCode;
use ethers::types::Bytes;
use executor_core::native_task::NativeTask;
use executor_core::native_task::NativeTaskWrapper;
use executor_core::native_task::PumpxChainId;
use executor_core::native_task::PumxWalletIndex;
use executor_primitives::utils::hex::FromHexPrefixed;
use heima_primitives::Address32;
use heima_primitives::Identity;
use heima_primitives::IntentId;
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use serde::Deserialize;
use serde::Serialize;
use tracing::{debug, error};

use super::common::handle_omni_native_task;

#[derive(Debug, Deserialize)]
pub struct SignLimitOrderParams {
	pub intent_id: IntentId,
	pub order_id: u32,
	pub chain_id: PumpxChainId,
	pub wallet_index: PumxWalletIndex,
	pub unsigned_tx: Vec<Bytes>,
}

#[derive(Serialize, Clone)]
pub struct SignLimitOrderResponse {
	pub intent_id: IntentId,
	pub order_id: u32,
	pub chain_id: PumpxChainId,
	pub signed_tx: Vec<Bytes>,
}

pub fn register_sign_limit_order_params(module: &mut RpcModule<RpcContext>) {
	module
		.register_async_method("omni_signLimitOrder", |params, ctx, ext| async move {
           	let user = check_auth(&ext).map_err(|e| {
				error!("Authentication check failed: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ServerError(
					AUTH_VERIFICATION_FAILED_CODE,
				))
			})?;

			let params = params.parse::<SignLimitOrderParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			debug!("Received omni_signLimitOrder, intent_id: {}, order_id: {}, chain_id: {}, wallet_index: {}", params.intent_id, params.order_id, params.chain_id, params.wallet_index);

			let Ok(address) = Address32::from_hex(&user.omni_account) else {
				error!("Failed to parse from omni account token");
				return Err(PumpxRpcError::from_error_code(ErrorCode::InternalError));
			};

			let wrapper = NativeTaskWrapper::new(
				NativeTask::PumpxSignLimitOrder(
					Identity::Substrate(address),
					params.chain_id,
					params.wallet_index,
					params.unsigned_tx.iter().map(|tx| tx.to_vec()).collect(),
				),
		     	None,
				None,
			);

			handle_omni_native_task(&ctx, wrapper, |task_ok| match task_ok {
				NativeTaskOk::PumpxSignLimitOrder(signed_txs) => Ok(SignLimitOrderResponse {
					intent_id: params.intent_id,
					order_id: params.order_id,
					chain_id: params.chain_id,
					signed_tx: signed_txs.into_iter().map(Bytes::from).collect(),
				}),
				_ => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from_error_code(ErrorCode::InternalError))
				},
			})
			.await
		})
		.expect("Failed to register omni_signLimitOrder method");
}
