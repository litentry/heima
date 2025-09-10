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

use super::common::handle_pumpx_native_task;
use crate::error_code::AUTH_VERIFICATION_FAILED_CODE;
use crate::methods::pumpx::PumpxRpcError;
use crate::server::RpcContext;
use crate::verify_auth::verify_auth_token_authentication;
use crate::ErrorCode;
use ethers::types::Bytes;
use executor_core::intent_executor::IntentExecutor;
use executor_core::native_task::NativeTask;
use executor_core::native_task::NativeTaskWrapper;
use executor_core::native_task::PumpxChainId;
use executor_core::native_task::PumxWalletIndex;
use executor_primitives::OmniAuth;
use executor_primitives::{utils::hex::FromHexPrefixed, AccountId};
use heima_authentication::constants::AUTH_TOKEN_ACCESS_TYPE;
use heima_primitives::Address32;
use heima_primitives::IntentId;
use jsonrpsee::RpcModule;
use native_task_handler::NativeTaskOk;
use parentchain_rpc_client::{SubstrateRpcClient, SubstrateRpcClientFactory};
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct SignLimitOrderParams {
	pub intent_id: IntentId,
	pub order_id: u32,
	pub chain_id: PumpxChainId,
	pub wallet_index: PumxWalletIndex,
	pub unsigned_tx: Vec<Bytes>,
	pub auth_token: String,
}

#[derive(Serialize, Clone)]
pub struct SignLimitOrderResponse {
	pub intent_id: IntentId,
	pub order_id: u32,
	pub chain_id: PumpxChainId,
	pub signed_tx: Vec<Bytes>,
}

#[tracing::instrument(skip(ctx, params), fields(intent_id = %params.intent_id, order_id = %params.order_id, chain_id = %params.chain_id, wallet_index = %params.wallet_index, unsigned_tx_count = %params.unsigned_tx.len()))]
async fn handle_sign_limit_order_request<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	params: SignLimitOrderParams,
	ctx: Arc<
		RpcContext<
			Header,
			RpcClient,
			RpcClientFactory,
			EthereumIntentExecutor,
			SolanaIntentExecutor,
			CrossChainIntentExecutor,
		>,
	>,
) -> Result<SignLimitOrderResponse, PumpxRpcError> {
	debug!("Processing pumpx_signLimitOrder request");

	let (omni_account, client_id) = match verify_auth_token_authentication(
		&ctx.jwt_rsa_private_key,
		&params.auth_token,
		AUTH_TOKEN_ACCESS_TYPE,
		true,
	) {
		Ok(claims) => (claims.sub, claims.aud),
		Err(_) => {
			error!("Failed to verify auth token");
			return Err(PumpxRpcError::from_error_code(ErrorCode::ServerError(
				AUTH_VERIFICATION_FAILED_CODE,
			)));
		},
	};

	let Ok(address) = Address32::from_hex(&omni_account) else {
		error!("Failed to parse from omni account token");
		return Err(PumpxRpcError::from_error_code(ErrorCode::InternalError));
	};

	let wrapper = NativeTaskWrapper::new(
		NativeTask::PumpxSignLimitOrder(
			AccountId::from(address),
			params.chain_id,
			params.wallet_index,
			params.unsigned_tx.iter().map(|tx| tx.to_vec()).collect(),
		),
		None,
		Some(OmniAuth::AuthToken(params.auth_token)),
		client_id,
	);

	handle_pumpx_native_task(&ctx, wrapper, |task_ok| match task_ok {
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
}

pub fn register_sign_limit_order_params<
	EthereumIntentExecutor: IntentExecutor + Send + Sync + 'static,
	SolanaIntentExecutor: IntentExecutor + Send + Sync + 'static,
	CrossChainIntentExecutor: IntentExecutor + Send + Sync + 'static,
	Header: Send + Sync + 'static,
	RpcClient: SubstrateRpcClient<Header> + Send + Sync + 'static,
	RpcClientFactory: SubstrateRpcClientFactory<Header, RpcClient> + Send + Sync + 'static,
>(
	module: &mut RpcModule<
		RpcContext<
			Header,
			RpcClient,
			RpcClientFactory,
			EthereumIntentExecutor,
			SolanaIntentExecutor,
			CrossChainIntentExecutor,
		>,
	>,
) {
	module
		.register_async_method("pumpx_signLimitOrder", |params, ctx, _| async move {
			let params = params.parse::<SignLimitOrderParams>().map_err(|e| {
				error!("Failed to parse params: {:?}", e);
				PumpxRpcError::from_error_code(ErrorCode::ParseError)
			})?;

			handle_sign_limit_order_request(params, ctx).await
		})
		.expect("Failed to register pumpx_signLimitOrder method");
}
