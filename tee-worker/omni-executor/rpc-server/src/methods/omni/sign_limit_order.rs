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
use crate::error_code::{AUTH_VERIFICATION_FAILED_CODE, PARSE_ERROR_CODE};
use crate::methods::omni::check_auth;
use crate::server::RpcContext;
use crate::utils::omni::to_omni_account;
use ethers::types::Bytes;
use executor_core::intent_executor::IntentExecutor;
use executor_core::native_task::{PumpxChainId, PumxWalletIndex};
use heima_primitives::IntentId;
use jsonrpsee::RpcModule;
use pumpx::signer_client::PumpxChainId as _;
use serde::Deserialize;
use serde::Serialize;
use signer_client::ChainType;
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
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
>(
	module: &mut RpcModule<RpcContext<CrossChainIntentExecutor>>,
) {
	module
		.register_async_method("omni_signLimitOrder", |params, ctx, ext| async move {
			let oa_str = check_auth(&ext).map_err(|e| {
				error!("Authentication check failed: {:?}", e);
				DetailedError::new(
					AUTH_VERIFICATION_FAILED_CODE,
					"Authentication verification failed",
				)
				.with_suggestion("Please check your authentication credentials")
				.to_rpc_error()
			})?;

			let params = params.parse::<SignLimitOrderParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				DetailedError::new(PARSE_ERROR_CODE, "Parse error")
					.with_reason("Invalid JSON format or missing required fields")
					.to_rpc_error()
			})?;

			debug!("Received omni_signLimitOrder, params: {:?}", params);

			let omni_account = to_omni_account(&oa_str).map_err(|_| {
				DetailedError::new(PARSE_ERROR_CODE, "Failed to parse omni account").to_rpc_error()
			})?;

			// Inline handle_pumpx_sign_limit_order logic
			let Some(chain) = ChainType::from_pumpx_chain_id(params.chain_id) else {
				error!("Failed to map pumpx chain_id {}", params.chain_id);
				return Err(DetailedError::new(
					crate::error_code::INVALID_CHAIN_ID_CODE,
					"Chain not supported",
				)
				.with_reason(format!("Chain ID {} is not supported", params.chain_id))
				.to_rpc_error());
			};

			let unsigned_tx_vec: Vec<Vec<u8>> =
				params.unsigned_tx.iter().map(|tx| tx.to_vec()).collect();
			let Ok(signed_txs) = ctx
				.signer_client
				.request_signatures(
					chain,
					params.wallet_index,
					omni_account.into(),
					unsigned_tx_vec,
				)
				.await
			else {
				error!("Failed to request signatures from pumpx-signer");
				return Err(DetailedError::new(
					crate::error_code::SIGNATURE_SERVICE_UNAVAILABLE_CODE,
					"Signature service unavailable",
				)
				.with_suggestion("Please try again later")
				.to_rpc_error());
			};

			Ok(SignLimitOrderResponse {
				intent_id: params.intent_id,
				order_id: params.order_id,
				chain_id: params.chain_id,
				signed_tx: signed_txs.into_iter().map(Bytes::from).collect(),
			})
		})
		.expect("Failed to register omni_signLimitOrder method");
}
