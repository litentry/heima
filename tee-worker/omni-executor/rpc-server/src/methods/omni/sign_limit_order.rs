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
use crate::error_code::{AUTH_VERIFICATION_FAILED_CODE, INTERNAL_ERROR_CODE, PARSE_ERROR_CODE};
use crate::methods::omni::common::check_auth;
use crate::methods::omni::PumpxRpcError;
use crate::server::RpcContext;
use ethers::types::Bytes;
use executor_core::intent_executor::IntentExecutor;
use executor_core::native_task::PumpxChainId;
use executor_core::native_task::PumxWalletIndex;
use executor_primitives::{utils::hex::FromHexPrefixed, AccountId};
use heima_primitives::Address32;
use heima_primitives::IntentId;
use jsonrpsee::RpcModule;
use native_task_handler::{handle_pumpx_sign_limit_order, NativeTaskError, NativeTaskOk};
use serde::Deserialize;
use serde::Serialize;
use tracing::{debug, error};

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

pub fn register_sign_limit_order_params<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<EthereumIntentExecutor, SolanaIntentExecutor, CrossChainIntentExecutor>,
	>,
) {
	module
		.register_async_method("omni_signLimitOrder", |params, ctx, ext| async move {
			let user = check_auth(&ext).map_err(|e| {
				error!("Authentication check failed: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(
						AUTH_VERIFICATION_FAILED_CODE,
						"Authentication verification failed",
					)
					.with_suggestion("Please check your authentication credentials"),
				)
			})?;

			let params = params.parse::<SignLimitOrderParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from(
					DetailedError::new(PARSE_ERROR_CODE, "Parse error")
						.with_reason("Invalid JSON format or missing required fields"),
				)
			})?;

			debug!("Received omni_signLimitOrder, params: {:?}", params);

			let Ok(address) = Address32::from_hex(&user.omni_account) else {
				error!("Failed to parse from omni account token");
				return Err(PumpxRpcError::from(
					DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
						.with_reason("Failed to parse omni account from authentication token"),
				));
			};
			let omni_account = AccountId::from(address);

			let result = handle_pumpx_sign_limit_order(
				ctx.to_task_handler_context(),
				omni_account,
				params.chain_id,
				params.wallet_index,
				params.unsigned_tx.iter().map(|tx| tx.to_vec()).collect(),
				user.client_id,
			)
			.await;

			match result {
				Ok(NativeTaskOk::PumpxSignLimitOrder(signed_txs)) => Ok(SignLimitOrderResponse {
					intent_id: params.intent_id,
					order_id: params.order_id,
					chain_id: params.chain_id,
					signed_tx: signed_txs.into_iter().map(Bytes::from).collect(),
				}),
				Ok(_) => {
					error!("Unexpected response type");
					Err(PumpxRpcError::from(
						DetailedError::new(INTERNAL_ERROR_CODE, "Internal error")
							.with_reason("Unexpected response type from native task handler"),
					))
				},
				Err(NativeTaskError::ChainNotSupported(chain_id)) => {
					error!("Chain not supported: {}", chain_id);
					Err(PumpxRpcError::from(
						DetailedError::new(
							crate::error_code::INVALID_CHAIN_ID_CODE,
							"Chain not supported",
						)
						.with_reason(format!("Chain ID {} is not supported", chain_id)),
					))
				},
				Err(NativeTaskError::SignatureServiceUnavailable) => {
					error!("Signature service unavailable");
					Err(PumpxRpcError::from(
						DetailedError::new(
							crate::error_code::SIGNATURE_SERVICE_UNAVAILABLE_CODE,
							"Signature service unavailable",
						)
						.with_suggestion("Please try again later"),
					))
				},
				Err(NativeTaskError::PumpxSignerError(signer_error)) => {
					error!("Pumpx signer error: {:?}", signer_error);
					Err(PumpxRpcError::from(
						DetailedError::new(
							crate::error_code::PUMPX_SIGNER_REQUEST_SIGNATURE_FAILED_CODE,
							"Failed to sign transactions",
						)
						.with_reason(format!("{:?}", signer_error)),
					))
				},
				Err(e) => {
					error!("Failed to sign limit order: {:?}", e);
					Err(PumpxRpcError::from(
						DetailedError::new(INTERNAL_ERROR_CODE, "Failed to sign limit order")
							.with_reason(format!("{:?}", e)),
					))
				},
			}
		})
		.expect("Failed to register omni_signLimitOrder method");
}
